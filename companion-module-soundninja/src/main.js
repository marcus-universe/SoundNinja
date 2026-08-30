import { InstanceBase, runEntrypoint, InstanceStatus } from '@companion-module/base'
import { getConfigFields } from './config.js'
import { SoundNinjaApi } from './api.js'
import { updateActions } from './actions.js'
import { updateFeedbacks } from './feedbacks.js'
import { updateVariables } from './variables.js'
import { updatePresets } from './presets.js'
import { UpgradeScripts } from './upgrades.js'

class SoundNinjaInstance extends InstanceBase {
	async init(config) {
		this.config = config
		this.sounds = []
		this.playing = new Set()
		this.connected = false
		this.lastTriggered = ''
		this._soundSig = ''

		this.api = new SoundNinjaApi((level, msg) => this.log(level, msg))
		this.api.onState = (state) => this._applyState(state)
		this.api.onStatus = (kind, message) => this._applyStatus(kind, message)

		updateVariables(this)
		updateActions(this)
		updateFeedbacks(this)
		updatePresets(this)
		this._connect()
	}

	async destroy() {
		if (this.api) this.api.disconnect(true)
		this.connected = false
	}

	async configUpdated(config) {
		this.config = config
		this._connect()
	}

	getConfigFields() {
		return getConfigFields()
	}

	_connect() {
		this.api.configure(this.config.host, this.config.port, this.config.token)
		this.updateStatus(InstanceStatus.Connecting)
		this.api.connect()
	}

	_applyStatus(kind, message) {
		if (kind === 'ok') {
			this.connected = true
			this.updateStatus(InstanceStatus.Ok)
		} else if (kind === 'connecting') {
			this.connected = false
			this.updateStatus(InstanceStatus.Connecting)
		} else if (kind === 'bad_config') {
			this.connected = false
			this.updateStatus(InstanceStatus.BadConfig, message)
		} else {
			this.connected = false
			this.updateStatus(InstanceStatus.ConnectionFailure, message || 'Disconnected')
		}
		this.setVariableValues({ connected: this.connected ? 'true' : 'false' })
	}

	_applyState(state) {
		this.sounds = Array.isArray(state.sounds) ? state.sounds : []
		this.playing = new Set(Array.isArray(state.playing) ? state.playing : [])
		this.setVariableValues({
			playing_count: this.playing.size,
			connected: this.connected ? 'true' : 'false',
		})
		this.checkFeedbacks('sound_playing')

		const sig = this.sounds.map((s) => s.id).join(',')
		if (sig !== this._soundSig) {
			this._soundSig = sig
			updateActions(this)
			updateFeedbacks(this)
			updatePresets(this)
		}
	}
}

runEntrypoint(SoundNinjaInstance, UpgradeScripts)
