<template>
  <div class="main settings flex_c_v align_c">
    <h1>Settings</h1>
    <div class="settingsContainer gap1 flex_c_v align_c">
      <div class="flex_c gap1 w100 setting">
        <label for="hue">Primary Hue:</label>
        <input
          type="range"
          id="hue"
          name="hue"
          min="0"
          max="360"
          v-model="hue"
          @change="changeHue"
        />
      </div>

      <div class="flex_c gap1 w100 setting">
        <label for="hue">Select Output-Device</label>
        <select
          name="cars"
          id="cars"
          v-model="outputSelected"
          @change="selectOutputDevice"
        >
          <option
            v-for="device in OutputDevices[0]"
            :key="device"
            :value="device"
          >
            {{ device }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>
<script>
import { ref, watch, computed, onUpdated, onMounted } from "vue";
import { useStore } from "vuex";
import { invoke } from "@tauri-apps/api/tauri";
export default {
  name: "Settings",

  setup() {
    const OutputDevices = ref([]);
    const outputSelected = ref("");
    const store = useStore();
    const hueState = computed(() => {
      return store.state.JsonHandeling.configFile?.settings.hue;
    });

    const OutputState = computed(() => {
      return store.state.JsonHandeling.configFile?.settings?.outputSource;
    });
    const hue = ref();

    onUpdated(() => {
      hue.value = hueState.value;
    });
    onMounted(() => {
      if (
        (OutputState.value !== undefined,
        OutputState.value !== null,
        OutputState.value !== "")
      ) {
        outputSelected.value = OutputState.value;
      } else {
        outputSelected.value = outputSelected.value[0];
      }
    });

    watch(hue, (hueval) => {
      store.dispatch("setHue", parseInt(hueval));
    });

    async function getOutputDevices() {
      const devices = [await invoke("get_out_devices")];
      OutputDevices.value.push(devices[0]);
      return devices;
    }
    getOutputDevices();

    const selectOutputDevice = (event) => {
      const selectedDevice = event.target.value;
      store.dispatch("setOutSource", selectedDevice);
      console.log(selectedDevice);
    };

    // async function selectOutputDevice() {
    //   // await invoke('set_out_device', {device: outputSelected.value})
    // }

    return {
      hue,
      getOutputDevices,
      OutputDevices,
      outputSelected,
      selectOutputDevice,
    };
  },
};
</script>
<style lang=""></style>
