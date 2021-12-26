<template>
    <svg @click="onClick" class="Maximize" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
        <g id="Fullscreen">
            <path
                class="cls-1"
                d="M166.93,53.44v92.89H33.07V53.44H166.93m12-24H21.07a12,12,0,0,0-12,12V158.33a12,12,0,0,0,12,12H178.93a12,12,0,0,0,12-12V41.44a12,12,0,0,0-12-12Z"
            />
        </g>
    </svg>
</template>
<script>
export default {
    name: "Maximize",
    methods: {
        onClick() {
            this.$emit("onClick");
        },
    },
};
</script>
