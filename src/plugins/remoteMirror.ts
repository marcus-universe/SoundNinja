import { isApplyingRemote } from '~/utils/webuiHost'
import { isRemote } from '~/utils/runtime'

const JSON_ACTIONS = new Set([
  'renameSound', 'removeSound', 'setSoundVolume', 'setSoundLoop', 'setSoundLoopByPath',
  'setSoundColor', 'setSoundGif', 'setSoundTabs', 'setSoundColorMany', 'setSoundTabsMany',
  'removeSoundsMany', 'reorderSounds', 'addTab', 'removeTab', 'renameTab', 'setTabColor',
  'reorderTabs', 'addSeparator', 'removeSeparator', 'setSeparatorPosition', 'updateSeparator',
  'setTabButtonAlign', 'applyBoardLayout', 'setTheme', 'setCustomCss', 'setOutSource',
  'setStopOnRetrigger', 'setOverlapSounds', 'setCacheConfig', 'setOutputVolume', 'setOutputHost',
  'setAsioChannels', 'setUniformButtonHeight', 'setAllowReorder', 'setGifPlayOnHover',
  'setPreloadGifs', 'setSetting', 'setThemeColors', 'addTag', 'removeTag', 'updateTag',
  'setSoundTags', 'toggleSoundTag', 'persistSelectedTagIds', 'setSortMode', 'setActiveSound',
  'ReturnStatusAll', 'undo', 'redo',
])

const SETTINGS_ACTIONS = new Set([
  'setNavbarSide', 'setNavbarTooltips', 'setShowTagBadges', 'setGifTitleChip', 'setBasicButtonColors',
  'setOutputSource', 'setOutputHost', 'setOutputVolume', 'setInputSource', 'setInputHost', 'setInputVolume',
  'setAsioOutputChannels', 'setAsioInputChannels', 'setAsioChannels', 'setHotkeys', 'setAppHotkey',
  'setSoundTriggersGlobal', 'setTitlebarMode', 'setHideTitlebar', 'setCheckUpdatesOnStart',
])

function plainArgs(args: unknown[]) {
  try { return JSON.parse(JSON.stringify(args)) } catch { return [] }
}

export default defineNuxtPlugin(() => {
  if (!isRemote) return
  const pinia = usePinia()
  pinia.use(({ store }) => {
    if (store.$id === 'jsonHandeling') {
      store.persistNow = async () => { store.dirty = false }
    }
    const allow = store.$id === 'jsonHandeling'
      ? JSON_ACTIONS
      : store.$id === 'appSettings'
        ? SETTINGS_ACTIONS
        : null
    if (!allow) return
    store.$onAction(({ name, args }) => {
      if (isApplyingRemote() || !allow.has(name)) return
      const send = (window as Window & { __SN_WEBUI_SEND__?: (msg: unknown) => void }).__SN_WEBUI_SEND__
      send?.({ type: 'action', store: store.$id, name, args: plainArgs(args) })
    })
  })
})
