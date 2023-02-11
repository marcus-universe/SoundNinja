<template>
  <div class="SoundContainer">
    <Suspense>
      <div class="SoundTab flex_c_h flex_start gap1 flex_wrap">
        <div
          v-for="(sound, soundindex) in JSONFile"
          class="Soundbtn flex_c_v flex_wrap"
          :class="{ active: sound.active }"
          :key="sound"
          @click="setActiveSound(soundindex)"
        >
          {{ sound.name }}
        </div>
      </div>

      <template #fallback>
        <div>Is loading...</div>
      </template>
    </Suspense>
  </div>
</template>

<script>
// import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";

export default {
  computed: {
    FileStruct() {
      return this.$store.state.JsonHandeling.FileStruct;
    },

    JSONFile() {
      return this.$store.state.JsonHandeling.configFile?.files?.filter(
        (sound) => {
          return sound.tabs.includes(this.currentTab);
        }
      );
    },
    ErrorMessage() {
      return this.$store.state.ErrorMessage;
    },
    TabList() {
      return this.$store.state.JsonHandeling.TabList;
    },
    currentTab() {
      return this.$store.state.currentTab;
    },
  },
  data() {
    return {
      activeSounds: null,
      soundPlaying: false,
    };
  },
  methods: {
    setActiveSound(soundindex) {
      var self = this;
      // var filepath = convertFileSrc(self.JSONFile[soundindex].path);

      if (!self.JSONFile[soundindex].active) {
        // invoke("play_sound", { filepath, });
        self.$store.dispatch("setActiveSound", { soundindex, status: true });
      } else {
        self.$store.dispatch("setActiveSound", { soundindex, status: false });
      }
    },
  },
};
</script>

<style lang=""></style>
