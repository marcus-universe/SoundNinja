import { invoke } from '@tauri-apps/api/core'
import type { KlipyGif, KlipyPage } from '~/utils/klipy'

const PAGE_SIZE = 24
const TTL_MS = 24 * 60 * 60 * 1000
const MAX_ENTRIES = 64

type CacheEntry = { at: number; page: KlipyPage }

const memory = new Map<string, CacheEntry>()
const inflight = new Map<string, Promise<KlipyPage>>()
let persistCache: ((blob: string) => Promise<void>) | null = null
let persistTimer: ReturnType<typeof setTimeout> | null = null

/** Hydrate disk cache and attach a saver. Call once from app settings load. */
export function attachGifsnapCache(blob: string, save: (next: string) => Promise<void>) {
  persistCache = save
  hydrate(blob)
}

function hydrate(blob: string) {
  if (!blob) return
  try {
    const parsed = JSON.parse(blob) as Record<string, CacheEntry>
    const now = Date.now()
    const entries = Object.entries(parsed)
      .filter(([, v]) => v && typeof v.at === 'number' && v.page && Array.isArray(v.page.items))
      .filter(([, v]) => now - v.at < TTL_MS)
      .sort((a, b) => a[1].at - b[1].at)
      .slice(-MAX_ENTRIES)
    memory.clear()
    for (const [k, v] of entries) memory.set(k, v)
  } catch {
    /* ignore corrupt cache */
  }
}

function serialize(): string {
  const out: Record<string, CacheEntry> = {}
  for (const [k, v] of memory) out[k] = v
  return JSON.stringify(out)
}

function schedulePersist() {
  if (!persistCache) return
  if (persistTimer) clearTimeout(persistTimer)
  persistTimer = setTimeout(() => {
    persistTimer = null
    persistCache?.(serialize()).catch(() => {})
  }, 250)
}

/** Wipe in-memory + disk GifSnap search/trending JSON. Consent is unchanged. */
export async function clearGifsnapCache(): Promise<void> {
  memory.clear()
  inflight.clear()
  if (persistTimer) {
    clearTimeout(persistTimer)
    persistTimer = null
  }
  if (persistCache) await persistCache('{}')
}

function touch(key: string, entry: CacheEntry) {
  memory.delete(key)
  memory.set(key, entry)
  while (memory.size > MAX_ENTRIES) {
    const oldest = memory.keys().next().value
    if (oldest == null) break
    memory.delete(oldest)
  }
  schedulePersist()
}

function normalizeQuery(q: string): string {
  return q.trim().toLowerCase().replace(/\s+/g, ' ')
}

function cacheKey(kind: 'search' | 'trending', q: string, page: number): string {
  if (kind === 'trending') return `trending:${page}`
  return `search:${normalizeQuery(q)}:${page}`
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
  const data = root?.data
  const list = Array.isArray(data) ? data : []
  const out: KlipyGif[] = []
  for (const raw of list) {
    if (!raw || typeof raw !== 'object') continue
    const item = raw as Record<string, unknown>
    const preview = typeof item.preview_url === 'string' ? item.preview_url : ''
    const gif = typeof item.url === 'string' ? item.url : ''
    if (!preview && !gif) continue
    const previewUrl = preview || gif
    const gifUrl = gif || preview
    out.push({
      id: String(item.id ?? gifUrl),
      title: typeof item.title === 'string' ? item.title : '',
      thumbUrl: previewUrl,
      previewUrl,
      gifUrl,
      downloadUrls: uniqueUrls([gifUrl, previewUrl]),
    })
  }
  return out
}

function parsePage(json: unknown, requestedPage: number, pageSize: number): KlipyPage {
  const items = parseItems(json)
  const root = json as Record<string, unknown> | null
  const pagination = root?.pagination as Record<string, unknown> | undefined
  let hasNext = false
  let page = requestedPage
  if (pagination && typeof pagination === 'object') {
    if (typeof pagination.has_next === 'boolean') hasNext = pagination.has_next
    if (typeof pagination.page === 'number' && Number.isFinite(pagination.page)) page = pagination.page
  }
  if (!hasNext && items.length >= pageSize) hasNext = true
  if (items.length === 0) hasNext = false
  return { items, page, hasNext }
}

function apiUrl(path: string, query: Record<string, string>): string {
  const u = new URL(`https://gifsnap.com/api/v1/${path}`)
  for (const [k, v] of Object.entries(query)) u.searchParams.set(k, v)
  return u.toString()
}

async function fetchPage(url: string, key: string, requestedPage: number): Promise<KlipyPage> {
  const hit = memory.get(key)
  if (hit && Date.now() - hit.at < TTL_MS) {
    touch(key, hit)
    return hit.page
  }
  const pending = inflight.get(key)
  if (pending) return pending
  const task = (async () => {
    const text = await invoke<string>('http_get_text', { url })
    let json: unknown
    try {
      json = JSON.parse(text)
    } catch {
      throw new Error('GifSnap returned invalid JSON')
    }
    const page = parsePage(json, requestedPage, PAGE_SIZE)
    touch(key, { at: Date.now(), page })
    return page
  })()
  inflight.set(key, task)
  try {
    return await task
  } finally {
    inflight.delete(key)
  }
}

export function searchGifsnap(q: string, page = 1): Promise<KlipyPage> {
  const query = normalizeQuery(q)
  return fetchPage(
    apiUrl('gifs/search', { q: query, page: String(page), limit: String(PAGE_SIZE) }),
    cacheKey('search', query, page),
    page,
  )
}

export function trendingGifsnap(page = 1): Promise<KlipyPage> {
  return fetchPage(
    apiUrl('gifs/trending', { page: String(page), limit: String(PAGE_SIZE) }),
    cacheKey('trending', '', page),
    page,
  )
}
