<template>
<div class="SoundContainer">
    <Suspense>
        <div
            class="SoundTab flex_c_h flex_start gap1 flex_wrap"
            v-for="(Tab, tabindex) in Sounds.tabs"
            :key="Tab">
            <div
                v-for="(sound, soundindex) in Tab.sounds"
                class="Soundbtn flex_c_v flex_wrap"
                :class="{'active': soundindex === activeSounds}"
                :key="sound"
                @click="setActiveSound(soundindex, tabindex)">
                {{sound.name}}
            </div>
        </div>

        <template #fallback>
            <div>
                Is loading...
            </div>
        </template>
    </Suspense>

</div>
</template>

<script>
import {
    readTextFile,
    // writeTextFile,
    BaseDirectory
} from '@tauri-apps/api/fs'
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { Howl, Howler } from 'howler';

export default {

    computed: {
        Sounds() {
            return this.$store.state.JsonHandeling.Sounds
        },
        ErrorMessage() {
            return this.$store.state.ErrorMessage
        },
    },
    data() {
        return {
            activeSounds: null,
            soundPlaying: false
        }
    },
    methods: {
        // writeJsonFile(contents) {
        //     writeTextFile({
        //         path: 'config.json',
        //         contents: JSON.stringify(contents, null, 2)
        //     }, {
        //         dir: BaseDirectory.App
        //     });

        //     this.$store.dispatch('setSoundsContent', contents)
        // },

        async readJsonFile() {
            try {
                var contents = JSON.parse(await readTextFile('config.json', {
                    dir: BaseDirectory.App
                }));
                return contents
            }catch (e) {
                this.$store.dispatch('setErrorActive', e)

                this.$store.dispatch('writeJsonFile', { "tabs": undefined})
                window.location.reload();
            }   
        },
        activeSound(soundindex) {
            if (this.soundPlaying) {
                this.activeSounds = soundindex
            } else {
                this.activeSounds = null
            }
            
        },
        setActiveSound(soundindex, tabindex) {
            var self = this;
            this.soundPlaying = true
            
            this.activeSound(soundindex)
            var filepath = convertFileSrc(this.Sounds.tabs[tabindex].sounds[soundindex].path)
            var volume = this.Sounds.tabs[tabindex].sounds[soundindex].volume
            var sound = new Howl({
                src: [filepath],
                volume: volume,
                html5: true,
                onend: function () {
                    self.soundPlaying = false
                    self.activeSounds = null
                    console.log('Finished!');
                },
                onplayerror: function () {
                    self.soundPlaying = false
                    self.activeSounds = null
                    self.ErrorMessage = "Error: " + self.Sounds.tabs[tabindex].sounds[soundindex].name + " can't be played!"
                    
                }
            });
            Howler.stop()
            this.Sounds.tabs[tabindex].sounds[soundindex].active = !this.Sounds.tabs[tabindex].sounds[soundindex].active
            this.Sounds.tabs[tabindex].sounds?.forEach((sound, index) => {
                if (index !== soundindex) {
                    sound.active = false
                }
            })
            if (self.Sounds.tabs[tabindex].sounds[soundindex].active) {
                self.Sounds.tabs[tabindex].sounds[soundindex].active = true
                sound.once('load', function () {
                    sound.play();
                });
            } else {
                self.Sounds.tabs[tabindex].sounds[soundindex].active = false
                sound.stop();
                self.soundPlaying = false
                self.activeSounds = null
            }
            
        }
    },
    mounted() {
        try {
            
            this.readJsonFile().then((data) => {
                if(data !== undefined || data !== null || data !== "" || data !== {tabs: []}) {
                    this.$store.dispatch('setSoundsContent', data)
                }
            })
        } catch (error) {
            console.log(error)
            window.location.reload();
        }
    }
}
</script>

<style lang="">

</style>
