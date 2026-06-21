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

<style lang="scss">
.about {
  text-align: center;
  justify-content: center;
  align-items: center;

  h1 {
    text-align: center;
  }

  .aboutContainer {
    margin-top: 1rem;
    text-align: center;
    align-items: center;

    p {
      font-size: 1.1rem;
      text-align: center;
    }

    .github-link {
      color: var(--primary_color);
      text-decoration: none;
      font-size: 1rem;
      text-align: center;

      &:hover {
        text-decoration: underline;
      }
    }
  }
}
</style>
