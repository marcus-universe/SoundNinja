// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2024-11-01',
  srcDir: 'src/',
  ssr: false,

  typescript: {
    tsConfig: {
      compilerOptions: {
        target: 'ESNext',
        moduleResolution: 'bundler',
        ignoreDeprecations: '6.0',
      },
    },
  },

  nitro: {
    preset: 'static',
    output: {
      publicDir: '../dist',
    },
  },

  modules: ['@pinia/nuxt', '@vueuse/nuxt'],

  components: {
    dirs: [
      {
        path: '~/components',
        pathPrefix: false,
      },
    ],
  },

  css: [
    '~/assets/scss/_base.scss',
    '~/assets/scss/style.scss',
    '~/assets/scss/navbar.scss',
  ],

  vite: {
    clearScreen: false,
    server: {
      strictPort: true,
    },
    envPrefix: ['VITE_', 'TAURI_'],
  },

  router: {
    options: {
      hashMode: true,
    },
  },

  app: {
    head: {
      title: 'Sound Ninja',
      meta: [
        { charset: 'utf-8' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1.0' },
      ],
    },
  },

  devtools: { enabled: false },

  experimental: {
    viteEnvironmentApi: true,
  },
})
