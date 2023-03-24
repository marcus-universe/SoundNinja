<template>
  <div data-tauri-drag-region class="titlebar">
    <div class="titlebar-button" id="titlebar-minimize" @click="minimize()">
      <img
        src="https://api.iconify.design/mdi:window-minimize.svg"
        alt="minimize"
      />
    </div>
    <div class="titlebar-button" id="titlebar-maximize" @click="maximize()">
      <img
        src="https://api.iconify.design/mdi:window-maximize.svg"
        alt="maximize"
      />
    </div>
    <div class="titlebar-button" id="titlebar-close" @click="close()">
      <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
    </div>
  </div>

  <div class="soundninja flex_c_h flex_space_between">
    <div>{{ getConfig }}</div>

    <NavBar />
    <ErrorAlert />
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component
          :is="Component"
          :key="$route.path"
        />
      </transition>
    </router-view>
  </div>
</template>

<script>
import NavBar from "@/components/Section/NavBar.vue";
import ErrorAlert from "@/components/Popups/ErrorAlert.vue";
import { onMounted, watch, computed } from "vue";
import { readTextFile, BaseDirectory } from "@tauri-apps/api/fs";
import { useStore } from "vuex";
import { appWindow } from "@tauri-apps/api/window";

export default {
  name: "App",

  setup() {
    const store = useStore();
    const hue = computed(() => {
      return store.state.JsonHandeling.configFile?.settings.hue;
    });
    // const configFileContents = ref("");

    watch(hue, (hueval) => {
      document.documentElement.style.setProperty(
        "--primary_color",
        "hsl(" + hueval + ", 100%, 58%)"
      );
    });

    async function readConfigFile() {
      const contents = JSON.parse(
        await readTextFile("config.json", {
          dir: BaseDirectory.App,
        })
      );
      store.dispatch("updateConfigFile", contents);
      return contents;
    }

    const minimize = () => {
      appWindow.minimize();
    };

    const maximize = () => {
      appWindow.maximize();
    };

    const close = () => {
      appWindow.close();
    };

    onMounted(() => {
      readConfigFile();
    });

    return {
      minimize,
      maximize,
      close,
    };
  },
  components: {
    NavBar,
    ErrorAlert,
  },

  computed: {
    store() {
      return this.$store.state;
    },
    getConfig() {
      return this.$store.getters.getConfig;
    },
  },
};
</script>

<style lang="scss">
@import "@/sass/_base.sass";
@import "@/sass/style.sass";
</style>
