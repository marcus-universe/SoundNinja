<template>
  <div class="soundninja flex_c_h flex_space_between">
    <div>{{ getConfig }}</div>

    <NavBar />
    <ErrorAlert />
    <router-view v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component
          :is="Component"
          :key="$route.path"
          :class="{ searchMove: store.Searchbar.SearchbarActive }"
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

    onMounted(() => {
      readConfigFile();
    });
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
