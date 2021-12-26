<template>
    <div id="SoundButton_Container" class="flex-c-w SoundButton_Container">
        <draggable
            style="width: 100%"
            :title="tab.name"
            v-show="selectedTab.includes(tab.name)"
            v-model="list"
            :group="tab.name"
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
import { inject } from "vue";
import SoundButton from "./SoundButton.vue";
import draggable from "vuedraggable";
import { ipcRenderer } from "electron";

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
            drag: false,
            tab: "",
            edited: false,
        };
    },
    computed: {
        list: {
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
        dragEnd(event) {
            this.drag = false;
            console.log({
                event,
            });
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
    async mounted() {
        const data = await ipcRenderer.invoke("getData");
        const tab = data?.folder[this.tabindex];
        this.tab = tab;
        this.list = tab.audiofiles;
    },
};
</script>
