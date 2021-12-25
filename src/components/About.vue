<template>
    <div class="AboutContainer flex-center ">
    <div class="AboutPage flex_c_h ">
        <img v-on:click="OpenAbout" src="../Assets/IMG/Exit.svg" alt="Exit Settings Button" class="ExitAbout">

        <div class="LogoAbout flex-">
            <img src="../Assets/IMG/Logo_Icon.svg" alt="">
            <h1>Sound<em>Ninja</em></h1>
        </div>
        <div class="Support">
            If you want to ❤ support ❤ me you can follow me here:
            <ul class="SupportLinks">
                <li class="flex-c-w">
                    <div class="flex-center">
                        <img src="../Assets/IMG/social_icons/Github.svg" alt="Github Icon">
                        <a @click="clicklink" data="https://github.com/MarcusTutorials" href="">Github</a>
                    </div>
                </li>

                <li class="flex-c-w">
                    <div class="flex-center">
                        <img src="../Assets/IMG/social_icons/Youtube.svg" alt="YouTube Icon">
                        <a @click="clicklink" data="https://www.youtube.com/user/MarcusTutorials1" href="">Youtube</a>
                    </div>
                </li>

                <li class="flex-c-w">
                    <div class="flex-center">
                        <img src="../Assets/IMG/social_icons/Instagram.svg" alt="Instagram Icon">
                        <a @click="clicklink" data="https://www.instagram.com/marcustutorials/">Instagram</a>
                    </div>
                </li>
            </ul>
        </div>

        <p>created by Marcus Harting (alias Marcus Tutorials) ©2021</p>
    </div>
</div>
</template>

<script>
const shell = require('electron').shell;

export default {
    name: "About",
    data() {
        return {
        }
    },
    methods: {
        OpenAbout() {
            document.querySelector(".AboutContainer").classList.toggle('ActiveSettings');
        },
        clicklink(e){
            shell.openExternal(e.currentTarget.getAttribute("href"))
        }
    }
}
</script>