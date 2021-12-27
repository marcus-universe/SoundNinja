<template>
    <div class="sidepanel">
        <div id="Upload" class="control_button" title="Upload File">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
                <g id="Upload">
                    <path
                        id="Ramen"
                        class="cls-1"
                        d="M162.12,61.73c-3.75-3.8-10.27-10.89-17.17-18.4-7.27-7.9-14.78-16.07-19.42-20.77l-2.3-2.34H63.05A27.71,27.71,0,0,0,35.57,48.08v104A27.7,27.7,0,0,0,63.05,180H137a27.7,27.7,0,0,0,27.48-27.85v-88ZM133.45,54.2c3,3.28,5.94,6.47,8.63,9.37h-12c-6.39,0-10.35-1.93-10.35-11.55V39.41C123.8,43.71,128.67,49,133.45,54.2ZM137,164H63.05A11.84,11.84,0,0,1,51.3,152.11v-104A11.85,11.85,0,0,1,63.05,36.16h41V52c0,17.22,9.74,27.49,26.07,27.49h18.6v72.6A11.84,11.84,0,0,1,137,164Z"
                    />
                    <path
                        id="Pfeil"
                        class="cls-1"
                        d="M81,107.54a6.87,6.87,0,0,0,4.9-2.06l7.13-7.23v27.89a6.94,6.94,0,1,0,13.88,0V97.84l7,7.11a6.87,6.87,0,0,0,9.81,0,7.1,7.1,0,0,0,0-10L105.31,76.29a6.91,6.91,0,0,0-5.11-2.05h-.28q-.3,0-.6,0l-.34,0c-.11,0-.22,0-.33.06s-.25.06-.37.09l-.29.07-.35.13-.29.11-.32.14-.3.15-.29.18a1.75,1.75,0,0,0-.29.18l-.31.22-.24.19a6.39,6.39,0,0,0-.48.44l0,0-19,19.24a7.1,7.1,0,0,0,0,10A6.88,6.88,0,0,0,81,107.54Z"
                    />
                    <path
                        id="Line"
                        class="cls-1"
                        d="M131.76,136.2H69a7,7,0,0,0,0,14.06h62.72a7,7,0,0,0,0-14.06Z"
                    />
                </g>
            </svg>
        </div>

        <div id="Folder" class="control_button" @click="OpenFolder" title="Upload Folder">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
                <g id="Folder">
                    <g id="darker">
                        <path
                            class="cls-2"
                            d="M164.85,74.82c0,19-2.62,75.75-2.94,82.83-.59.28-1.2.53-1.81.75l-70.18.86H37.27c-.92-.11-1.81-.26-2.66-.42-11.27-2.2-15.26-8.89-15.26-26.07,0,0-.4-52.58-.4-68.1S17.46,36.33,40.43,36.33H73.26L88.47,51.24h51.31C155.89,51.24,164.85,54.53,164.85,74.82Z"
                        />
                    </g>
                    <g id="path">
                        <path
                            class="cls-1"
                            d="M181.67,89.07,171.83,145a17.86,17.86,0,0,1-9.92,12.64c-.59.28-1.2.53-1.81.75l-70.18.86H37.27c-.92-.11-1.81-.26-2.66-.42L46.89,89.07c1.37-7.85,8.91-14.25,16.74-14.25H169.94C177.77,74.82,183,81.22,181.67,89.07Z"
                        />
                    </g>
                </g>
            </svg>
        </div>

        <div id="Stop" class="control_button" title="Stop Playback">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
                <g id="Fullscreen">
                    <rect
                        class="cls-1"
                        x="19.3"
                        y="19.3"
                        width="161.41"
                        height="161.41"
                        rx="23.39"
                    />
                </g>
            </svg>
        </div>

        <div id="Reset" class="control_button" @click="Reset" title="Reset">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
                <g id="Reset">
                    <path
                        class="cls-1"
                        d="M175.9,100.2a77.14,77.14,0,0,0-154.28.39c0,20.07,9.15,39.42,25.75,54.47,14.93,13.53,34.14,21.61,51.4,21.61v-27c-19.73,0-50.15-21.08-50.15-49.08a50.14,50.14,0,0,1,100.28-.19l-18.86.13,32.13,31.88,32.61-32.35Z"
                    />
                </g>
            </svg>
        </div>

        <div v-on:click="OpenSettings" id="Settings" class="control_button" title="Open Settings">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
                <g id="Settings">
                    <path
                        class="cls-1"
                        d="M169.43,82.73H157.55a59.63,59.63,0,0,0-6.14-13.8l8-8a10.56,10.56,0,0,0,0-14.9l-7.66-7.66a10.56,10.56,0,0,0-14.9,0L128.18,47a59.32,59.32,0,0,0-13.84-5.26V30.05a10.56,10.56,0,0,0-10.53-10.53H93A10.57,10.57,0,0,0,82.43,30.05V42.74a59.69,59.69,0,0,0-13.2,5.92h0l-8.59-8.58a10.56,10.56,0,0,0-14.9,0l-7.66,7.66a10.56,10.56,0,0,0,0,14.9l9.16,9.16a59,59,0,0,0-5.18,13.34H29.76A10.57,10.57,0,0,0,19.22,95.67v10.84a10.57,10.57,0,0,0,10.54,10.54H42.64a59.77,59.77,0,0,0,5.72,13.21h0l-8.58,8.58a10.58,10.58,0,0,0,0,14.9l7.66,7.66a10.56,10.56,0,0,0,14.9,0l8.85-8.85h0A61,61,0,0,0,84.84,158v11.69a10.57,10.57,0,0,0,10.54,10.54h10.83a10.57,10.57,0,0,0,10.54-10.54V157.66a58.7,58.7,0,0,0,13.8-6l8,8a10.56,10.56,0,0,0,14.9,0l7.67-7.66a10.58,10.58,0,0,0,0-14.9l-8.34-8.34h0a60.17,60.17,0,0,0,5.54-14.16h11.12A10.57,10.57,0,0,0,180,104.1V93.27A10.57,10.57,0,0,0,169.43,82.73Zm-69.29,50A32.66,32.66,0,1,1,132.8,100,32.66,32.66,0,0,1,100.14,132.7Z"
                    />
                </g>
            </svg>
        </div>

        <div v-on:click="OpenAbout" id="About" class="About control_button" title="Open About">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
                <g id="About">
                    <rect class="cls-1" x="84.9" y="65.18" width="30.2" height="115.96" rx="5.33" />
                    <circle class="cls-1" cx="100" cy="34.04" r="15.1" />
                </g>
            </svg>
        </div>
    </div>
