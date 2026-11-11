import { invoke } from '@tauri-apps/api/core'

/** What `get_file_waveform` returns. `peaks` are interleaved [min, max] pairs. */
export interface WaveformResult {
  peaks: number[]
  /** The file is still being decoded; `peaks` covers only what is loaded so far. */
  pumping: boolean
  /** Best known duration in seconds, 0 when unknown. Exact once `pumping` is false. */
  durationSecs: number
}

/**
 * Waveform, decode state and duration of a sound in one IPC call.
 *
 * Not cached here on purpose: the backend keeps finished waveforms keyed by
 * file size and modification time, which this side cannot see. A cache by path
 * alone would show the old shape after a file is re-recorded.
 */
export async function fetchWaveform(path: string, buckets: number): Promise<WaveformResult> {
  const raw = await invoke<Partial<WaveformResult> | null>('get_file_waveform', { path, buckets })
  const duration = Number(raw?.durationSecs)
  return {
    peaks: Array.isArray(raw?.peaks) ? raw.peaks : [],
    pumping: raw?.pumping === true,
    durationSecs: Number.isFinite(duration) && duration > 0 ? duration : 0,
  }
}
