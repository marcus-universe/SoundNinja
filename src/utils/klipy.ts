import { invoke } from '@tauri-apps/api/core'

export type KlipyGif = {
  id: string
  title: string
  thumbUrl: string
  /** Small animated GIF/WebP for the search grid (loops in <img>). */
  previewUrl: string
  gifUrl: string
  /** Animated slots, small first — picker tries next if one exceeds 8 MiB. */
  downloadUrls: string[]
}

type KlipyFileSlot = {
  jpg?: { url?: string }
  gif?: { url?: string }
  webp?: { url?: string }
}

type KlipyItem = {
  id?: string | number
  slug?: string
  title?: string
  file?: Record<string, KlipyFileSlot>
}

function slotUrl(item: KlipyItem, size: string, kind: 'jpg' | 'gif' | 'webp'): string {
  const url = item.file?.[size]?.[kind]?.url
  return typeof url === 'string' ? url : ''
}

function uniqueUrls(urls: string[]): string[] {
  const out: string[] = []
  const seen = new Set<string>()
  for (const url of urls) {
    if (!url || seen.has(url)) continue
    seen.add(url)
    out.push(url)
  }
  return out
}

function parseItems(json: unknown): KlipyGif[] {
  const root = json as Record<string, unknown> | null
  const data = (root?.data ?? root) as Record<string, unknown> | unknown[] | null
  let list: unknown[] = []
  if (Array.isArray(data)) list = data
  else if (data && typeof data === 'object') {
    const inner = data as Record<string, unknown>
    if (Array.isArray(inner.data)) list = inner.data
    else if (Array.isArray(inner.items)) list = inner.items
    else if (Array.isArray(inner.results)) list = inner.results
  }
  const out: KlipyGif[] = []
  for (const raw of list) {
    if (!raw || typeof raw !== 'object') continue
    const item = raw as KlipyItem
    const thumb =
      slotUrl(item, 'sm', 'jpg') ||
      slotUrl(item, 'xs', 'jpg') ||
      slotUrl(item, 'sm', 'gif') ||
      slotUrl(item, 'md', 'gif')
    const preview =
      slotUrl(item, 'xs', 'webp') ||
      slotUrl(item, 'sm', 'webp') ||
      slotUrl(item, 'xs', 'gif') ||
      slotUrl(item, 'sm', 'gif') ||
      slotUrl(item, 'md', 'webp') ||
      slotUrl(item, 'md', 'gif') ||
      thumb
    const downloadUrls = uniqueUrls([
      slotUrl(item, 'sm', 'webp'),
      slotUrl(item, 'sm', 'gif'),
      slotUrl(item, 'md', 'webp'),
      slotUrl(item, 'md', 'gif'),
      slotUrl(item, 'hd', 'webp'),
      slotUrl(item, 'hd', 'gif'),
    ])
    const gif = downloadUrls[0] || ''
    if (!preview || !gif) continue
    out.push({
      id: String(item.id ?? item.slug ?? gif),
      title: item.title || '',
      thumbUrl: thumb || preview,
      previewUrl: preview,
      gifUrl: gif,
      downloadUrls,
    })
  }
  return out
}

function apiUrl(apiKey: string, path: string, query: Record<string, string>): string {
  const key = apiKey.trim()
  const u = new URL(`https://api.klipy.com/api/v1/${encodeURIComponent(key)}/${path}`)
  for (const [k, v] of Object.entries(query)) u.searchParams.set(k, v)
  return u.toString()
}

export type KlipyPage = {
  items: KlipyGif[]
  page: number
  hasNext: boolean
}

function parsePage(json: unknown, requestedPage: number, pageSize: number): KlipyPage {
  const items = parseItems(json)
  const root = json as Record<string, unknown> | null
  const data = (root?.data ?? root) as Record<string, unknown> | unknown[] | null
  let hasNext = false
  let page = requestedPage
  if (data && typeof data === 'object' && !Array.isArray(data)) {
    const inner = data as Record<string, unknown>
    const flag = inner.has_next ?? inner.hasNext
    if (typeof flag === 'boolean') hasNext = flag
    const p = inner.current_page ?? inner.currentPage ?? inner.page
    if (typeof p === 'number' && isFinite(p)) page = p
  }
  if (typeof (json as { has_next?: unknown } | null)?.has_next === 'boolean') {
    hasNext = !!(json as { has_next: boolean }).has_next
  }
  if (!hasNext && items.length >= pageSize) hasNext = true
  if (items.length === 0) hasNext = false
  return { items, page, hasNext }
}

async function fetchKlipy(url: string, requestedPage: number, pageSize: number): Promise<KlipyPage> {
  const text = await invoke<string>('http_get_text', { url })
  let json: unknown
  try {
    json = JSON.parse(text)
  } catch {
    throw new Error('Klipy returned invalid JSON')
  }
  return parsePage(json, requestedPage, pageSize)
}

const PAGE_SIZE = 24

export function searchKlipy(apiKey: string, q: string, page = 1): Promise<KlipyPage> {
  return fetchKlipy(
    apiUrl(apiKey, 'gifs/search', {
      q,
      per_page: String(PAGE_SIZE),
      perPage: String(PAGE_SIZE),
      page: String(page),
      rating: 'pg',
    }),
    page,
    PAGE_SIZE,
  )
}

export function trendingKlipy(apiKey: string, page = 1): Promise<KlipyPage> {
  return fetchKlipy(
    apiUrl(apiKey, 'gifs/trending', {
      per_page: String(PAGE_SIZE),
      perPage: String(PAGE_SIZE),
      page: String(page),
      rating: 'pg',
    }),
    page,
    PAGE_SIZE,
  )
}
