<template>
    <div class="DropZone">
        <div class="DropZone__inner" v-on:drop="drop" v-on:dragover="dragOver">
            <h1>Drop here!</h1>
        </div>
    </div>
</template>
<script>
import { ipcRenderer } from "electron";
import { inject } from "vue";

let root = document.documentElement;
export default {
    setup() {
        const selectedTab = inject("selectedTab");
        return {
            selectedTab,
        };
    },
    methods: {
        drop(e) {
            e.preventDefault();
            root.style.setProperty("--dropzone-display", "0%");
            root.style.setProperty("--dropzone-pointer-events", "none");

            // Save files if some are dropped
            if (e.dataTransfer.files.length < 1) return;

            let files = e.dataTransfer.files;
            for (const file of files) saveFile(file, this.selectedTab);
        },
        dragOver(e) {
            e.preventDefault();
        },
    },
};

const saveFile = (file, tab) => {
    tab = tab + "";
    const { name, path } = file;
    console.log({ file: { name, path }, tab });
    ipcRenderer.send("saveFile", { name, path }, tab);
};
</script>
