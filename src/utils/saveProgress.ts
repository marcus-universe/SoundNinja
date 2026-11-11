/** Steps of a project save, in the order the backend reports them. */
export const SAVE_PHASES = ['preparing', 'backup', 'board', 'audio', 'images', 'finalizing'] as const
export type SavePhase = (typeof SAVE_PHASES)[number]

export interface SaveProgress {
  phase: SavePhase
  /** Whole save, 0–100. Never goes backwards within one save. */
  percent: number
}

/** Payload of the backend's `project_save_progress` event. */
export interface SaveProgressEvent {
  path?: string
  phase?: string
  done?: number
  total?: number
  percent?: number
}

export function initialSaveProgress(): SaveProgress {
  return { phase: 'preparing', percent: 0 }
}

function isPhase(v: unknown): v is SavePhase {
  return typeof v === 'string' && (SAVE_PHASES as readonly string[]).includes(v)
}

/**
 * Fold one backend event into the shown progress. Events can arrive late or
 * out of order, so the bar only ever moves forward and an unknown phase keeps
 * the current label.
 */
export function applySaveProgress(current: SaveProgress, event: SaveProgressEvent | null | undefined): SaveProgress {
  if (!event) return current
  const raw = Number(event.percent)
  const percent = Number.isFinite(raw) ? Math.min(100, Math.max(0, Math.round(raw))) : current.percent
  const phase = isPhase(event.phase) ? event.phase : current.phase
  const later = SAVE_PHASES.indexOf(phase) >= SAVE_PHASES.indexOf(current.phase)
  return {
    phase: later ? phase : current.phase,
    percent: Math.max(current.percent, percent),
  }
}
