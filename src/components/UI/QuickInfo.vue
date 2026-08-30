<template>
  <span
    ref="rootRef"
    class="qtip"
    :class="{
      'qtip--below': below && !side,
      'qtip--left': side === 'left',
      'qtip--right': side === 'right',
    }"
    @mouseenter="onEnter"
    @mouseleave="onLeave"
  >
    <slot />
    <Teleport v-if="fixed" to="body">
      <span
        ref="tipRef"
        class="qtip__tip qtip__tip--fixed"
        :class="{
          'is-open': open,
          'qtip__tip--below': below && !side,
          'qtip__tip--left': side === 'left',
          'qtip__tip--right': side === 'right',
        }"
        :style="fixedStyle"
      >{{ text }}</span>
    </Teleport>
    <span v-else class="qtip__tip">{{ text }}</span>
  </span>
</template>

<script setup lang="ts">
const props = defineProps<{
  text: string
  /** Show tip below the trigger instead of above. */
  below?: boolean
  /** Show tip to the left or right (sidebar). Overrides above/below. */
  side?: 'left' | 'right'
  /** Render tip on document.body with fixed coords — escapes overflow clip. */
  fixed?: boolean
}>()

const rootRef = ref<HTMLElement | null>(null)
const tipRef = ref<HTMLElement | null>(null)
const open = ref(false)
const fixedStyle = ref<Record<string, string>>({})

const VIEW_PAD = 8
const GAP = 8

function clamp(n: number, min: number, max: number) {
  return Math.min(max, Math.max(min, n))
}

function place() {
  const el = rootRef.value
  if (!el) return
  const r = el.getBoundingClientRect()
  const style: Record<string, string> = { transform: 'none' }

  if (props.side === 'left') {
    style.top = `${r.top + r.height / 2}px`
    style.right = `${window.innerWidth - r.left + GAP}px`
    style.left = 'auto'
    style.transform = 'translateY(-50%)'
    fixedStyle.value = style
    return
  }
  if (props.side === 'right') {
    style.top = `${r.top + r.height / 2}px`
    style.left = `${r.right + GAP}px`
    style.transform = 'translateY(-50%)'
    fixedStyle.value = style
    return
  }

  // Prefer below; flip above if it would leave the viewport.
  const preferBelow = !!props.below
  style.top = preferBelow ? `${r.bottom + GAP}px` : `${r.top - GAP}px`
  style.left = `${r.left + r.width / 2}px`
  style.transform = preferBelow ? 'translateX(-50%)' : 'translate(-50%, -100%)'
  fixedStyle.value = style
}

async function clampToViewport() {
  const tip = tipRef.value
  const el = rootRef.value
  if (!tip || !el) return
  const r = el.getBoundingClientRect()
  const tr = tip.getBoundingClientRect()
  const maxLeft = window.innerWidth - VIEW_PAD
  const maxTop = window.innerHeight - VIEW_PAD

  let left = tr.left
  let top = tr.top
  if (tr.right > maxLeft) left -= tr.right - maxLeft
  if (left < VIEW_PAD) left = VIEW_PAD
  if (tr.bottom > maxTop) {
    top = r.top - GAP - tr.height
  }
  if (top < VIEW_PAD) top = VIEW_PAD

  left = clamp(left, VIEW_PAD, Math.max(VIEW_PAD, window.innerWidth - tr.width - VIEW_PAD))
  top = clamp(top, VIEW_PAD, Math.max(VIEW_PAD, window.innerHeight - tr.height - VIEW_PAD))

  fixedStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
    transform: 'none',
  }
}

async function onEnter() {
  if (!props.fixed) return
  place()
  open.value = true
  await nextTick()
  await clampToViewport()
}

function onLeave() {
  if (!props.fixed) return
  open.value = false
}
</script>
