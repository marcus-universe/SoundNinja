import { createRouter, createWebHashHistory } from "vue-router";
import MainView from "@/views/Main.vue";
import Settings from "@/views/Settings.vue";

const routes = [
  {
    path: "/",
    name: "main",
    component: MainView,
  },
  {
    path: "/settings",
    name: "settings",
    component: Settings,
  },
];

const router = createRouter({
  history: createWebHashHistory(process.env.BASE_URL),
  routes,
});

export default router;
