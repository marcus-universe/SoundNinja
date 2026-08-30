import tailwindcss from '@tailwindcss/vite'

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: false },

  css: ['./app/assets/css/main.css'],

  experimental: {
    appManifest: false,
  },

  vite: {
    plugins: [tailwindcss()],
  },

  modules: ['@nuxt/content', '@nuxt/fonts', '@nuxtjs/i18n', '@nuxt/icon'],

  content: {
    experimental: { sqliteConnector: 'native' },
  },

  fonts: {
    families: [
      {
        name: 'Nunito',
        provider: 'google',
        weights: [400, 600, 700, 800, 900],
      },
    ],
  },

  icon: {
    provider: 'none',
    clientBundle: {
      icons: [
        'ic:baseline-apple',
        'ri:windows-fill',
        'fa7-brands:linux',
      ],
    },
  },

  i18n: {
    defaultLocale: 'en',
    strategy: 'prefix_except_default',
    langDir: 'locales',
    locales: [
      { code: 'en', language: 'en-US', name: 'English', file: 'en.json' },
      { code: 'de', language: 'de-DE', name: 'Deutsch', file: 'de.json' },
      { code: 'fr', language: 'fr-FR', name: 'Français', file: 'fr.json' },
      { code: 'es', language: 'es-ES', name: 'Español', file: 'es.json' },
      { code: 'ja', language: 'ja-JP', name: '日本語', file: 'ja.json' },
      { code: 'zh', language: 'zh-CN', name: '中文', file: 'zh.json' },
    ],
    detectBrowserLanguage: false,
  },

  app: {
    baseURL: '/SoundNinja/',
    head: {
      title: 'SoundNinja — Open Source Soundboard',
      meta: [
        { name: 'description', content: 'Modern, customizable, open source soundboard. Built with Tauri + Nuxt.' },
        { name: 'theme-color', content: '#222831' },
        { property: 'og:title', content: 'SoundNinja — Open Source Soundboard' },
        { property: 'og:description', content: 'Modern, customizable, open source soundboard.' },
        { property: 'og:type', content: 'website' },
      ],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
      ],
      htmlAttrs: { lang: 'en' },
    },
  },

  nitro: {
    prerender: {
      crawlLinks: true,
      routes: prerenderRoutes(),
    },
  },
})

function prerenderRoutes() {
  const localeCodes = ['en', 'de', 'fr', 'es', 'ja', 'zh']
  const pages = ['', '/download', '/impressum']
  const docsSlugs = [
    '',
    '/installation',
    '/adding-sounds',
    '/organizing-sounds',
    '/theme-editor',
    '/recording-editor',
    '/remote-control',
    '/support',
  ]

  return localeCodes.flatMap((code) => {
    const prefix = code === 'en' ? '' : `/${code}`
    return [
      ...pages.map(page => `${prefix}${page || '/'}`),
      ...docsSlugs.map(slug => `${prefix}/docs${slug}`),
    ]
  })
}
