import { createRouter, createWebHashHistory } from "vue-router";
import MainView from "@/views/Main.vue";
import Settings from "@/views/Settings.vue";
import About from "@/views/About.vue";

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
  {
    path: "/about",
    name: "about",
    component: About,
  },
];

const router = createRouter({
  history: createWebHashHistory(process.env.BASE_URL),
  routes,
});

export default router;
