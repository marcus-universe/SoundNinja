<template>
  <div class="main about flex_c_v align_c">
    <h1>{{ $t('about.title') }}</h1>
    <div class="aboutContainer flex_c_v align_c gap1">
      <p>{{ $t('about.by') }} <strong>MarcusUniverse</strong></p>
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
</template>

<script setup>
const { t } = useI18n()
</script>
