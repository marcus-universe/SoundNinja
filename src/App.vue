<template>
<Topbar />

<div class="flex-c-w max_h ContentSection">
    <div class="flex_c_h sidepanel_w">
        <Sidemenu />
    </div>

    <About />
    <Settings />

    <div class="flex_c_h Soundpad">

        <Tabwrapper :tabList="tabList">
            <Tab
                :soundList="soundList"
                :tabList="tabList" />

        </Tabwrapper>

    </div>
</div>
</template>

<style lang="sass" src="./SASS/style.sass"></style>

<script>
import Topbar from './components/Topbar.vue';
import Sidemenu from './components/Sidemenu.vue';
import About from './components/About.vue';
import Settings from './components/Settings.vue';

// import axios from 'axios';

const fs = require('fs');
let root = document.documentElement;

import db from '../db.json';

import Tabwrapper from './components/TabwrapperContainer.vue';
import Tab from './components/Tab.vue';

export default {
    name: 'SoundNinja',
    methods: {

    },
    data() {
        return {
            tabList: [],
            soundList: [],
        }
    },
    components: {
        Topbar,
        Sidemenu,
        About,
        Settings,
        Tabwrapper,
        Tab
    },
    //get all names of tabs in db.json
    created() {
        this.tabList = [];
            for (let i = 0; i < db.folder.length; i++) {
                this.tabList.push(db.folder[i]);
                for (let j = 0; j < db.folder[i].audiofiles.length; j++) {
                    this.soundList.push(db.folder[i].audiofiles[j]);
                }
            }
    },

}

function setColorTheme(color) {
    root.style.setProperty('--primary_color', color);

}

fs.readFile('./config.json', 'utf-8', function (err, data) {
    if (err) throw err;
    var config = JSON.parse(data);
    var color = config.settings[0].color
    setColorTheme(color)

});

</script>
