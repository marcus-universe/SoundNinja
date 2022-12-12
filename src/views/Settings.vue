<template>
    <div class="main settings flex_c_v align_c">
        <h1>Settings</h1> 
        <div class="settingsContainer gap1 flex_c_v align_c">
            <div class="flex_c gap1 w100 setting">
                <label for="hue">Primary Hue:</label>
                <input type="range" id="hue" name="hue" min="0" max="360" v-model="hue" @change="changeHue">
            </div>

            <div class="flex_c gap1 w100 setting">
                <label for="hue">Select Output-Device</label>
                <select name="cars" id="cars" v-model="outputSelected">
                    <option v-for="device in OutputDevices[0]" :key="device" >{{device}}</option>
                </select>
            </div>
        </div>   
        
    </div>
</template>
<script>
import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
export default {
    name: 'Settings',

    setup() {
        const hue = ref('189')
        const OutputDevices = ref([])
        const outputSelected = ref('')

        watch(hue, (hueval) => {
            document.documentElement.style.setProperty('--primary_color', 'hsl(' + hueval + ', 100%, 58%)');
        })


        async function getOutputDevices () {
            const devices = [await invoke('get_out_devices')];
            console.log(devices[0])
            OutputDevices.value.push(devices[0])
            return devices
        }
        getOutputDevices()

        return {
            hue,
            getOutputDevices,
            OutputDevices,
            outputSelected
        }
    },
    
}
</script>
<style lang="">
    
</style>