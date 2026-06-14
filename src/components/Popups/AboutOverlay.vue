<template>
  <Transition name="overlay-fade">
    <div v-if="appStore.activeOverlay === 'about'" class="overlay" @click.self="appStore.setActiveOverlay(null)">
      <div class="overlay-panel flex_c_v align_c">
        <div class="overlay-header flex_c_h flex_space_between w100">
          <h1>About</h1>
          <Icons :icon="'exit'" :customClass="'exit overlay-close'" @triggered="appStore.setActiveOverlay(null)" />
        </div>
        <div class="aboutContainer flex_c_v align_c gap1">
          <p>by <strong>MarcusUniverse</strong></p>
          <a
            href="https://github.com/marcus-universe/SoundNinja"
            target="_blank"
            rel="noopener noreferrer"
            class="github-link"
          >
            SoundNinja on GitHub
          </a>
        </div>
      </div>
      <BlurBG />
    </div>
  </Transition>
</template>

<script setup>
const appStore = useAppStore()
</script>
