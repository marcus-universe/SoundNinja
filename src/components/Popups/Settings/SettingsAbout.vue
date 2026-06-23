<template>
  <section class="about-section">
    <h2 class="settings-content__title">{{ $t('settings.about.title') }}</h2>

    <div class="about-logo">
      <img src="/designs/Logo_Animated.gif" alt="Sound Ninja" class="about-logo__img" loop autoplay />
    </div>

    <div class="about-version">
      <span class="about-version__num">v{{ appVersion }}</span>
    </div>

    <div class="about-links">
      <a
        href="https://github.com/marcus-universe/SoundNinja"
        target="_blank"
        rel="noopener noreferrer"
        class="about-link"
      >
        <svg viewBox="0 0 24 24" fill="currentColor" class="about-link__icon">
          <path d="M12 0C5.37 0 0 5.37 0 12c0 5.3 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.63-5.37-12-12-12z"/>
        </svg>
        {{ $t('settings.about.githubLink') }}
      </a>
      <a
        href="https://github.com/marcus-universe/SoundNinja/blob/Sound-Ninja-Tauri/LICENSE"
        target="_blank"
        rel="noopener noreferrer"
        class="about-link"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="about-link__icon">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
        {{ $t('settings.about.licenseLink') }}
      </a>
    </div>
  </section>
</template>

<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app'

const appVersion = ref('')

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = ''
  }
})
</script>
