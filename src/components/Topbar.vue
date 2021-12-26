<template>
    <div class="flex-center LogoSection">
        <div class="flex-center LogoIconContainer">
            <img src="../Assets/IMG/Logo_Icon.svg" alt="" />
        </div>

        <h1>Sound<em>Ninja</em></h1>
    </div>

    <div class="windowoptions">
        <Minimize @onClick="Minimize" />
        <Maximize @onClick="Maximize" />
        <ExitApp @onClick="ExitApp" />
    </div>
</template>

<script>
import Minimize from "./Icons/Minimize.vue";
import Maximize from "./Icons/Maximize.vue";
import ExitApp from "./Icons/ExitApp.vue";
import { ipcRenderer } from "electron";

export default {
    name: "TopBar",
    components: {
        Minimize,
        Maximize,
        ExitApp,
    },
    props: {
        msg: String,
    },
    methods: {
        Minimize() {
            ipcRenderer.send("hideapp");
        },
        ExitApp() {
            window.close();
        },
        Maximize() {
            ipcRenderer.send("maximizeapp");
        },
    },
};
</script>
