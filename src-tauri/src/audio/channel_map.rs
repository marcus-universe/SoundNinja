//! Stereo L/R routing onto a multi-channel ASIO (or similar) device.
//!
//! Playback stays stereo in the PCM cache. A [`ChannelScatter`] source expands
//! each frame to `dst_ch` channels, writing L/R into the selected slots.
//! Capture does the inverse: pick two hardware channels and emit stereo.

use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::Duration;

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};

static ENABLED: AtomicBool = AtomicBool::new(false);
static OUT_LEFT: AtomicU16 = AtomicU16::new(0);
static OUT_RIGHT: AtomicU16 = AtomicU16::new(1);
static IN_LEFT: AtomicU16 = AtomicU16::new(0);
static IN_RIGHT: AtomicU16 = AtomicU16::new(1);

pub fn configure(enabled: bool, out_left: u16, out_right: u16, in_left: u16, in_right: u16) {
    ENABLED.store(enabled, Ordering::Relaxed);
    OUT_LEFT.store(out_left, Ordering::Relaxed);
    OUT_RIGHT.store(out_right, Ordering::Relaxed);
    IN_LEFT.store(in_left, Ordering::Relaxed);
    IN_RIGHT.store(in_right, Ordering::Relaxed);
}

pub fn output_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn input_enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn mix_channels(device_channels: ChannelCount) -> ChannelCount {
    if output_enabled() {
        ChannelCount::new(2).unwrap_or(device_channels)
    } else {
        device_channels
    }
}

/// Scatter one stereo (or mono) interleaved buffer into `dst_ch` channels.
pub fn scatter_interleaved(
    stereo: &[f32],
    src_ch: u16,
    dst_ch: u16,
    left: u16,
    right: u16,
) -> Vec<f32> {
    let src = src_ch.max(1) as usize;
    let dst = dst_ch.max(1) as usize;
    let left = (left as usize).min(dst.saturating_sub(1));
    let right = (right as usize).min(dst.saturating_sub(1));
    if src == dst && left == 0 && right == 1.min(dst.saturating_sub(1)) && src == 2 {
        return stereo.to_vec();
    }
    let frames = stereo.len() / src;
    let mut out = vec![0.0f32; frames * dst];
    for i in 0..frames {
        let (l, r) = if src == 1 {
            let s = stereo[i];
            (s, s)
        } else {
            let base = i * src;
            (stereo[base], stereo[base + 1])
        };
        let dst_base = i * dst;
        out[dst_base + left] = l;
        out[dst_base + right] = r;
    }
    out
}

/// Pick two channels from an interleaved N-channel buffer and emit stereo.
pub fn extract_stereo(interleaved: &[f32], src_ch: u16) -> Vec<f32> {
    extract_stereo_at(
        interleaved,
        src_ch,
        IN_LEFT.load(Ordering::Relaxed),
        IN_RIGHT.load(Ordering::Relaxed),
    )
}

pub fn extract_stereo_at(interleaved: &[f32], src_ch: u16, left: u16, right: u16) -> Vec<f32> {
    let ch = src_ch.max(1) as usize;
    if ch == 1 {
        let mut out = Vec::with_capacity(interleaved.len() * 2);
        for &s in interleaved {
            out.push(s);
            out.push(s);
        }
        return out;
    }
    if ch == 2 && left == 0 && right == 1 {
        return interleaved.to_vec();
    }
    let frames = interleaved.len() / ch;
    let li = (left as usize).min(ch - 1);
    let ri = (right as usize).min(ch - 1);
    let mut out = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let base = i * ch;
        out.push(interleaved[base + li]);
        out.push(interleaved[base + ri]);
    }
    out
}

/// Expands a stereo (or mono) [`Source`] to `dst_ch` channels.
/// L/R indices are read from atomics every frame so settings can change live.
pub struct ChannelScatter<S> {
    inner: S,
    dst_ch: u16,
    frame: Vec<f32>,
    pos: usize,
}

impl<S: Source> ChannelScatter<S> {
    pub fn new(inner: S, dst_ch: u16) -> Self {
        let dst_ch = dst_ch.max(1);
        Self {
            inner,
            dst_ch,
            frame: vec![0.0; dst_ch as usize],
            pos: dst_ch as usize,
        }
    }

    fn pull_frame(&mut self) -> bool {
        let src_ch = self.inner.channels().get().max(1);
        let left_s = match self.inner.next() {
            Some(s) => s,
            None => return false,
        };
        let right_s = if src_ch == 1 {
            left_s
        } else {
            let r = self.inner.next().unwrap_or(left_s);
            for _ in 2..src_ch {
                let _ = self.inner.next();
            }
            r
        };
        let dst = self.dst_ch.max(1) as usize;
        if self.frame.len() != dst {
            self.frame.resize(dst, 0.0);
        }
        self.frame.fill(0.0);
        let left = (OUT_LEFT.load(Ordering::Relaxed) as usize).min(dst.saturating_sub(1));
        let right = (OUT_RIGHT.load(Ordering::Relaxed) as usize).min(dst.saturating_sub(1));
        self.frame[left] = left_s;
        self.frame[right] = right_s;
        self.pos = 0;
        true
    }
}

impl<S: Source> Iterator for ChannelScatter<S> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.pos >= self.frame.len() && !self.pull_frame() {
            return None;
        }
        let s = self.frame[self.pos];
        self.pos += 1;
        Some(s)
    }
}

impl<S: Source> Source for ChannelScatter<S> {
    fn current_span_len(&self) -> Option<usize> {
        let inner = self.inner.current_span_len()?;
        let src = self.inner.channels().get().max(1) as usize;
        let dst = self.dst_ch.max(1) as usize;
        Some(inner / src * dst)
    }

    fn channels(&self) -> ChannelCount {
        ChannelCount::new(self.dst_ch.max(1))
            .or_else(|| ChannelCount::new(1))
            .expect("ChannelCount 1 is always valid")
    }

    fn sample_rate(&self) -> SampleRate {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        self.inner.try_seek(pos)?;
        self.pos = self.frame.len();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{extract_stereo_at, scatter_interleaved};

    #[test]
    fn scatter_stereo_into_8ch_slots_2_and_5() {
        let stereo = vec![0.5, -0.5, 0.25, -0.25];
        let out = scatter_interleaved(&stereo, 2, 8, 2, 5);
        assert_eq!(out.len(), 16);
        assert_eq!(out[2], 0.5);
        assert_eq!(out[5], -0.5);
        assert_eq!(out[8 + 2], 0.25);
        assert_eq!(out[8 + 5], -0.25);
        assert!(out.iter().enumerate().all(|(i, s)| {
            let slot = i % 8;
            slot == 2 || slot == 5 || *s == 0.0
        }));
    }

    #[test]
    fn extract_stereo_picks_indices() {
        // frames of 4ch: [0,1,2,3] then [4,5,6,7]; pick ch 0 and 2
        let src = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let out = extract_stereo_at(&src, 4, 0, 2);
        assert_eq!(out, vec![0.1, 0.3, 0.5, 0.7]);
    }

    #[test]
    fn extract_mono_duplicates() {
        let src = vec![0.2, -0.4];
        assert_eq!(extract_stereo_at(&src, 1, 0, 0), vec![0.2, 0.2, -0.4, -0.4]);
    }
}
