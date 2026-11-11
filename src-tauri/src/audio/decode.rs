//! Opening audio files for decoding.
//!
//! One place builds decoders so every caller gets the same settings, and one
//! batch-oriented source type feeds both the PCM cache and the progressive
//! pump, so a warmed and a cold play decode the exact same samples.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rodio::Source;

use super::pcm::int_sample_to_f32;

const READ_BUFFER_BYTES: usize = 64 * 1024;

fn build_decoder(
    path: &str,
    hint: Option<&str>,
) -> Result<rodio::Decoder<BufReader<File>>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open '{path}': {e}"))?;
    let byte_len = file
        .metadata()
        .map_err(|e| format!("Failed to stat '{path}': {e}"))?
        .len();
    // The byte length is what makes container durations trustworthy: Vorbis
    // reports its exact length, header-less MP3 gets an estimate, and M4A
    // files with the index at the end become decodable at all.
    let mut builder = rodio::Decoder::builder()
        .with_data(BufReader::with_capacity(READ_BUFFER_BYTES, file))
        .with_byte_len(byte_len);
    if let Some(ext) = hint {
        builder = builder.with_hint(ext);
    }
    builder.build().map_err(|e| e.to_string())
}

/// Decoder for a file on disk. Reads through a buffered handle, never the
/// whole file, so probing a duration only touches the headers.
pub(crate) fn open_decoder(path: &str) -> Result<rodio::Decoder<BufReader<File>>, String> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        // A mislabelled file (WAV named `.mp3`) must still open: the hint only
        // speeds up format detection, so retry without it before giving up.
        Some(ext) => build_decoder(path, Some(ext)).or_else(|_| build_decoder(path, None)),
        None => build_decoder(path, None),
    }
}

/// Interleaved f32 samples pulled in batches. WAV goes through hound for exact
/// PCM; everything else through rodio.
pub(crate) enum SampleSource {
    Wav(WavSource),
    Rodio(Box<rodio::Decoder<BufReader<File>>>),
}

/// A [`SampleSource`] plus the native format it produces.
pub(crate) struct OpenedSource {
    pub source: SampleSource,
    pub sample_rate: u32,
    pub channels: u16,
    /// Length from the container, `0.0` when it does not say.
    pub duration_hint_secs: f64,
}

impl SampleSource {
    pub(crate) fn open(path: &str) -> Result<OpenedSource, String> {
        if path.to_lowercase().ends_with(".wav") {
            if let Ok(opened) = WavSource::open(path) {
                return Ok(opened);
            }
        }
        let decoder = open_decoder(path)?;
        let sample_rate: u32 = decoder.sample_rate().into();
        let channels: u16 = decoder.channels().into();
        let duration_hint_secs = decoder
            .total_duration()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        Ok(OpenedSource {
            source: Self::Rodio(Box::new(decoder)),
            sample_rate,
            channels,
            duration_hint_secs,
        })
    }

    /// Append up to `max_samples` samples to `out`. Returns how many were
    /// added; `0` means the end of the file.
    pub(crate) fn read(&mut self, out: &mut Vec<f32>, max_samples: usize) -> usize {
        let before = out.len();
        match self {
            Self::Wav(wav) => wav.read(out, max_samples),
            Self::Rodio(decoder) => out.extend(decoder.by_ref().take(max_samples)),
        }
        out.len() - before
    }
}

pub(crate) struct WavSource {
    reader: hound::WavReader<BufReader<File>>,
    bits: u16,
    float: bool,
}

impl WavSource {
    fn open(path: &str) -> Result<OpenedSource, String> {
        let reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
        let spec = reader.spec();
        let duration_hint_secs = reader.duration() as f64 / spec.sample_rate.max(1) as f64;
        Ok(OpenedSource {
            source: SampleSource::Wav(Self {
                bits: spec.bits_per_sample,
                float: spec.sample_format == hound::SampleFormat::Float,
                reader,
            }),
            sample_rate: spec.sample_rate,
            channels: spec.channels,
            duration_hint_secs,
        })
    }

    /// A damaged sample reads as silence instead of ending the clip early.
    fn read(&mut self, out: &mut Vec<f32>, max_samples: usize) {
        if self.float {
            out.extend(
                self.reader
                    .samples::<f32>()
                    .take(max_samples)
                    .map(|r| r.unwrap_or(0.0)),
            );
        } else {
            let bits = self.bits;
            out.extend(
                self.reader
                    .samples::<i32>()
                    .take(max_samples)
                    .map(|r| r.map(|s| int_sample_to_f32(s, bits)).unwrap_or(0.0)),
            );
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::path::Path;

    /// Write a 16-bit WAV for tests. `sample(frame, channel)` is in [-1, 1].
    pub(crate) fn write_wav(
        path: &Path,
        sample_rate: u32,
        channels: u16,
        frames: usize,
        sample: impl Fn(usize, u16) -> f32,
    ) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for frame in 0..frames {
            for channel in 0..channels {
                let value = (sample(frame, channel).clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                writer.write_sample(value).unwrap();
            }
        }
        writer.finalize().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::write_wav;
    use super::{open_decoder, SampleSource};
    use rodio::Source;

    fn fixture(name: &str) -> String {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
            .to_string_lossy()
            .into_owned()
    }

    #[test]
    fn open_decoder_reports_wav_duration_from_headers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tone.wav");
        write_wav(&path, 8_000, 1, 4_000, |i, _| (i as f32 * 0.05).sin() * 0.5);
        let decoder = open_decoder(&path.to_string_lossy()).unwrap();
        let secs = decoder.total_duration().unwrap().as_secs_f64();
        assert!((secs - 0.5).abs() < 0.001, "duration {secs}");
    }

    #[test]
    fn open_decoder_ignores_a_wrong_extension() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("really-a-wav.mp3");
        write_wav(&path, 8_000, 1, 800, |_, _| 0.25);
        let decoder = open_decoder(&path.to_string_lossy()).unwrap();
        assert_eq!(decoder.count(), 800);
    }

    #[test]
    fn open_decoder_fails_for_a_missing_file() {
        assert!(open_decoder("/no/such/file.wav").is_err());
    }

    #[test]
    fn sample_source_reads_wav_in_batches_until_the_end() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stereo.wav");
        write_wav(&path, 8_000, 2, 1_000, |_, ch| if ch == 0 { 0.5 } else { -0.5 });
        let mut opened = SampleSource::open(&path.to_string_lossy()).unwrap();
        assert_eq!((opened.sample_rate, opened.channels), (8_000, 2));
        assert!((opened.duration_hint_secs - 0.125).abs() < 1e-9);

        let mut samples = Vec::new();
        while opened.source.read(&mut samples, 300) > 0 {}
        assert_eq!(samples.len(), 2_000);
        assert!((samples[0] - 0.5).abs() < 0.001);
        assert!((samples[1] + 0.5).abs() < 0.001);
    }

    /// Header-less CBR MP3: only the byte length makes a duration available.
    #[test]
    fn open_decoder_estimates_mp3_duration_from_the_byte_length() {
        let decoder = open_decoder(&fixture("tone-1s.mp3")).unwrap();
        let secs = decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0);
        assert!((secs - 1.0).abs() < 0.15, "duration {secs}");
    }

    #[test]
    fn open_decoder_reports_exact_vorbis_duration() {
        let decoder = open_decoder(&fixture("tone-1s.ogg")).unwrap();
        let secs = decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0);
        assert!((secs - 1.0).abs() < 0.01, "duration {secs}");
    }
}
