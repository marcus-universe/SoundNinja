import type Database from '@tauri-apps/plugin-sql'
import { loadGifBlob, type GifBlobRow } from '~/utils/db'

export type GifUrls = {
  animUrl: string
  posterUrl: string
  mime: string
}

const cache = new Map<string, GifUrls>()
const inflight = new Map<string, Promise<GifUrls | null>>()

export function peekGifUrls(id: string): GifUrls | null {
  return cache.get(id) ?? null
}

export function revokeGifUrls(id: string): void {
  const urls = cache.get(id)
  if (!urls) return
  URL.revokeObjectURL(urls.animUrl)
  if (urls.posterUrl !== urls.animUrl) URL.revokeObjectURL(urls.posterUrl)
  cache.delete(id)
}

export function revokeAllGifUrls(): void {
  for (const urls of cache.values()) {
    URL.revokeObjectURL(urls.animUrl)
    if (urls.posterUrl !== urls.animUrl) URL.revokeObjectURL(urls.posterUrl)
  }
  cache.clear()
  inflight.clear()
}

export async function ensureGifUrls(d: Database, id: string): Promise<GifUrls | null> {
  const hit = cache.get(id)
  if (hit) return hit
  const pending = inflight.get(id)
  if (pending) return pending
  const task = loadAndCache(d, id)
  inflight.set(id, task)
  try {
    return await task
  } finally {
    inflight.delete(id)
  }
}

async function loadAndCache(d: Database, id: string): Promise<GifUrls | null> {
  const row = await loadGifBlob(d, id)
  if (!row) return null
  const urls = urlsFromRow(row)
  cache.set(id, urls)
  return urls
}

export function cacheGifRow(row: GifBlobRow): GifUrls {
  const existing = cache.get(row.id)
  if (existing) return existing
  const urls = urlsFromRow(row)
  cache.set(row.id, urls)
  return urls
}

function urlsFromRow(row: GifBlobRow): GifUrls {
  const animBytes = b64ToBytes(row.data)
  const animBlob = new Blob([animBytes], { type: row.mime || 'image/gif' })
  const animUrl = URL.createObjectURL(animBlob)
  let posterUrl = animUrl
  if (row.poster) {
    const posterBytes = b64ToBytes(row.poster)
    posterUrl = URL.createObjectURL(new Blob([posterBytes], { type: 'image/png' }))
  }
  return { animUrl, posterUrl, mime: row.mime }
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
