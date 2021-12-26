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
let root = document.documentElement;
const fs = require("fs");
var colorPick;

export default {
    data() {
        return {
            colorpick: "#000000",
        };
    },

    created() {
        fs.readFile("../config.json", "utf-8", function (err, data) {
            if (err) throw err;
            var SettingData = JSON.parse(data);
            colorPick = SettingData.settings[0].color;
        });
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
            fs.readFile("./config.json", "utf-8", function (err, data) {
                if (err) throw err;
                var SettingData = JSON.parse(data);
                SettingData.settings[0].color = colorPick;
                fs.writeFile(
                    "./config.json",
                    JSON.stringify(SettingData, null, 2),
                    "utf-8",
                    function (err) {
                        if (err) throw err;
                    }
                );
            });
        },
    },
};
</script>
