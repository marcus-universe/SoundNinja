<template>
    <svg @click="onClick" class="HideApp" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
        <g id="Fullscreen">
            <rect class="cls-1" x="9.07" y="139.37" width="181.85" height="30.96" rx="12" />
        </g>
    </svg>
</template>
<script>
export default {
    name: "Minimize",
    methods: {
        onClick() {
            this.$emit("onClick");
        },
    },
};
</script>