</template>

<script>
const dialog = require("electron").remote.dialog;
const fs = require("fs");
const path = require("path");

export default {
    name: "SideMenu",
    data() {
        return {
            isOpenAbout: false,
            isOpenSettings: false,
        };
    },
    emits: ["refreshData"],

    methods: {
        Reset() {
            let AllFolderData;

            fs.readFile("./db.json", "utf8", (err, data) => {
                if (err) throw err;
                AllFolderData = JSON.parse(data);
                AllFolderData.folder.forEach((element) => {
                    AllFolderData.folder.splice(element, AllFolderData.folder.length);
                });

                fs.writeFile("./db.json", JSON.stringify(AllFolderData, null, 2), "utf8", (err) => {
                    if (err) throw err;
                    this.$emit("refreshData", AllFolderData);
                });
            });
        },

        OpenAbout() {
            document.querySelector(".AboutContainer").classList.toggle("ActiveSettings");
        },
        OpenSettings() {
            document.querySelector(".SettingPanel").classList.toggle("ActiveSettings");
        },
        OpenFolder() {
            let json;

            fs.readFile("./db.json", "utf8", (err, data) => {
                if (err) throw err;
                json = JSON.parse(data);
            });

            dialog
                .showOpenDialog({
                    properties: ["openDirectory", "multiSelections"],
                })
                .then((result) => {
                    console.log("Three");
                    //?Get Selected Folders

                    var folderpath = result.filePaths;

                    folderpath.forEach((element) => {
                        var foldername = path.basename(element.toString());
                        let folderfiles = fs.readdirSync(element);
                        var index = folderpath.indexOf(element);

                        var JsonStruct = {
                            id: Number,
                            name: "",
                            foldername: "",
                            path: "",
                            audiofiles: [],
                        };

                        JsonStruct.id = index;
                        JsonStruct.name = foldername;
                        JsonStruct.foldername = foldername;
                        JsonStruct.path = element;

                        folderfiles.forEach((file) => {
                            var indexfile = folderfiles.indexOf(file);

                            JsonStruct.audiofiles.push({
                                id: indexfile,
                                audioname: file,
                                audiourl: path.join(element, file),
                            });
                        });

                        json.folder.push(JsonStruct);

                        fs.writeFile("./db.json", JSON.stringify(json, null, 2), (err) => {
                            if (err) throw err;
                            this.$emit("refreshData", json);
                        });
                    });
                });
        },
    },
};
</script>
