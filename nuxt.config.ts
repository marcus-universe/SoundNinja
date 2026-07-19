// https://nuxt.com/docs/api/configuration/nuxt-config
import { defineNuxtConfig } from 'nuxt/config'

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
      publicDir: 'dist',
    },
  },

  modules: ['@pinia/nuxt', '@nuxtjs/i18n'],

  i18n: {
    strategy: 'no_prefix',
    defaultLocale: 'en',
    locales: [
      { code: 'en', name: 'English', file: 'en.json' },
      { code: 'de', name: 'Deutsch', file: 'de.json' },
    ],
    langDir: 'locales/',
    detectBrowserLanguage: false,
  },

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
    '~/assets/scss/settings.scss',
    '~/assets/scss/dialogs.scss',
    '~/assets/scss/ui-elements.scss',
    '~/assets/scss/titlebar.scss',
    '~/assets/scss/context-menu.scss',
    '~/assets/scss/import-folders.scss',
    '~/assets/scss/pages.scss',
    '~/assets/scss/settings-audio.scss',
  ],

  vite: {
    clearScreen: false,
    server: {
      strictPort: true,
    },
    envPrefix: ['VITE_', 'TAURI_'],
    build: {
      // Disable source maps in production to reduce bundle size and startup I/O.
      sourcemap: process.env.NODE_ENV !== 'production',
      rollupOptions: {
        output: {
          // Split heavy settings/theme-creator panels into separate chunks so
          // the main soundboard loads without waiting for them.
          manualChunks(id) {
            if (id.includes('SettingsThemeCreator')) return 'chunk-theme-creator'
            if (id.includes('SettingsAudio') || id.includes('SettingsMain') || id.includes('SettingsAbout') || id.includes('SettingsOverlay')) return 'chunk-settings'
          },
        },
      },
    },
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
