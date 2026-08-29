import { invoke } from '@tauri-apps/api/core'

export type KlipyGif = {
  id: string
  title: string
  thumbUrl: string
  gifUrl: string
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
    const gif =
      slotUrl(item, 'md', 'gif') ||
      slotUrl(item, 'sm', 'gif') ||
      slotUrl(item, 'hd', 'gif') ||
      slotUrl(item, 'md', 'webp')
    if (!thumb || !gif) continue
    out.push({
      id: String(item.id ?? item.slug ?? gif),
      title: item.title || '',
      thumbUrl: thumb,
      gifUrl: gif,
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

async function fetchKlipy(url: string): Promise<KlipyGif[]> {
  const text = await invoke<string>('http_get_text', { url })
  let json: unknown
  try {
    json = JSON.parse(text)
  } catch {
    throw new Error('Klipy returned invalid JSON')
  }
  return parseItems(json)
}

export function searchKlipy(apiKey: string, q: string, page = 1): Promise<KlipyGif[]> {
  return fetchKlipy(
    apiUrl(apiKey, 'gifs/search', {
      q,
      per_page: '24',
      perPage: '24',
      page: String(page),
      rating: 'pg',
    }),
  )
}

export function trendingKlipy(apiKey: string, page = 1): Promise<KlipyGif[]> {
  return fetchKlipy(
    apiUrl(apiKey, 'gifs/trending', {
      per_page: '24',
      perPage: '24',
      page: String(page),
      rating: 'pg',
    }),
  )
}
