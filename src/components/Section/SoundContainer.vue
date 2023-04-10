<template>
  <div class="SoundContainer">
    <Suspense>
      <div
        class="SoundTab flex_c_h flex_start gap1 flex_wrap"
        ref="dropZoneRef"
        @drop="onDrop($event)"
        @dragover.prevent
        @dragenter.prevent
      >
        <div
          v-for="(sound, soundindex) in JSONFile"
          class="Soundbtn flex_c_v flex_wrap"
          :class="{ active: sound.active }"
          :key="sound"
          ref="dragButton"
          draggable="true"
          @dragstart="onDragStart($event, sound)"
          @dragend="onDragEnd"
          @click="setActiveSound(soundindex)"
          style=""
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
import { ref, computed } from "vue";
import { useStore } from "vuex";
import { invoke } from "@tauri-apps/api/tauri";

export default {
  setup() {
    const dragButton = ref(null);
    const dropZoneRef = ref < HTMLDivElement > null;
    const store = useStore();
    const currentTab = computed(() => {
      return store.state.currentTab;
    });
    const JSONFile = computed(() => {
      const sortByIndex = (a, b) => {
        return a.index - b.index;
      };

      return store.state.JsonHandeling.configFile?.files
        ?.filter((sound) => {
          return sound.tabs.includes(currentTab.value);
        })
        .sort(sortByIndex);
    });

    const Settings = computed(() => {
      return store.state.JsonHandeling.configFile?.settings;
    });

    function onDragStart(event, sound) {
      event.dataTransfer.setData("SoundID", sound.index);
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.dropEffect = "move";
      console.log(event.target, sound, sound.index);
      event.target.style.opacity = "0.5";
    }

    function onDragEnd(event) {
      event.target.style.opacity = "1";
    }

    function onDrop(event) {
      const itemID = event.dataTransfer.getData("SoundID");
      console.log(itemID);
    }

    return {
      dragButton,
      dropZoneRef,
      onDragStart,
      onDragEnd,
      onDrop,
      JSONFile,
      currentTab,
      Settings,
    };
  },

  computed: {
    FileStruct() {
      return this.$store.state.JsonHandeling.FileStruct;
    },

    // JSONFile() {
    //   return this.$store.state.JsonHandeling.configFile?.files?.filter(
    //     (sound) => {
    //       return sound.tabs.includes(this.currentTab);
    //     }
    //   );
    // },
    ErrorMessage() {
      return this.$store.state.ErrorMessage;
    },
    TabList() {
      return this.$store.state.JsonHandeling.TabList;
    },
    // currentTab() {
    //   return this.$store.state.currentTab;
    // },
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
        self.$store.dispatch("setActiveSound", { soundindex, status: true });
        invoke("play_sound", {
          soundPath: self.JSONFile[soundindex].path,
          deviceName: self.Settings.outputSource,
          active: false,
        });
      } else {
        self.$store.dispatch("setActiveSound", { soundindex, status: false });
        invoke("play_sound", {
          soundPath: self.JSONFile[soundindex].path,
          deviceName: self.Settings.outputSource,
          active: true,
        });
      }
    },
  },
};
</script>

<style lang=""></style>
