import { combineRgb } from '@companion-module/base'
import { soundChoices } from './actions.js'

export function updateFeedbacks(self) {
	const choices = soundChoices(self)
	self.setFeedbackDefinitions({
		sound_playing: {
			type: 'boolean',
			name: 'Sound Playing',
			description: 'True while the selected sound is playing.',
			defaultStyle: {
				bgcolor: combineRgb(0, 160, 80),
				color: combineRgb(255, 255, 255),
			},
			options: [
				{
					type: 'dropdown',
					id: 'soundId',
					label: 'Sound',
					default: choices[0]?.id || '',
					choices,
				},
			],
			callback: (feedback) => {
				const id = feedback.options.soundId
				return !!(id && self.playing && self.playing.has(id))
			},
		},
	})
}
