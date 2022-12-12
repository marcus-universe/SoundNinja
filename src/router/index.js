import { createRouter, createWebHashHistory } from 'vue-router'
const MainView = () => import(/* webpackChunkName: 'ContactSubmit' */ '@/views/Main.vue')
const Settings = () => import(/* webpackChunkName: 'ContactSubmit' */ '@/views/Settings.vue')

const routes = [
    {
    path: '/',
    name: 'main',
    component: MainView
  },
  {
    path: '/settings',
    name: 'settings',
    component: Settings
  },
]

const router = createRouter({
  history: createWebHashHistory(process.env.BASE_URL),
  routes
})

export default router
