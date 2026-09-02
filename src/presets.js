import { combineRgb } from '@companion-module/base'

export function updatePresets(self) {
	const presets = {
		stop_all: {
			type: 'simple',
			name: 'Stop All',
			style: {
				text: 'Stop All',
				size: '14',
				color: combineRgb(255, 255, 255),
				bgcolor: combineRgb(140, 20, 20),
			},
			steps: [
				{
					down: [{ actionId: 'stop_all', options: {} }],
					up: [],
				},
			],
			feedbacks: [],
		},
	}

	const soundIds = []
	for (const sound of self.sounds || []) {
		if (!sound.id) continue
		const id = `sound_${sound.id}`
		soundIds.push(id)
		presets[id] = {
			type: 'simple',
			name: sound.name || sound.id,
			style: {
				text: sound.name || sound.id,
				size: '14',
				color: combineRgb(255, 255, 255),
				bgcolor: combineRgb(30, 30, 30),
			},
			steps: [
				{
					down: [
						{
							actionId: 'trigger_sound',
							options: { useCustom: false, soundId: sound.id, customId: '' },
						},
					],
					up: [],
				},
			],
			feedbacks: [
				{
					feedbackId: 'sound_playing',
					options: { soundId: sound.id },
					style: {
						bgcolor: combineRgb(0, 160, 80),
						color: combineRgb(255, 255, 255),
					},
				},
			],
		}
	}

	const structure = [
		{
			id: 'control',
			name: 'Control',
			definitions: ['stop_all'],
		},
	]
	if (soundIds.length) {
		structure.push({
			id: 'sounds',
			name: 'Sounds',
			definitions: soundIds,
		})
	}

	self.setPresetDefinitions(structure, presets)
}
