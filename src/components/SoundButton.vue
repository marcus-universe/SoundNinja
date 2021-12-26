<template>
    <div
        class="SoundButton grid_c"
        :data-attribute="sound.audiourl"
        :class="{ SoundButtonActive: clicked }"
        @click="playSound(sound.audiourl)"
    >
        {{ sound.audioname.replaceAll(".mp3", "").replaceAll(".wav", "").replaceAll(".ogg", "") }}
    </div>
</template>

<script>
import { Howl } from "howler";

export default {
    name: "SoundButton",
    data() {
        return {
            clicked: false,
            playingsound: null,
        };
    },
    props: ["sound"],
    methods: {
        playSound(url) {
            this.clicked = !this.clicked;

            const contextmenu = document.querySelector(".contextmenuTab");
            contextmenu.style.display = "none";

            console.log("playSound: " + url);
            if (this.clicked == true) {
                this.playingsound = new Howl({
                    src: [url],
                    autoplay: false,
                    loop: false,
                    volume: 0.5,
                    html5: true,
                    preload: true,
                    onend: function () {
                        this.clicked = false;
                        console.log("Finished!");
                        console.log(this.clicked);
                    }.bind(this),
                });
                this.playingsound.play();
            } else {
                this.playingsound.stop();
                this.clicked = false;
            }
            // if (url) {
            //     var audio = new Audio(url);
            //     audio.play();
            // }
        },
    },
};
</script>
<style lang="">
.SoundButton {
    /* Make text unselectable */
    -webkit-touch-callout: none;
    -webkit-user-select: none;
    -khtml-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    align-items: center;
    background: $color_s;
    padding: 20px 20px;
    max-height: 160px;
    width: clamp(100px, 10%, 150px);
    max-width: var(--ButtonWidth);
    word-break: break-word;
    hyphens: auto;
    white-space: pre-wrap;
    width: 100%;
    color: $color-li;
    font-family: $Nunito-R;
    text-align: center;
    transition: all 0.3s ease;
    border-radius: 10px;
    animation: ShowButtons 0.3s ease;
}
.SoundButton:hover {
    cursor: pointer;
    background: $color_s_li;
}
</style>
