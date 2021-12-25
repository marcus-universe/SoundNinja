<template>
<div
    class="Tab flex_c_h addtab"
    >
    {{tab.name}}
</div>
</template>

<script>
export default {
    name: 'TabButton',
    data() {
        return {
            isActive: false,
        }
    },
    props: {
        tab: {
            type: Object,
            required: false
        },
        tabList: {
            type: Array,
            required: false
        },
        selectedTab: {
            type: Array,
            required: false
        },
        TabActive: {
            type: Boolean,
            required: false
        }

    },
    methods: {
    },

}
</script>
