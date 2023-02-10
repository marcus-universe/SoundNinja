<template>
  <div class="SoundContainer">
    <Suspense>
      <div class="SoundTab flex_c_h flex_start gap1 flex_wrap">
        <div
          v-for="(sound, soundindex) in JSONFile"
          class="Soundbtn flex_c_v flex_wrap"
          :class="{ active: soundindex === activeSounds }"
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
import { convertFileSrc, invoke } from "@tauri-apps/api/tauri";
import { Howl, Howler } from "howler";

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
    activeSound(soundindex) {
      if (this.soundPlaying) {
        this.activeSounds = soundindex;
      } else {
        this.activeSounds = null;
      }
    },
    setActiveSound(soundindex) {
      var self = this;
      this.soundPlaying = true;

      this.activeSound(soundindex);
      var filepath = convertFileSrc(self.JSONFile[soundindex].path);
      var volume = this.JSONFile[soundindex]?.volume;
      var sound = new Howl({
        src: [filepath],
        volume: volume,
        html5: true,
        onend: function () {
          self.soundPlaying = false;
          self.activeSounds = null;
          console.log("Finished!");
        },
        onplayerror: function () {
          self.soundPlaying = false;
          self.activeSounds = null;
          self.ErrorMessage =
            "Error: " +
            self.JSONFile?.files.tabs[soundindex].sounds[soundindex].name +
            " can't be played!";
        },
      });

      Howler.stop();

      this.JSONFile[soundindex].active = !self.JSONFile[soundindex].active;
      this.JSONFile[soundindex].forEach((sound, index) => {
        if (index !== soundindex) {
          sound.active = false;
        }
      });

      if (self.JSONFile[soundindex].active) {
        self.JSONFile[soundindex].active = true;
        sound.once("load", function () {
          //   sound.play();
          invoke("play_sound", filepath);
        });
      } else {
        self.JSONFile[soundindex].active = false;
        sound.stop();

        self.soundPlaying = false;
        self.activeSounds = null;
      }
    },
  },
};
</script>

<style lang=""></style>
