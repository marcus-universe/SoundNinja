export function updateActions(self) {
	const choices = soundChoices(self)

	self.setActionDefinitions({
		trigger_sound: {
			name: 'Trigger Sound',
			options: [
				{
					type: 'checkbox',
					id: 'useCustom',
					label: 'Use custom ID',
					default: false,
				},
				{
					type: 'dropdown',
					id: 'soundId',
					label: 'Sound',
					default: choices[0]?.id || '',
					choices,
					isVisibleExpression: '!$(options:useCustom)',
				},
				{
					type: 'textinput',
					id: 'customId',
					label: 'Sound ID',
					default: '',
					useVariables: true,
					tooltip: '8-character sound ID. Companion variables allowed.',
					isVisibleExpression: '!!$(options:useCustom)',
				},
			],
			callback: async (action) => {
				let id = action.options.soundId
				if (action.options.useCustom) {
					id = action.options.customId || ''
				}
				id = (id || '').trim()
				if (!id) return
				await self.api.trigger(id)
				self.lastTriggered = id
				self.setVariableValues({ last_triggered: id })
			},
		},
		stop_sound: {
			name: 'Stop Sound',
			options: [
				{
					type: 'checkbox',
					id: 'useCustom',
					label: 'Use custom ID',
					default: false,
				},
				{
					type: 'dropdown',
					id: 'soundId',
					label: 'Sound',
					default: choices[0]?.id || '',
					choices,
					isVisibleExpression: '!$(options:useCustom)',
				},
				{
					type: 'textinput',
					id: 'customId',
					label: 'Sound ID',
					default: '',
					useVariables: true,
					isVisibleExpression: '!!$(options:useCustom)',
				},
			],
			callback: async (action) => {
				let id = action.options.soundId
				if (action.options.useCustom) {
					id = action.options.customId || ''
				}
				id = (id || '').trim()
				if (!id) return
				await self.api.stop(id)
			},
		},
		stop_all: {
			name: 'Stop All',
			options: [],
			callback: async () => {
				await self.api.stop()
			},
		},
	})
}

export function soundChoices(self) {
	const sounds = self.sounds || []
	if (!sounds.length) {
		return [{ id: '', label: 'No sounds (connect to Sound Ninja)' }]
	}
	return sounds.map((s) => ({
		id: s.id,
		label: `${s.name} (${s.id})`,
	}))
}
