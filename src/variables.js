export function updateVariables(self) {
	self.setVariableDefinitions({
		connected: { name: 'Connected' },
		playing_count: { name: 'Playing count' },
		last_triggered: { name: 'Last triggered sound ID' },
	})
	self.setVariableValues({
		connected: self.connected ? 'true' : 'false',
		playing_count: self.playing ? self.playing.size : 0,
		last_triggered: self.lastTriggered || '',
	})
}
