const RELEASES_API =
  'https://api.github.com/repos/marcus-universe/SoundNinja/releases?per_page=30'
const FALLBACK_URL =
  'https://github.com/marcus-universe/SoundNinja/releases/latest'
const RELEASES_PAGE_URL =
  'https://github.com/marcus-universe/SoundNinja/releases'

export interface ReleaseAsset {
  name: string
  browser_download_url: string
  size: number
}

export interface GithubRelease {
  tag_name: string
  name: string | null
  html_url: string
  published_at: string | null
  draft: boolean
  prerelease: boolean
  assets: ReleaseAsset[]
}

export type Platform = 'windows' | 'mac' | 'linux' | 'unknown'
export type AssetKind = 'exe' | 'msi' | 'dmg' | 'appimage' | 'deb' | 'other'

export interface ClassifiedAsset {
  name: string
  url: string
  kind: AssetKind
  platform: Platform
  size: number
}

export interface ReleaseView {
  tag: string
  name: string
  htmlUrl: string
  publishedAt: string | null
  prerelease: boolean
  assets: ClassifiedAsset[]
  byPlatform: Record<'windows' | 'mac' | 'linux', ClassifiedAsset[]>
}

function classifyAsset(asset: ReleaseAsset): ClassifiedAsset {
  const lower = asset.name.toLowerCase()
  let kind: AssetKind = 'other'
  let platform: Platform = 'unknown'

  if (lower.endsWith('.exe')) {
    kind = 'exe'
    platform = 'windows'
  } else if (lower.endsWith('.msi')) {
    kind = 'msi'
    platform = 'windows'
  } else if (lower.endsWith('.dmg')) {
    kind = 'dmg'
    platform = 'mac'
  } else if (lower.endsWith('.appimage')) {
    kind = 'appimage'
    platform = 'linux'
  } else if (lower.endsWith('.deb')) {
    kind = 'deb'
    platform = 'linux'
  }

  return {
    name: asset.name,
    url: asset.browser_download_url,
    kind,
    platform,
    size: asset.size,
  }
}

function toReleaseView(release: GithubRelease): ReleaseView {
  const assets = (release.assets ?? []).map(classifyAsset)
  const byPlatform: ReleaseView['byPlatform'] = {
    windows: assets.filter((a) => a.platform === 'windows'),
    mac: assets.filter((a) => a.platform === 'mac'),
    linux: assets.filter((a) => a.platform === 'linux'),
  }
  return {
    tag: release.tag_name,
    name: release.name || release.tag_name,
    htmlUrl: release.html_url,
    publishedAt: release.published_at,
    prerelease: release.prerelease,
    assets,
    byPlatform,
  }
}

function detectPlatform(): Platform {
  if (import.meta.server) return 'unknown'
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('win')) return 'windows'
  if (ua.includes('mac')) return 'mac'
  if (ua.includes('linux') || ua.includes('x11')) return 'linux'
  return 'unknown'
}

function preferredAssetUrl(release: ReleaseView, platform: Platform): string {
  const list =
    platform === 'windows' || platform === 'mac' || platform === 'linux'
      ? release.byPlatform[platform]
      : []
  if (platform === 'windows') {
    return (
      list.find((a) => a.kind === 'exe')?.url ??
      list.find((a) => a.kind === 'msi')?.url ??
      release.htmlUrl
    )
  }
  if (platform === 'mac') {
    return list.find((a) => a.kind === 'dmg')?.url ?? release.htmlUrl
  }
  if (platform === 'linux') {
    return (
      list.find((a) => a.kind === 'appimage')?.url ??
      list.find((a) => a.kind === 'deb')?.url ??
      release.htmlUrl
    )
  }
  return release.htmlUrl
}

export function useReleases() {
  const latest = useState<ReleaseView | null>('sn-release-latest', () => null)
  const older = useState<ReleaseView[]>('sn-release-older', () => [])
  const version = useState<string | null>('sn-release-version', () => null)
  const platformHref = useState('sn-release-href', () => FALLBACK_URL)
  const loading = useState('sn-release-loading', () => true)
  const error = useState('sn-release-error', () => false)
  const fetched = useState('sn-release-fetched', () => false)

  onMounted(async () => {
    if (fetched.value) return
    fetched.value = true
    loading.value = true
    error.value = false

    try {
      const res = await fetch(RELEASES_API, {
        headers: {
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
        },
        cache: 'no-store',
      })
      if (!res.ok) throw new Error(`GitHub API ${res.status}`)

      const data = (await res.json()) as GithubRelease[]
      const published = data.filter((r) => !r.draft)
      const stable = published.filter((r) => !r.prerelease)
      const pool = stable.length ? stable : published

      if (!pool.length) throw new Error('No releases')

      const [first, ...rest] = pool.map(toReleaseView)
      latest.value = first
      older.value = rest
      version.value = first.tag
      platformHref.value = preferredAssetUrl(first, detectPlatform())
    } catch {
      error.value = true
      platformHref.value = FALLBACK_URL
    } finally {
      loading.value = false
    }
  })

  return {
    latest,
    older,
    version,
    platformHref,
    loading,
    error,
    fallbackUrl: FALLBACK_URL,
    releasesPageUrl: RELEASES_PAGE_URL,
  }
}

/** Shared release helpers also expose a thin latest-only API. */
export function useLatestRelease() {
  const { version, platformHref, loading, error, fallbackUrl } = useReleases()
  return {
    version,
    href: platformHref,
    loading,
    error,
    fallbackUrl,
  }
}
