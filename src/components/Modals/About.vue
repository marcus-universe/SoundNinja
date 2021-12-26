<template>
    <div class="AboutContainer flex-center">
        <div class="AboutPage flex_c_h">
            <img
                v-on:click="OpenAbout"
                src="../../Assets/IMG/Exit.svg"
                alt="Exit Settings Button"
                class="ExitAbout"
            />

            <div class="LogoAbout flex-">
                <img src="../../Assets/IMG/Logo_Icon.svg" alt="" />
                <h1>Sound<em>Ninja</em></h1>
            </div>
            <div class="Support">
                If you want to ❤ support ❤ me you can follow me here:
                <ul class="SupportLinks">
                    <GitHub @clickLink="clickLink" />
                    <YouTube @clickLink="clickLink" />
                    <Instagram @clickLink="clickLink" />
                </ul>
            </div>

            <p>created by Marcus Harting (alias Marcus Tutorials) ©2021</p>
        </div>
    </div>
</template>

<script>
import GitHub from "../Icons/GitHub.vue";
import YouTube from "../Icons/YouTube.vue";
import Instagram from "../Icons/Instagram.vue";

const shell = require("electron").shell;

export default {
    name: "About",
    components: {
        GitHub,
        YouTube,
        Instagram,
    },
    data() {
        return {};
    },
    methods: {
        OpenAbout() {
            document.querySelector(".AboutContainer").classList.toggle("ActiveSettings");
        },
        clickLink(e) {
            shell.openExternal(e.currentTarget.getAttribute("href"));
        },
    },
};
</script>
