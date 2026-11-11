//! Sample-rate and channel conversion shared by the PCM cache and the pump.
//!
//! Both paths feed the same [`FormatConverter`], so a clip sounds the same
//! whether it was warmed into the cache or started cold. Nothing here runs in
//! the audio callback.

use rubato::{
    FftFixedInOut, Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

/// Input frames per resampler block. The FFT engine rounds this up to a
/// multiple of its ratio unit (1029 frames for 44.1 kHz → 48 kHz).
const CHUNK_FRAMES: usize = 1024;
/// Largest ratio unit (`from / gcd(from, to)`) the FFT engine is used for.
/// Odd rates such as 44 099 Hz would need an FFT as long as the rate itself.
const MAX_FFT_UNIT: u32 = 4096;

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

enum Engine {
    /// Fixed-ratio FFT resampler: fast and transparent for standard rates.
    Fft(FftFixedInOut<f32>),
    /// Windowed-sinc fallback for ratios the FFT engine cannot hold.
    Sinc(SincFixedIn<f32>),
}

impl Engine {
    fn input_frames(&self) -> usize {
        match self {
            Self::Fft(r) => r.input_frames_next(),
            Self::Sinc(r) => r.input_frames_next(),
        }
    }

    fn output_frames_max(&self) -> usize {
        match self {
            Self::Fft(r) => r.output_frames_max(),
            Self::Sinc(r) => r.output_frames_max(),
        }
    }

    /// Output frames by which the engine's first output lags the input.
    fn output_delay(&self) -> usize {
        match self {
            Self::Fft(r) => r.output_delay(),
            // `SincFixedIn` starts its read position half a filter length
            // before the first input frame, so its output is already aligned
            // (the impulse test below pins this). It only holds back the last
            // ~sinc_len input frames of each block, which `finish` flushes.
            // Its `output_delay()` reports that holdback, not a time shift.
            Self::Sinc(_) => 0,
        }
    }

    fn process(&mut self, input: &[&[f32]], output: &mut [Vec<f32>]) -> Result<usize, String> {
        let result = match self {
            Self::Fft(r) => r.process_into_buffer(input, output, None),
            Self::Sinc(r) => r.process_into_buffer(input, output, None),
        };
        result.map(|(_, produced)| produced).map_err(|e| e.to_string())
    }
}

/// Streaming resampler over interleaved frames.
///
/// Push input in slices of any size, then call [`Self::finish`]. The filter
/// delay is removed from the front and the tail is flushed, so the total
/// output is exactly `ceil(frames_in * to / from)` frames and lines up in time
/// with the input — a clip neither starts late nor loses its end.
pub(crate) struct StreamResampler {
    engine: Engine,
    channels: usize,
    from: u32,
    to: u32,
    chunk_in: usize,
    /// Per-channel input waiting for a full block.
    pending: Vec<Vec<f32>>,
    /// Per-channel output of the block just processed.
    scratch: Vec<Vec<f32>>,
    /// Leading output frames still to drop (the engine's delay).
    skip: usize,
    frames_in: u64,
    frames_out: u64,
}

impl StreamResampler {
    pub(crate) fn new(from: u32, to: u32, channels: usize) -> Result<Self, String> {
        if from == 0 || to == 0 {
            return Err(format!("Invalid sample rate {from} -> {to}"));
        }
        let channels = channels.max(1);
        let engine = if from / gcd(from, to) <= MAX_FFT_UNIT {
            Engine::Fft(
                FftFixedInOut::<f32>::new(from as usize, to as usize, CHUNK_FRAMES, channels)
                    .map_err(|e| e.to_string())?,
            )
        } else {
            let params = SincInterpolationParameters {
                sinc_len: 64,
                f_cutoff: 0.95,
                interpolation: SincInterpolationType::Linear,
                oversampling_factor: 256,
                window: WindowFunction::BlackmanHarris2,
            };
            Engine::Sinc(
                SincFixedIn::<f32>::new(to as f64 / from as f64, 2.0, params, CHUNK_FRAMES, channels)
                    .map_err(|e| e.to_string())?,
            )
        };
        let chunk_in = engine.input_frames();
        let out_max = engine.output_frames_max();
        Ok(Self {
            skip: engine.output_delay(),
            engine,
            channels,
            from,
            to,
            chunk_in,
            pending: vec![Vec::with_capacity(chunk_in * 2); channels],
            scratch: vec![vec![0.0; out_max]; channels],
            frames_in: 0,
            frames_out: 0,
        })
    }

    #[cfg(test)]
    fn uses_fft(&self) -> bool {
        matches!(self.engine, Engine::Fft(_))
    }

    /// Output frames owed for the input seen so far.
    fn expected_frames(&self) -> u64 {
        (self.frames_in * self.to as u64).div_ceil(self.from as u64)
    }

    /// Resample `input` (interleaved, whole frames) and append the frames that
    /// are ready to `out`.
    pub(crate) fn push(&mut self, input: &[f32], out: &mut Vec<f32>) -> Result<(), String> {
        let ch = self.channels;
        let frames = input.len() / ch;
        if frames == 0 {
            return Ok(());
        }
        if ch == 1 {
            self.pending[0].extend_from_slice(&input[..frames]);
        } else {
            for frame in input.chunks_exact(ch) {
                for (plane, &s) in self.pending.iter_mut().zip(frame) {
                    plane.push(s);
                }
            }
        }
        self.frames_in += frames as u64;

        let mut pos = 0;
        while self.pending[0].len() - pos >= self.chunk_in {
            self.process_block(pos, None, out)?;
            pos += self.chunk_in;
        }
        // One drain per push: removing each block separately would shift the
        // remaining input once per block.
        for plane in &mut self.pending {
            plane.drain(..pos);
        }
        Ok(())
    }

    /// Flush what the filter still holds. Call once, after the last `push`.
    pub(crate) fn finish(&mut self, out: &mut Vec<f32>) -> Result<(), String> {
        let expected = self.expected_frames();
        // Zero-pad the remaining input to a full block, then keep feeding
        // silence until the delayed tail has come out. Two blocks cover every
        // engine's delay; the bound only guards against a stuck resampler.
        for plane in &mut self.pending {
            plane.resize(self.chunk_in, 0.0);
        }
        let mut blocks = 0;
        while self.frames_out < expected && blocks < 16 {
            self.process_block(0, Some(expected), out)?;
            for plane in &mut self.pending {
                plane.fill(0.0);
            }
            blocks += 1;
        }
        for plane in &mut self.pending {
            plane.clear();
        }
        // Never hand back a short buffer: a wrong length plays at the wrong
        // speed relative to everything computed from the frame count.
        let missing = expected.saturating_sub(self.frames_out) as usize;
        out.resize(out.len() + missing * self.channels, 0.0);
        self.frames_out += missing as u64;
        Ok(())
    }

    /// Run one block starting at `pos` in `pending` and interleave its output
    /// into `out`, dropping the delay and anything past `limit` total frames.
    fn process_block(
        &mut self,
        pos: usize,
        limit: Option<u64>,
        out: &mut Vec<f32>,
    ) -> Result<(), String> {
        let produced = {
            let input: Vec<&[f32]> = self
                .pending
                .iter()
                .map(|plane| &plane[pos..pos + self.chunk_in])
                .collect();
            self.engine.process(&input, &mut self.scratch)?
        };
        let start = self.skip.min(produced);
        self.skip -= start;
        let mut take = produced - start;
        if let Some(limit) = limit {
            take = take.min(limit.saturating_sub(self.frames_out) as usize);
        }
        let range = start..start + take;
        match self.scratch.as_slice() {
            [mono] => out.extend_from_slice(&mono[range]),
            planes => {
                out.reserve(take * planes.len());
                for i in range {
                    for plane in planes {
                        out.push(plane[i]);
                    }
                }
            }
        }
        self.frames_out += take as u64;
        Ok(())
    }
}

/// Converts interleaved audio to another rate and channel count.
///
/// Channel count is reduced before resampling and raised after it, so the
/// expensive stage always runs on the fewest channels.
pub(crate) struct FormatConverter {
    src_ch: usize,
    dst_ch: usize,
    resampler: Option<StreamResampler>,
    /// Scratch: input after the down-mix.
    mixed: Vec<f32>,
    /// Scratch: resampler output before the up-mix.
    resampled: Vec<f32>,
}

impl FormatConverter {
    pub(crate) fn new(
        src_rate: u32,
        src_channels: u16,
        dst_rate: u32,
        dst_channels: u16,
    ) -> Result<Self, String> {
        let src_ch = src_channels.max(1) as usize;
        let dst_ch = dst_channels.max(1) as usize;
        let resampler = if src_rate != dst_rate {
            Some(StreamResampler::new(src_rate, dst_rate, src_ch.min(dst_ch))?)
        } else {
            None
        };
        Ok(Self {
            src_ch,
            dst_ch,
            resampler,
            mixed: Vec::new(),
            resampled: Vec::new(),
        })
    }

    /// True when input already is in the output format and can be used as is.
    pub(crate) fn is_passthrough(&self) -> bool {
        self.resampler.is_none() && self.src_ch == self.dst_ch
    }

    /// Convert `input` (interleaved source frames) and append to `out`.
    pub(crate) fn push(&mut self, input: &[f32], out: &mut Vec<f32>) -> Result<(), String> {
        if self.is_passthrough() {
            out.extend_from_slice(input);
            return Ok(());
        }
        let frames = input.len() / self.src_ch;
        let input = &input[..frames * self.src_ch];
        let mid_ch = self.src_ch.min(self.dst_ch);

        let mixed: &[f32] = if self.dst_ch < self.src_ch {
            self.mixed.clear();
            down_mix(input, self.src_ch, self.dst_ch, &mut self.mixed);
            &self.mixed
        } else {
            input
        };
        let resampled: &[f32] = match self.resampler.as_mut() {
            Some(resampler) => {
                self.resampled.clear();
                resampler.push(mixed, &mut self.resampled)?;
                &self.resampled
            }
            None => mixed,
        };
        up_mix(resampled, mid_ch, self.dst_ch, out);
        Ok(())
    }

    /// Flush the resampler tail. Call once, after the last `push`.
    pub(crate) fn finish(&mut self, out: &mut Vec<f32>) -> Result<(), String> {
        let Some(resampler) = self.resampler.as_mut() else {
            return Ok(());
        };
        self.resampled.clear();
        resampler.finish(&mut self.resampled)?;
        up_mix(&self.resampled, self.src_ch.min(self.dst_ch), self.dst_ch, out);
        Ok(())
    }
}

/// Down-mix to mono by averaging; any other reduction keeps the leading
/// channels rather than matrixing them.
fn down_mix(input: &[f32], src_ch: usize, dst_ch: usize, out: &mut Vec<f32>) {
    out.reserve(input.len() / src_ch * dst_ch);
    if dst_ch == 1 {
        let scale = 1.0 / src_ch as f32;
        out.extend(
            input
                .chunks_exact(src_ch)
                .map(|frame| frame.iter().sum::<f32>() * scale),
        );
    } else {
        for frame in input.chunks_exact(src_ch) {
            out.extend_from_slice(&frame[..dst_ch]);
        }
    }
}

/// Raise the channel count by cycling the existing channels; a plain copy when
/// the counts already match.
fn up_mix(input: &[f32], src_ch: usize, dst_ch: usize, out: &mut Vec<f32>) {
    if src_ch == dst_ch {
        out.extend_from_slice(input);
        return;
    }
    out.reserve(input.len() / src_ch * dst_ch);
    for frame in input.chunks_exact(src_ch) {
        out.extend((0..dst_ch).map(|i| frame[i % src_ch]));
    }
}

/// Resample one channel. Shared with the edit-session DSP so both paths sound
/// identical.
pub fn resample_mono(input: &[f32], from: u32, to: u32) -> Result<Vec<f32>, String> {
    if from == to || input.is_empty() {
        return Ok(input.to_vec());
    }
    let mut resampler = StreamResampler::new(from, to, 1)?;
    let expected = (input.len() as u64 * to as u64).div_ceil(from as u64) as usize;
    let mut out = Vec::with_capacity(expected);
    // Bounded slices keep the resampler's pending buffer small for long input.
    for part in input.chunks(CHUNK_FRAMES * 64) {
        resampler.push(part, &mut out)?;
    }
    resampler.finish(&mut out)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{resample_mono, FormatConverter, StreamResampler};

    fn expected_len(n: usize, from: u32, to: u32) -> usize {
        (n as u64 * to as u64).div_ceil(from as u64) as usize
    }

    fn rms(samples: &[f32]) -> f64 {
        let sum: f64 = samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
        (sum / samples.len().max(1) as f64).sqrt()
    }

    fn sine(freq: f64, rate: u32, frames: usize) -> Vec<f32> {
        (0..frames)
            .map(|i| (std::f64::consts::TAU * freq * i as f64 / rate as f64).sin() as f32 * 0.5)
            .collect()
    }

    #[test]
    fn resample_mono_same_rate_is_identity() {
        let input = vec![0.1, -0.2, 0.3];
        assert_eq!(resample_mono(&input, 48_000, 48_000).unwrap(), input);
    }

    #[test]
    fn resample_mono_empty_stays_empty() {
        assert!(resample_mono(&[], 44_100, 48_000).unwrap().is_empty());
    }

    #[test]
    fn resample_mono_short_input_is_not_padded_to_chunk() {
        let input = vec![0.2f32; 32];
        let out = resample_mono(&input, 44_100, 48_000).unwrap();
        let expect = (32.0f64 * 48_000.0 / 44_100.0).round() as usize;
        let delta = (out.len() as isize - expect as isize).unsigned_abs();
        assert!(delta < 64, "len {} vs about {}", out.len(), expect);
    }

    #[test]
    fn resample_mono_output_length_is_exact_for_common_rates() {
        let cases = [
            (44_100u32, 48_000u32),
            (48_000, 44_100),
            (22_050, 48_000),
            (96_000, 48_000),
            (8_000, 44_100),
            (44_099, 48_000),
        ];
        for (from, to) in cases {
            for n in [1usize, 31, 1_024, 1_029, 5_000, 44_100] {
                let out = resample_mono(&vec![0.1f32; n], from, to).unwrap();
                assert_eq!(out.len(), expected_len(n, from, to), "{n} frames {from}->{to}");
            }
        }
    }

    #[test]
    fn resample_mono_keeps_an_impulse_at_its_time() {
        for (from, to) in [(44_100u32, 48_000u32), (48_000, 44_100), (44_099, 48_000)] {
            for k in [0usize, 100, 3_000, 9_999] {
                let mut input = vec![0.0f32; 10_000];
                input[k] = 1.0;
                let out = resample_mono(&input, from, to).unwrap();
                let (peak_at, _) = out
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
                    .unwrap();
                let want = k as f64 * to as f64 / from as f64;
                assert!(
                    (peak_at as f64 - want).abs() <= 1.0,
                    "{from}->{to}: impulse at {k} landed at {peak_at}, want {want:.1}"
                );
            }
        }
    }

    #[test]
    fn resample_mono_preserves_the_level_of_a_1khz_sine() {
        for (from, to) in [(44_100u32, 48_000u32), (48_000, 44_100), (44_099, 48_000)] {
            let input = sine(1_000.0, from, from as usize);
            let out = resample_mono(&input, from, to).unwrap();
            // Skip the edges, where the filter sees the silence around the clip.
            let trim_in = from as usize / 10;
            let trim_out = to as usize / 10;
            let before = rms(&input[trim_in..input.len() - trim_in]);
            let after = rms(&out[trim_out..out.len() - trim_out]);
            assert!(
                (after / before - 1.0).abs() < 0.01,
                "{from}->{to}: rms {before:.5} -> {after:.5}"
            );
        }
    }

    #[test]
    fn stream_resampler_uses_fft_for_standard_rates_and_sinc_for_odd_ones() {
        assert!(StreamResampler::new(44_100, 48_000, 2).unwrap().uses_fft());
        assert!(StreamResampler::new(48_000, 44_100, 1).unwrap().uses_fft());
        assert!(!StreamResampler::new(44_099, 48_000, 2).unwrap().uses_fft());
    }

    #[test]
    fn stream_resampler_output_does_not_depend_on_push_sizes() {
        let input = sine(440.0, 44_100, 20_000);
        let whole = resample_mono(&input, 44_100, 48_000).unwrap();

        let mut resampler = StreamResampler::new(44_100, 48_000, 1).unwrap();
        let mut pieces = Vec::new();
        for part in input.chunks(777) {
            resampler.push(part, &mut pieces).unwrap();
        }
        resampler.finish(&mut pieces).unwrap();
        assert_eq!(whole, pieces);
    }

    #[test]
    fn format_converter_passes_matching_formats_through_untouched() {
        let mut conv = FormatConverter::new(48_000, 2, 48_000, 2).unwrap();
        assert!(conv.is_passthrough());
        let mut out = Vec::new();
        conv.push(&[0.1, 0.2, 0.3, 0.4], &mut out).unwrap();
        conv.finish(&mut out).unwrap();
        assert_eq!(out, vec![0.1, 0.2, 0.3, 0.4]);
    }

    #[test]
    fn format_converter_averages_to_mono_and_duplicates_to_stereo() {
        let mut to_mono = FormatConverter::new(48_000, 2, 48_000, 1).unwrap();
        let mut mono = Vec::new();
        to_mono.push(&[0.2, 0.4, -1.0, 1.0], &mut mono).unwrap();
        to_mono.finish(&mut mono).unwrap();
        assert!((mono[0] - 0.3).abs() < 1e-6 && mono[1].abs() < 1e-6);

        let mut to_stereo = FormatConverter::new(48_000, 1, 48_000, 2).unwrap();
        let mut stereo = Vec::new();
        to_stereo.push(&[0.5, -0.5], &mut stereo).unwrap();
        to_stereo.finish(&mut stereo).unwrap();
        assert_eq!(stereo, vec![0.5, 0.5, -0.5, -0.5]);
    }

    #[test]
    fn format_converter_resamples_stereo_to_the_exact_frame_count() {
        let frames = 4_410;
        let input: Vec<f32> = (0..frames).flat_map(|i| [i as f32 / frames as f32, 0.25]).collect();
        let mut conv = FormatConverter::new(44_100, 2, 48_000, 2).unwrap();
        let mut out = Vec::new();
        for part in input.chunks(2 * 500) {
            conv.push(part, &mut out).unwrap();
        }
        conv.finish(&mut out).unwrap();
        assert_eq!(out.len(), expected_len(frames, 44_100, 48_000) * 2);
        // The constant right channel stays constant away from the clip edges.
        let right_mid = out[out.len() / 2 + 1];
        assert!((right_mid - 0.25).abs() < 0.01, "right {right_mid}");
    }
}
