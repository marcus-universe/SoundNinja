<template>
<div
    class="SoundButton grid_c"
    :data-attribute="sound.audiourl"
    :class="{'SoundButtonActive': clicked}"
    @click="playSound(sound.audiourl)"
    >

    {{sound.audioname.replaceAll('.mp3', '').replaceAll('.wav', '').replaceAll('.ogg', '')}}

</div>
</template>

<script>
import {Howl} from 'howler';


export default {
    name: 'SoundButton',
    data() {
        return {
            clicked: false,
            playingsound: null,
        }
    },
    props: [
        'sound'
    ],
    methods: {
        playSound(url) {
            this.clicked = !this.clicked;
            
            const contextmenu = document.querySelector('.contextmenuTab')
            contextmenu.style.display = 'none';

            console.log('playSound: ' + url);
            if (this.clicked == true) {
                this.playingsound = new Howl({
                    src: [url],
                    autoplay: false,
                    loop: false,
                    volume: 0.5,
                    html5: true,
                    preload: true,
                    onend: function() {
                        this.clicked = false;
                        console.log('Finished!');
                        console.log(this.clicked);
                        

                    }.bind(this)
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
        }
    }
}
</script>
