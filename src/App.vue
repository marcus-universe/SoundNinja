<template>
    <div v-on:dragenter="dragEnter" v-on:dragleave="dragLeave">
        <TopBar />

        <div class="flex-c-w max_h ContentSection">
            <div class="flex_c_h sidepanel_w">
                <Sidemenu />
            </div>

            <About />
            <Settings />

            <div class="flex_c_h Soundpad">
                <Tabwrapper :tabList="tabList">
                    <Tab />
                    <DropZone />
                </Tabwrapper>
            </div>
        </div>
    </div>
</template>

<style lang="sass" src="./SASS/style.sass"></style>

<script>
import TopBar from "./components/TopBar.vue";
import Sidemenu from "./components/Sidemenu.vue";
import About from "./components/Modals/About.vue";
import Settings from "./components/Modals/Settings.vue";
import DropZone from "./components/DropZone.vue";
import { ipcRenderer } from "electron";
import Tabwrapper from "./components/TabwrapperContainer.vue";
import Tab from "./components/Tab.vue";

let root = document.documentElement;
let count = 0;

import db from "../db.json";

export default {
    name: "SoundNinja",
    methods: {
        dragEnter() {
            count++;
            root.style.setProperty("--dropzone-display", "100%");
            root.style.setProperty("--dropzone-pointer-events", "auto");
        },
        dragLeave() {
            count--;
            console.log(count);
            if (count === 0) {
                root.style.setProperty("--dropzone-display", "0%");
                root.style.setProperty("--dropzone-pointer-events", "none");
            }
        },
    },
    data() {
        return {
            tabList: [],
            soundList: [],
            dragState: false,
        };
    },
    components: {
        TopBar,
        Sidemenu,
        About,
        Settings,
        Tabwrapper,
        Tab,
        DropZone,
    },
    created() {
        this.tabList = [];

        for (const tab of db.folder) {
            this.tabList.push(tab);
        }
    },
};

ipcRenderer.invoke("getColor").then((color) => {
    // Set Color Theme
    root.style.setProperty("--primary_color", color);
});
</script>
