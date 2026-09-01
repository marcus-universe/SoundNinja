import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { GifBlobRow } from '~/utils/db'

export type GifUrls = {
  animUrl: string
  posterUrl: string
  mime: string
}

/** What the Rust side hands back: real files on disk, not bytes. */
type GifCacheEntry = {
  id: string
  mime: string
  path: string
  posterPath: string
}

const cache = new Map<string, GifUrls>()
const inflight = new Map<string, Promise<void>>()

function entryToUrls(entry: GifCacheEntry): GifUrls {
  return {
    animUrl: convertFileSrc(entry.path),
    posterUrl: convertFileSrc(entry.posterPath),
    mime: entry.mime,
  }
}

export function peekGifUrls(id: string): GifUrls | null {
  return cache.get(id) ?? null
}

/**
 * Forget an id. The bytes live in files owned by Rust and in the project
 * database, so nothing needs releasing here — dropping the entry only frees the
 * webview's decoded-image memory once the `<img>` stops referencing it.
 */
export function revokeGifUrls(id: string): void {
  cache.delete(id)
}

export function revokeAllGifUrls(): void {
  cache.clear()
  inflight.clear()
}

/**
 * Resolve ids to file URLs, extracting anything still missing from the project
 * database on the Rust side. Blobs never cross the IPC boundary.
 */
export async function ensureGifUrls(projectPath: string, ids: string[]): Promise<void> {
  const wanted = [...new Set(ids.filter(Boolean))].filter((id) => !cache.has(id))
  if (!wanted.length || !projectPath) return

  const pending = wanted.filter((id) => inflight.has(id))
  const fresh = wanted.filter((id) => !inflight.has(id))

  if (fresh.length) {
    const task = invoke<GifCacheEntry[]>('gif_cache_paths', { projectDb: projectPath, ids: fresh })
      .then((entries) => {
        for (const entry of entries) cache.set(entry.id, entryToUrls(entry))
      })
      .catch((e) => {
        console.error('Failed to resolve GIF cache paths', e)
      })
      .finally(() => {
        for (const id of fresh) inflight.delete(id)
      })
    for (const id of fresh) inflight.set(id, task)
  }

  await Promise.all([...pending, ...fresh].map((id) => inflight.get(id)).filter(Boolean))
}

/**
 * Seed the cache for a freshly stored image so the button updates immediately
 * instead of waiting for the next database read.
 */
export async function cacheGifRow(row: GifBlobRow): Promise<GifUrls | null> {
  const existing = cache.get(row.id)
  if (existing) return existing
  try {
    const entry = await invoke<GifCacheEntry>('gif_cache_put', {
      id: row.id,
      mime: row.mime,
      data: row.data,
      poster: row.poster,
    })
    const urls = entryToUrls(entry)
    cache.set(row.id, urls)
    return urls
  } catch (e) {
    console.error('Failed to seed GIF cache', e)
    return null
  }
}

export function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64)
  const out = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i)
  return out
}

export function bytesToB64(bytes: Uint8Array): string {
  const chunk = 0x2000
  let s = ''
  for (let i = 0; i < bytes.length; i += chunk) {
    s += String.fromCharCode(...bytes.subarray(i, i + chunk))
  }
  return btoa(s)
}

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', bytes)
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, '0')).join('')
}

export function detectImageMime(bytes: Uint8Array): string | null {
  if (bytes.length >= 6 && bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46) {
    return 'image/gif'
  }
  if (
    bytes.length >= 12 &&
    bytes[0] === 0x52 && bytes[1] === 0x49 && bytes[2] === 0x46 && bytes[3] === 0x46 &&
    bytes[8] === 0x57 && bytes[9] === 0x45 && bytes[10] === 0x42 && bytes[11] === 0x50
  ) {
    return 'image/webp'
  }
  if (
    bytes.length >= 8 &&
    bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4e && bytes[3] === 0x47
  ) {
    return 'image/png'
  }
  if (bytes.length >= 3 && bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff) {
    return 'image/jpeg'
  }
  return null
}

export const LOCAL_IMAGE_MIMES = ['image/gif', 'image/webp', 'image/png', 'image/jpeg'] as const

export function isLocalImageMime(mime: string): boolean {
  return (LOCAL_IMAGE_MIMES as readonly string[]).includes(mime)
}

export function isAnimatedImageMime(mime: string): boolean {
  return mime === 'image/gif' || mime === 'image/webp'
}

/** First-frame PNG from an animated image. Used so hover-off does not keep a GIF decoder alive. */
export function extractPosterPng(bytes: Uint8Array, mime: string): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const blob = new Blob([bytes], { type: mime })
    const url = URL.createObjectURL(blob)
    const img = new Image()
    img.onload = () => {
      try {
        const canvas = document.createElement('canvas')
        canvas.width = Math.max(1, img.naturalWidth)
        canvas.height = Math.max(1, img.naturalHeight)
        const ctx = canvas.getContext('2d')
        if (!ctx) {
          URL.revokeObjectURL(url)
          reject(new Error('canvas'))
          return
        }
        ctx.drawImage(img, 0, 0)
        canvas.toBlob((png) => {
          URL.revokeObjectURL(url)
          if (!png) {
            reject(new Error('poster'))
            return
          }
          png.arrayBuffer().then((ab) => resolve(new Uint8Array(ab)), reject)
        }, 'image/png')
      } catch (e) {
        URL.revokeObjectURL(url)
        reject(e)
      }
    }
    img.onerror = () => {
      URL.revokeObjectURL(url)
      reject(new Error('decode'))
    }
    img.src = url
  })
}
