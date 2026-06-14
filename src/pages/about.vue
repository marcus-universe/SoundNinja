<template>
  <div class="main about flex_c_v align_c">
    <h1>About</h1>
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
</template>

<style lang="scss">
.about {
  .aboutContainer {
    margin-top: 1rem;

    p {
      font-size: 1.1rem;
    }

    .github-link {
      color: var(--primary_color);
      text-decoration: none;
      font-size: 1rem;

      &:hover {
        text-decoration: underline;
      }
    }
  }
}
</style>
