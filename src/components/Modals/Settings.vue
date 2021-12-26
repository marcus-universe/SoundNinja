<template>
    <div class="SettingPanel flex-center">
        <div class="SettingBox flex-c-w">
            <img
                v-on:click="OpenSettings"
                src="../../Assets/IMG/Exit.svg"
                alt="Exit Settings Button"
                class="ExitSetting"
            />
            <h2>Settings</h2>
            <ul class="flex_c_h SettingList">
                <li class="flex-c-rand">
                    Color Theme:
                    <input
                        v-model="colorpick"
                        v-on:input="changeColor"
                        v-on:change="writeColor"
                        type="color"
                        name="Colorpicker"
                        id="Colorpicker"
                        class="colorpicker"
                    />
                </li>
                <li class="flex-c-rand">
                    Show Logo Icon:<input
                        type="checkbox"
                        name="LogoActive"
                        id=""
                        class="checkbox"
                    />
                </li>
                <li class="flex-c-rand select">
                    <label for="audioSource">Audio input source: </label>
                    <select id="audioSource"></select>
                </li>
                <li></li>
            </ul>
        </div>
    </div>
</template>

<script>
import { ipcRenderer } from "electron";
let root = document.documentElement;
let colorPick;

export default {
    data() {
        return {
            colorpick: "#000000",
        };
    },

    async created() {
        const config = await ipcRenderer.invoke("getConfig");
        colorPick = config.settings[0].color;
        this.colorpick = colorPick;
    },

    methods: {
        OpenSettings() {
            document.querySelector(".SettingPanel").classList.toggle("ActiveSettings");
        },

        changeColor() {
            root.style.setProperty("--primary_color", this.colorpick);
            colorPick = this.colorpick;
        },

        writeColor() {
            ipcRenderer.invoke("mergeConfig", {
                settings: [
                    {
                        color: colorPick,
                    },
                ],
            });
        },
    },
};
</script>
