//! Incremental waveform peaks.
//!
//! A [`PeakAcc`] keeps one absolute peak per fixed window of frames while audio
//! streams through it. Any number of display buckets can then be produced in
//! `O(buckets + windows)` without touching the samples again, which is what
//! lets the pump, the PCM cache and the recorder answer waveform requests
//! without holding a lock across a fold of the whole clip.

/// Frames per window for a fresh accumulator. Small, so short clips keep
/// sample-level detail; long audio coarsens itself (see [`PeakAcc::push`]).
const INITIAL_WINDOW_FRAMES: usize = 16;
/// Window budget for file / pump / cache waveforms: 64 KiB of peaks per clip,
/// and always at least half of this many windows across long audio — more
/// than any scrubber draws.
const DEFAULT_MAX_WINDOWS: usize = 16_384;

#[derive(Clone, Debug)]
pub(crate) struct PeakAcc {
    /// Frames covered by one entry of `peaks`. Doubles when the budget is hit.
    window_frames: usize,
    /// Upper bound for `peaks.len()`; always even so windows merge in pairs.
    max_windows: usize,
    /// Absolute peak of each completed window.
    peaks: Vec<f32>,
    /// Absolute peak of the window still being filled.
    partial: f32,
    /// Frames already folded into `partial`.
    partial_frames: usize,
}

impl Default for PeakAcc {
    fn default() -> Self {
        Self::new()
    }
}

impl PeakAcc {
    pub(crate) fn new() -> Self {
        Self::with_limits(INITIAL_WINDOW_FRAMES, DEFAULT_MAX_WINDOWS)
    }

    /// `window_frames` is the starting resolution, `max_windows` the memory
    /// budget (one `f32` per window).
    pub(crate) fn with_limits(window_frames: usize, max_windows: usize) -> Self {
        Self {
            window_frames: window_frames.max(1),
            // Even and at least 2, so a merge always halves the list exactly.
            max_windows: (max_windows.max(2) / 2) * 2,
            peaks: Vec::new(),
            partial: 0.0,
            partial_frames: 0,
        }
    }

    pub(crate) fn from_samples(samples: &[f32], channels: u16) -> Self {
        let mut acc = Self::new();
        acc.push(samples, channels);
        acc
    }

    /// Frames folded in so far.
    pub(crate) fn frames(&self) -> usize {
        self.peaks.len() * self.window_frames + self.partial_frames
    }

    /// Fold interleaved samples. Callers pass whole frames; a trailing
    /// incomplete frame is counted as one frame rather than dropped.
    pub(crate) fn push(&mut self, samples: &[f32], channels: u16) {
        let ch = channels.max(1) as usize;
        let mut rest = samples;
        while !rest.is_empty() {
            let room_frames = self.window_frames - self.partial_frames;
            let take = (room_frames * ch).min(rest.len());
            let (head, tail) = rest.split_at(take);
            // The peak of a frame is the max over its channels and the peak of
            // a window the max over its frames, so one pass over the raw
            // samples gives the same answer without per-frame bookkeeping.
            self.partial = head.iter().fold(self.partial, |peak, s| peak.max(s.abs()));
            self.partial_frames += take.div_ceil(ch);
            rest = tail;
            if self.partial_frames >= self.window_frames {
                self.peaks.push(self.partial);
                self.partial = 0.0;
                self.partial_frames = 0;
                if self.peaks.len() >= self.max_windows {
                    self.coarsen();
                }
            }
        }
    }

    /// Merge neighbouring windows so memory stays bounded however long the
    /// audio is. The window in progress starts on a boundary of the doubled
    /// size (the list length was even), so it carries over unchanged.
    fn coarsen(&mut self) {
        let merged: Vec<f32> = self
            .peaks
            .chunks(2)
            .map(|pair| pair.iter().copied().fold(0.0f32, f32::max))
            .collect();
        self.peaks = merged;
        self.window_frames *= 2;
    }

    fn window_peak(&self, window: usize) -> f32 {
        self.peaks.get(window).copied().unwrap_or(self.partial)
    }

    /// `[min0, max0, min1, max1, …]` for `buckets` buckets spread over a
    /// timeline of `timeline_frames`. Every bucket that overlaps loaded audio
    /// gets a value, even when there are fewer windows than buckets; only the
    /// part of the timeline that has not been decoded yet stays at zero.
    pub(crate) fn to_buckets(&self, buckets: usize, timeline_frames: usize) -> Vec<f32> {
        self.range_to_buckets(buckets, 0, timeline_frames)
    }

