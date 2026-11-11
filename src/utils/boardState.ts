import type { ComputedRef, InjectionKey } from 'vue'
import type { SoundProgress } from '~/utils/soundProgress'

/**
 * Board → SoundButton channel for state that changes per button.
 *
 * Passed as props, each of these made the board template depend on every
 * sound: one play, one finished load or one selection re-rendered all buttons.
 * Read per key inside the button, a change re-renders that button only.
 */
export interface BoardButtonState {
  /** Running playback progress by sound path. */
  progress: Map<string, SoundProgress>
  /** Paths whose playback was requested and has not started yet. */
  loading: Set<string>
  /** Paths whose file is missing on disk. */
  missing: ComputedRef<Set<string>>
  /** Selected paths while multi-select is on; empty otherwise. */
  selected: ComputedRef<Set<string>>
  /** Tags by id, shared so buttons do not each build the lookup. */
  tagsById: ComputedRef<Map<string, { id: string; name: string; color?: string }>>
  /** How many tag badges fit on a button at the current button width. */
  badgeCap: ComputedRef<number>
}

export const BOARD_STATE_KEY: InjectionKey<BoardButtonState> = Symbol('board-state')
