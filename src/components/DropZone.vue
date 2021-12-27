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
        inject("selectedTab");
    },
    components: {},
    methods: {
        drop(e) {
            console.log("Drop handler");
            console.log(e);
            e.preventDefault();
            root.style.setProperty("--dropzone-display", "0%");

            if (e.dataTransfer.files.length > 0) {
                let files = e.dataTransfer.files;
                for (let i = 0; i < files.length; i++) {
                    let file = files[i];
                    ipcRenderer.send("saveFile", file);
                }
            } else {
                console.log("No files");
            }
        },
        dragOver(e) {
            e.preventDefault();
        },
        dragLeave(e) {
            e.preventDefault();
            console.log("Inner drag leave");
        },
        dragEnter(e) {
            e.preventDefault();
            console.log("Inner drag enter");
        },
    },
};
</script>
