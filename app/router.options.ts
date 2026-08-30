import type { RouterConfig } from '@nuxt/schema'
import { defineComponent } from 'vue'

const Empty = defineComponent({
  name: 'ViteAssetIgnore',
  render: () => null,
})

export default {
  routes: (routes) => [
    {
      name: 'vite-asset-ignore',
      path: '/_nuxt/:pathMatch(.*)*',
      component: Empty,
      meta: { i18n: false },
    },
    ...routes,
  ],
} satisfies RouterConfig
