import { invoke } from '@tauri-apps/api/core'
import type { SoundFile } from '~/utils/db'

export interface LocalIp {
  name: string
  ip: string
  primary: boolean
}

export interface RemoteStatus {
  running: boolean
  port: number
  clients: number
}

export interface RemoteSound {
  id: string
  name: string
  tabs: string[]
  active: boolean
}

export async function remoteStart(port: number, token: string): Promise<RemoteStatus> {
  return invoke<RemoteStatus>('remote_start', {
    port,
    token: token.trim() || null,
  })
}

export async function remoteStop(): Promise<RemoteStatus> {
  return invoke<RemoteStatus>('remote_stop')
}

export async function remoteStatus(): Promise<RemoteStatus> {
  return invoke<RemoteStatus>('remote_status')
}

export async function remotePublishState(sounds: RemoteSound[], playing: string[]): Promise<void> {
  await invoke('remote_publish_state', { sounds, playing })
}

export async function getLocalIps(): Promise<LocalIp[]> {
  return invoke<LocalIp[]>('get_local_ips')
}

/** Push current board snapshot to the remote server / WS clients. */
export async function publishRemoteState(files: SoundFile[] | undefined | null): Promise<void> {
  const list = files || []
  const sounds: RemoteSound[] = list
    .filter((f) => f && f.id)
    .map((f) => ({
      id: f.id,
      name: f.name,
      tabs: Array.isArray(f.tabs) ? f.tabs : [],
      active: !!f.active,
    }))
  const playing = sounds.filter((s) => s.active).map((s) => s.id)
  await remotePublishState(sounds, playing)
}
