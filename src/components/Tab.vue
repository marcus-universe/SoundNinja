<template>
    <div id="SoundButton_Container" class="flex-c-w SoundButton_Container">
        <div
            v-for="(tab, index) of tabList"
            :key="index"
            class="flex_c_h ButtonTab"
            style="width: 100%"
            :title="tab.name"
            v-show="selectedTab.includes(tab.name)"
        >
            <div
                v-for="sound of tabList[index].audiofiles"
                :key="sound"
                class="SoundButtonContainBox"
            >
                <SoundButton :sound="sound" />
            </div>
        </div>
    </div>
</template>

<script>
import { inject } from "vue";
// import { TouchBarButton } from 'electron';
import SoundButton from "./SoundButton.vue";
export default {
    name: "Tab",
    setup() {
        const selectedTab = inject("selectedTab");
        return {
            selectedTab,
        };
    },
    data() {
        return {
            tabindex: 0,
        };
    },
    components: {
        SoundButton,
    },
    props: {
        tabList: {
            type: Array,
            default: function () {
                return [];
            },
        },
        soundList: {
            type: Array,
            default: function () {
                return [];
            },
        },
        ShowAllButtons: {
            type: Boolean,
            default: false,
        },
    },
    methods: {
        tabindexchange() {
            this.tabindex += 1;
        },
    },
    mounted() {
        // this.soundList = this.soundList;
    },
};
</script>
