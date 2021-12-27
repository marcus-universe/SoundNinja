<template>
    <ul class="contextmenuTab">
        <li>Rename</li>
        <li>Delete</li>
    </ul>

    <div v-on:wheel="changeScrolldirection" id="Tabs_Container" class="flex-c-w Tabs_Container">
        <div class="TabButtonBox">
            <div class="Tab flex_c_h addtab" @click="AllTabClick">All</div>
        </div>

        <div
            v-for="tab of tabList"
            :key="tab.id"
            class="TabButtonBox"
            :data-id="tab.id"
            :data-attribute="tab.foldername"
            v-on:click="selectTab(tab)"
            @contextmenu="contextTabs(tab, $event)"
        >
            <TabButton
                :class="{ SoundButtonActive: TabClick }"
                :TabActive="TabActive"
                :tab="tab"
                :tabList="tabList"
            />
        </div>
    </div>

    <slot />
</template>

<script>
import { provide, ref } from "vue";
import TabButton from "./TabButton.vue";

export default {
    name: "Tabwrapper",
    props: {
        tabList: {
            type: Array,
            default: function () {
                return [];
            },
        },
    },
    emits: ["updateTab"],

    setup() {
        const selectedTab = ref("All");

        provide("selectedTab", selectedTab);

        return {
            selectedTab,
        };
    },

    data() {
        return {
            TabClick: false,
            TabActive: false,
            ShowAllButtons: false,
            contextTab: {},
        };
    },
    methods: {
        changeScrolldirection(evt) {
            const TabsContainer = document.querySelector("#Tabs_Container");
            evt.preventDefault();
            TabsContainer.scrollLeft += evt.deltaY;
        },

        selectTab(tab) {
            const tabName = tab.name;
            console.log({ tabName });

            const TabElements = document.querySelectorAll(".TabButtonBox");
            const contextmenu = document.querySelector(".contextmenuTab");
            contextmenu.style.display = "none";

            this.updateSelectedTab(tab);

            TabElements.forEach((tab) => {
                if (this.selectedTab === tab.dataset.attribute) {
                    tab.firstChild.classList.add("SoundButtonActive");
                    this.TabActive = true;
                } else {
                    tab.firstChild.classList.remove("SoundButtonActive");
                    this.TabActive = false;
                }
            });
        },
        AllTabClick() {
            this.updateSelectedTab("All");
        },

        contextTabs(tab, event) {
            console.log(tab);
            const contextmenu = document.querySelector(".contextmenuTab");
            contextmenu.style.display = "block";
            var x = event.clientX;
            var y = event.clientY;

            contextmenu.style.left = x + "px";
            contextmenu.style.top = y + "px";
        },
        updateSelectedTab(tab) {
            this.selectedTab = tab;
        },
    },
    components: {
        TabButton,
    },
};
</script>
