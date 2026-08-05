import { check, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'

export type { Update, DownloadEvent }

const GITHUB_LATEST_API =
  'https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest'

export type UpdateCheckResult =
  | { status: 'none' }
  | { status: 'available'; update: Update }
  /** Newer GitHub release exists but updater `latest.json` is missing — open browser. */
  | { status: 'availableManual'; version: string; notes: string; url: string }

export async function getAppVersion(): Promise<string> {
  try {
    return await getVersion()
  } catch {
    return '0.0.0'
  }
}

/** Parse `1.2.3` / `v1.2.3` into numeric parts. */
export function parseSemver(raw: string): number[] {
  const cleaned = raw.trim().replace(/^v/i, '').split(/[+-]/)[0] ?? '0'
  return cleaned.split('.').map((p) => {
    const n = parseInt(p, 10)
    return Number.isFinite(n) ? n : 0
  })
}

/** True when `remote` is strictly newer than `current`. */
export function isNewerVersion(remote: string, current: string): boolean {
  const a = parseSemver(remote)
  const b = parseSemver(current)
  const len = Math.max(a.length, b.length)
  for (let i = 0; i < len; i++) {
    const av = a[i] ?? 0
    const bv = b[i] ?? 0
    if (av > bv) return true
    if (av < bv) return false
  }
  return false
}

function isMissingReleaseJsonError(e: unknown): boolean {
  const msg = e instanceof Error ? e.message : String(e)
  return /release JSON|valid release|404|Not Found|failed to fetch|error sending request/i.test(msg)
}

async function fetchGithubLatest(): Promise<{ version: string; notes: string; url: string } | null> {
  try {
    const res = await fetch(GITHUB_LATEST_API, {
      headers: { Accept: 'application/vnd.github+json' },
    })
    if (!res.ok) return null
    const data = (await res.json()) as {
      tag_name?: string
      body?: string | null
      html_url?: string
      draft?: boolean
      prerelease?: boolean
    }
    if (data.draft || data.prerelease) return null
    const version = (data.tag_name || '').trim()
    if (!version) return null
    return {
      version,
      notes: (data.body || '').trim(),
      url: data.html_url || 'https://github.com/marcus-universe/SoundNinja/releases/latest',
    }
  } catch {
    return null
  }
}

/**
 * Prefer signed Tauri updater feed. If `latest.json` is missing (common until
 * the first signed release), fall back to the GitHub Releases API for version compare.
 */
export async function checkForAppUpdate(): Promise<UpdateCheckResult> {
  try {
    const update = await check()
    if (!update) return { status: 'none' }
    return { status: 'available', update }
  } catch (e) {
    if (!isMissingReleaseJsonError(e)) throw e

    const current = await getAppVersion()
    const remote = await fetchGithubLatest()
    if (!remote || !isNewerVersion(remote.version, current)) {
      return { status: 'none' }
    }
    return {
      status: 'availableManual',
      version: remote.version.replace(/^v/i, ''),
      notes: remote.notes,
      url: remote.url,
    }
  }
}

export async function installAppUpdate(
  update: Update,
  onEvent?: (event: DownloadEvent) => void,
): Promise<void> {
  await update.downloadAndInstall(onEvent)
  await relaunch()
}

export async function openReleasePage(url: string): Promise<void> {
  await invoke('open_external_url', { url })
}
