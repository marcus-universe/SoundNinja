// Runtime application of the per-project flat theme color model.
// Prefer importing applyThemeTokens from ~/utils/themeTokens directly.
import { applyThemeTokens } from '~/utils/themeTokens'

/** @deprecated Use applyThemeTokens — kept as alias for call-site migration. */
export function applyThemeColors(settings: Record<string, unknown> | undefined | null): void {
  applyThemeTokens(settings)
}
