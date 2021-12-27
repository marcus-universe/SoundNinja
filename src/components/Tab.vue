<template>
    <div id="SoundButton_Container" class="flex-c-w SoundButton_Container">
        <draggable
            style="width: 100%"
            v-model="list"
            :data="list"
            @start="drag = true"
            @end="dragEnd"
            item-key="index"
            class="SoundButtonContainBox"
        >
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
            tabindex: 0,
            drag: false,
            tab: "",
            edited: false,
            data: { folder: [] },
            list: [],
        };
    },
    computed: {
        computedList: {
            get() {
                return this.tab.audiofiles;
            },
            set(value) {
                this.updateList();
                this.tab.audiofiles = value;
            },
        },
    },

    components: {
        SoundButton,
        draggable,
    },
    props: {
        tabList: {
            type: Array,
            default() {
                return [];
            },
        },
        soundList: {
            type: Array,
            default() {
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
            console.log(this.tabindex);
        },
        dragEnd() {
            this.drag = false;
        },
        updateList() {
            this.edited = false;

            ipcRenderer.invoke("getData").then((data) => {
                if (!data) return;

                let list = JSON.parse(JSON.stringify(this.list));
                data.folder[this.tabindex].audiofiles = list;
                ipcRenderer.invoke("setData", data);
            });
        },
    },
    watch: {
        selectedTab(val) {
            console.log("tab update");
            if (!val) return;
            const tab = Array.from(this.data.folder).find((tab) => {
                console.log({ val: val.foldername }, { tab: tab.foldername });
                return tab.foldername === val.foldername;
            });
            console.log({ tab });
            if (tab) {
                console.log(tab.audiofiles.length);
                console.log({ files: tab.audiofiles });
                this.list = Array.from(tab.audiofiles);
                console.log(this.list.map((file) => file.audioname));
                this.list.length = 0;
                this.list.push(...tab.audiofiles);
            } else {
                this.list = [];
                this.list.length = 0;
            }
        },
    },
    async mounted() {
        const data = await ipcRenderer.invoke("getData");
        this.data = data;
        const tab = data?.folder[this.tabindex];
        this.list = tab.audiofiles;
    },
};
</script>
