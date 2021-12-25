<template>
<ul class="contextmenuTab">
    <li>Rename</li>
    <li>Delete</li>
</ul>

<div
    v-on:wheel="changeScrolldirection"
    id="Tabs_Container"
    class="flex-c-w Tabs_Container">

    <div class="TabButtonBox">
        <div
            class="Tab flex_c_h addtab"
            @click="AllTabClick">All</div>
    </div>

    <div
        v-for="tab of tabList"
        :key="tab.id"
        class="TabButtonBox"
        :data-id="tab.id"
        :data-attribute="tab.foldername"
        v-on:click="selectTab(tab)"
        @contextmenu="contextTabs(tab, $event)">
        <TabButton
            :class="{'SoundButtonActive': TabClick}"
            :TabActive="TabActive"
            :tab="tab"
            :tabList="tabList"
            :selectedTab="selectedTab" />
    </div>
</div>

<slot />
</template>

<script>
import {
    ref,
    provide
} from 'vue'
import TabButton from './TabButton.vue'

export default {
    name: 'Tabwrapper',
    props: {
        tabList: {
            type: Array,
            default: function () {
                return [];
            }
        }
    },
    setup(props) {
        

        console.log(props.tabList);

        function checkSelectedTab() {
            try {
                var selectedTab = ref([props.tabList[0].foldername]);
                return selectedTab;
            } catch (error) {
                selectedTab = [];
                return selectedTab
            }
        }
        
        const selectedTab = checkSelectedTab();
        // function () {
        //     try {
        //         var Liste = ref([props.tabList[0].name]);
        //         return Liste
        //     } catch (error) {
        //         Liste = []
        //         return Liste
        //     }
        // }

        provide('selectedTab', selectedTab);
        return {
            selectedTab,
        }
    },

    data() {
        return {
            // selectedTab: '',
            TabClick: false,
            TabActive: false,
            ShowAllButtons: false,
            contextTab: {},
        }
    },
    methods: {

        changeScrolldirection(evt) {
            const TabsContainer = document.querySelector('#Tabs_Container')
            evt.preventDefault();
            TabsContainer.scrollLeft += evt.deltaY;
        },

        selectTab(tabName) {
            const TabElements = document.querySelectorAll('.TabButtonBox')
            const contextmenu = document.querySelector('.contextmenuTab')
            contextmenu.style.display = 'none';

            this.selectedTab = [tabName.foldername];

            TabElements.forEach((tab) => {
                if (this.selectedTab.includes(tab.dataset.attribute)) {
                    tab.firstChild.classList.add('SoundButtonActive');
                    this.TabActive = true;
                } else {
                    tab.firstChild.classList.remove('SoundButtonActive');
                    this.TabActive = false;
                }
            })

        },
        AllTabClick() {
            // const AllSounds = document.querySelectorAll('.ButtonTab')
            this.selectedTab = ['All'];
            // AllSounds.forEach((sound) => {
            //     sound.style.removeProperty('display');
            // })
            // this.ShowAllButtons = true;
        },

        contextTabs(tab, event) {
            console.log(tab);
            // this.preventDefault();
            const contextmenu = document.querySelector('.contextmenuTab')
            contextmenu.style.display = 'block';
            var x = event.clientX;
            var y = event.clientY;

            contextmenu.style.left = x + 'px';
            contextmenu.style.top = y + 'px';

            console.log(x, y);
        }

        // addTab(tab) {
        //     this.tabList.push(tab);
        // },
        // removeTab(tab) {
        //     this.tabList.splice(this.tabList.indexOf(tab), 1);
        // }

    },
    components: {
        TabButton
    },
    created() {

    }
}
</script>
