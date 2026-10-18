import { invoke } from '@tauri-apps/api/core'

export interface WebuiStatus {
  running: boolean
  port: number
  clients: number
  token: string
}

export async function webuiStart(port: number, token: string): Promise<WebuiStatus> {
  return invoke<WebuiStatus>('webui_start', {
    port,
    token: token.trim() || null,
  })
}

export async function webuiStop(): Promise<WebuiStatus> {
  return invoke<WebuiStatus>('webui_stop')
}

export async function webuiStatus(): Promise<WebuiStatus> {
  return invoke<WebuiStatus>('webui_status')
}

export async function webuiQrSvg(url: string): Promise<string> {
  return invoke<string>('webui_qr_svg', { url })
}

const TOKEN_ALPHABET = 'abcdefghjkmnpqrstuvwxyz23456789'

export function makeWebToken(): string {
  const buf = new Uint8Array(8)
  crypto.getRandomValues(buf)
  return Array.from(buf, (b) => TOKEN_ALPHABET[b % TOKEN_ALPHABET.length]).join('')
}