    /// Same as [`Self::to_buckets`] for the frame range `start..end`.
    pub(crate) fn range_to_buckets(&self, buckets: usize, start: usize, end: usize) -> Vec<f32> {
        let buckets = buckets.max(1);
        let mut out = vec![0.0f32; buckets * 2];
        let loaded = self.frames();
        if end <= start || loaded == 0 {
            return out;
        }
        let span = (end - start) as u64;
        let bucket_start =
            |b: usize| start + ((b as u64 * span) / buckets as u64) as usize;
        for b in 0..buckets {
            let first = bucket_start(b);
            if first >= loaded {
                break;
            }
            let last = bucket_start(b + 1).max(first + 1).min(end).min(loaded) - 1;
            let peak = (first / self.window_frames..=last / self.window_frames)
                .map(|w| self.window_peak(w))
                .fold(0.0f32, f32::max);
            if peak > 0.0 {
                out[b * 2] = -peak;
                out[b * 2 + 1] = peak;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::PeakAcc;

    fn maxs(peaks: &[f32]) -> Vec<f32> {
        peaks.chunks_exact(2).map(|p| p[1]).collect()
    }

    #[test]
    fn to_buckets_should_map_partial_decode_onto_full_timeline() {
        // 4 mono frames decoded, 8-frame metadata timeline, 4 buckets.
        // Energy in the left half; unloaded tail stays empty.
        let mut acc = PeakAcc::with_limits(1, 64);
        acc.push(&[1.0, 1.0, 1.0, 1.0], 1);
        let peaks = acc.to_buckets(4, 8);
        assert_eq!(peaks.len(), 8);
        assert!(peaks[1] > 0.5);
        assert!(peaks[3] > 0.5);
        assert_eq!(peaks[4], 0.0);
        assert_eq!(peaks[5], 0.0);
        assert_eq!(peaks[6], 0.0);
        assert_eq!(peaks[7], 0.0);
    }

    #[test]
    fn to_buckets_should_fill_all_buckets_when_timeline_matches_ready() {
        let mut acc = PeakAcc::with_limits(1, 64);
        acc.push(&[1.0, 1.0, 1.0, 1.0], 1);
        let peaks = acc.to_buckets(4, 4);
        assert!(maxs(&peaks).iter().all(|&m| m > 0.5));
    }

    #[test]
    fn to_buckets_has_no_gaps_when_there_are_fewer_windows_than_buckets() {
        // 10 windows of 1024 frames spread over 100 buckets: the old fold left
        // 90 buckets at zero, which drew short clips as a comb.
        let mut acc = PeakAcc::with_limits(1024, 4096);
        acc.push(&vec![0.5f32; 10 * 1024], 1);
        let peaks = acc.to_buckets(100, acc.frames());
        assert!(maxs(&peaks).iter().all(|&m| m > 0.4), "{peaks:?}");
    }

    #[test]
    fn to_buckets_leaves_only_the_undecoded_tail_empty() {
        // Half of a 20-window timeline is decoded.
        let mut acc = PeakAcc::with_limits(1024, 4096);
        acc.push(&vec![0.5f32; 10 * 1024], 1);
        let maxs = maxs(&acc.to_buckets(100, 20 * 1024));
        assert!(maxs[..50].iter().all(|&m| m > 0.4));
        assert!(maxs[50..].iter().all(|&m| m == 0.0));
    }

    #[test]
    fn push_takes_the_loudest_channel_of_each_frame() {
        let mut acc = PeakAcc::with_limits(2, 64);
        // Two stereo frames: quiet left, loud right.
        acc.push(&[0.1, -0.9, 0.1, 0.2], 2);
        assert_eq!(acc.frames(), 2);
        let peaks = acc.to_buckets(1, 2);
        assert!((peaks[1] - 0.9).abs() < 1e-6);
        assert!((peaks[0] + 0.9).abs() < 1e-6);
    }

    #[test]
    fn push_keeps_loud_then_quiet_order_across_split_pushes() {
        let mut acc = PeakAcc::new();
        let loud = vec![0.9f32; 2 * 1024];
        let quiet = vec![0.05f32; 2 * 1024];
        // Uneven slices: window boundaries must not depend on push sizes.
        for part in loud.chunks(700).chain(quiet.chunks(300)) {
            acc.push(part, 2);
        }
        assert_eq!(acc.frames(), 2 * 1024);
        let peaks = acc.to_buckets(4, acc.frames());
        assert!(peaks[1] > 0.8 && peaks[3] > 0.8);
        assert!(peaks[5] < 0.1 && peaks[7] < 0.1);
    }

    #[test]
    fn push_coarsens_windows_instead_of_growing_without_bound() {
        let mut acc = PeakAcc::with_limits(1, 8);
        let ramp: Vec<f32> = (0..100).map(|i| i as f32 / 100.0).collect();
        acc.push(&ramp, 1);
        assert_eq!(acc.frames(), 100);
        assert!(acc.peaks.len() <= 8);
        // The loudest sample survives every merge and stays at the end.
        let peaks = acc.to_buckets(4, 100);
        assert!((peaks[7] - 0.99).abs() < 1e-6);
        assert!(peaks[1] < peaks[7]);
    }

    #[test]
    fn range_to_buckets_reads_only_the_requested_span() {
        let mut acc = PeakAcc::with_limits(1, 64);
        acc.push(&[0.1, 0.1, 0.8, 0.8], 1);
        let tail = acc.range_to_buckets(2, 2, 4);
        assert!(maxs(&tail).iter().all(|&m| (m - 0.8).abs() < 1e-6));
        assert!(acc.range_to_buckets(2, 3, 3).iter().all(|&p| p == 0.0));
    }
}
