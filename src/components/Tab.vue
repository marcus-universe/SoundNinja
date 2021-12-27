<template>
    <div v-if="selectedTab === 'All'" class="flex-c-w SoundButton_Container">
        <div class="SoundButtonContainBox">
            <SoundButton
                :sound="element"
                v-for="element in completeList"
                v-bind:key="element.index"
            />
        </div>
    </div>
    <div v-else id="SoundButton_Container" class="flex-c-w SoundButton_Container">
        <draggable v-model="computedList" item-key="index" class="SoundButtonContainBox">
            <template #item="{ element }">
                <SoundButton :sound="element" />
            </template>
        </draggable>
    </div>
</template>

<script>
import SoundButton from "./SoundButton.vue";
import draggable from "vuedraggable";
import { ipcRenderer } from "electron";
import { inject } from "vue";

export default {
    name: "Tab",

    setup() {
        const selectedTab = inject("selectedTab");

        return { selectedTab };
    },
    data() {
        return {
            drag: false,
            data: { folder: [] },
            completeList: [],
            list: [],
        };
    },
    computed: {
        computedList: {
            get() {
                return this.list;
            },
            set(value) {
                this.list = value;
                this.updateList(value);
            },
        },
    },

    components: {
        SoundButton,
        draggable,
    },
    methods: {
        updateList(newList) {
            ipcRenderer.invoke("getData").then((data) => {
                if (!data) return;

                let list = JSON.parse(JSON.stringify(newList));
                const tab = data.folder.find((tab) => tab.name === this.selectedTab);
                if (!tab) return;
                tab.audiofiles = list;
                ipcRenderer.invoke("setData", data);
            });
        },
    },
    watch: {
        selectedTab(val) {
            if (!val) return;
            const tab = Array.from(this.data.folder).find((tab) => {
                return tab.foldername === val;
            });
            if (tab) {
                this.list = Array.from(tab.audiofiles);
            } else {
                this.list = [];
            }
        },
    },
    async mounted() {
        const data = await ipcRenderer.invoke("getData");
        this.data = data;
        const tab = data?.folder[this.selectedTab];
        this.list = tab?.audiofiles || [];
        this.completeList = data?.folder.reduce((acc, cur) => {
            return acc.concat(cur.audiofiles);
        }, []);
    },
};
</script>
