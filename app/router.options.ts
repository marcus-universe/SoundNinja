import type { RouterConfig } from '@nuxt/schema'
import { defineComponent } from 'vue'

const Empty = defineComponent({
  name: 'ViteAssetIgnore',
  render: () => null,
})

export default {
  scrollBehavior(to, _from, savedPosition) {
    if (to.hash) {
      return { el: to.hash, top: 72, behavior: 'smooth' }
    }
    if (savedPosition) {
      return savedPosition
    }
    return { top: 0 }
  },
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
