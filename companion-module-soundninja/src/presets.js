import { combineRgb } from '@companion-module/base'

export function updatePresets(self) {
	const presets = {
		stop_all: {
			type: 'button',
			category: 'Control',
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

	for (const sound of self.sounds || []) {
		if (!sound.id) continue
		presets[`sound_${sound.id}`] = {
			type: 'button',
			category: 'Sounds',
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

	self.setPresetDefinitions(presets)
}
