import { Regex } from '@companion-module/base'

export function getConfigFields() {
	return [
		{
			type: 'static-text',
			id: 'info',
			width: 12,
			label: 'Connection',
			value:
				'Enable Remote in Sound Ninja (Settings → Remote), then enter this PC IP. Copy the address from that tab or from Settings → About.',
		},
		{
			type: 'textinput',
			id: 'host',
			label: 'Target IP',
			width: 8,
			regex: Regex.IP,
			default: '127.0.0.1',
		},
		{
			type: 'number',
			id: 'port',
			label: 'Port',
			width: 4,
			min: 1,
			max: 65535,
			default: 7331,
		},
		{
			type: 'textinput',
			id: 'token',
			label: 'Token (optional)',
			width: 12,
			default: '',
		},
	]
}
