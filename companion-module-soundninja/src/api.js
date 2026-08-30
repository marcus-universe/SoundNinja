const BACKOFF_START = 1000
const BACKOFF_MAX = 15000

export class SoundNinjaApi {
	constructor(log) {
		this.log = log
		this.ws = null
		this.destroyed = false
		this.backoff = BACKOFF_START
		this.reconnectTimer = null
		this.onState = null
		this.onStatus = null
		this.host = ''
		this.port = 7331
		this.token = ''
	}

	configure(host, port, token) {
		this.host = (host || '').trim()
		this.port = Number(port) || 7331
		this.token = (token || '').trim()
	}

	baseHttp() {
		return `http://${this.host}:${this.port}`
	}

	qs() {
		return this.token ? `?token=${encodeURIComponent(this.token)}` : ''
	}

	headers(json = false) {
		const h = {}
		if (json) h['Content-Type'] = 'application/json'
		if (this.token) h.Authorization = `Bearer ${this.token}`
		return h
	}

	async rest(method, path, body) {
		const url = `${this.baseHttp()}/api/v1${path}${this.qs()}`
		const res = await fetch(url, {
			method,
			headers: this.headers(body != null),
			body: body != null ? JSON.stringify(body) : undefined,
		})
		if (!res.ok) {
			throw new Error(`${method} ${path} → ${res.status}`)
		}
		const text = await res.text()
		return text ? JSON.parse(text) : null
	}

	async probe() {
		return this.rest('GET', '/info')
	}

	async trigger(id) {
		if (this.ws && this.ws.readyState === 1) {
			this.ws.send(JSON.stringify({ cmd: 'trigger', id }))
			return
		}
		await this.rest('POST', '/trigger', { id })
	}

	async stop(id) {
		const payload = id ? { id } : {}
		if (this.ws && this.ws.readyState === 1) {
			this.ws.send(JSON.stringify({ cmd: 'stop', id: id || undefined }))
			return
		}
		await this.rest('POST', '/stop', payload)
	}

	connect() {
		this.disconnect(false)
		this.destroyed = false
		if (!this.host) {
			this._setStatus('bad_config', 'Missing target IP')
			return
		}
		this._setStatus('connecting')
		this._open()
	}

	disconnect(destroy = true) {
		if (destroy) this.destroyed = true
		if (this.reconnectTimer) {
			clearTimeout(this.reconnectTimer)
			this.reconnectTimer = null
		}
		if (this.ws) {
			try {
				this.ws.onopen = null
				this.ws.onclose = null
				this.ws.onerror = null
				this.ws.onmessage = null
				this.ws.close()
			} catch {
				/* ignore */
			}
			this.ws = null
		}
	}

	_open() {
		if (this.destroyed) return
		const url = `ws://${this.host}:${this.port}/api/v1/ws${this.qs()}`
		let socket
		try {
			socket = new WebSocket(url)
		} catch (e) {
			this.log('error', `WebSocket create failed: ${e}`)
			this._scheduleReconnect()
			return
		}
		this.ws = socket
		socket.onopen = () => {
			this.backoff = BACKOFF_START
			this._setStatus('ok')
		}
		socket.onmessage = (ev) => {
			try {
				const data = typeof ev.data === 'string' ? JSON.parse(ev.data) : null
				if (data && data.type === 'state' && typeof this.onState === 'function') {
					this.onState(data)
				}
			} catch (e) {
				this.log('debug', `Bad WS frame: ${e}`)
			}
		}
		socket.onerror = () => {
			/* onclose handles reconnect */
		}
		socket.onclose = () => {
			if (this.ws === socket) this.ws = null
			if (this.destroyed) return
			this._setStatus('error', 'Disconnected')
			this._scheduleReconnect()
		}
	}

	_scheduleReconnect() {
		if (this.destroyed || this.reconnectTimer) return
		const wait = this.backoff
		this.backoff = Math.min(this.backoff * 2, BACKOFF_MAX)
		this.reconnectTimer = setTimeout(() => {
			this.reconnectTimer = null
			this._open()
		}, wait)
	}

	_setStatus(kind, message) {
		if (typeof this.onStatus === 'function') this.onStatus(kind, message)
	}
}
