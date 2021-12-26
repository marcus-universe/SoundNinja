<template>
    <svg @click="onClick" class="ExitApp" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200">
        <g id="Exit">
            <path
                class="cls-1"
                d="M122.39,100.06l53.3-53.31a7,7,0,0,0,0-9.85L163.16,24.37a7,7,0,0,0-9.86,0L100,77.67,46.7,24.37a7,7,0,0,0-9.86,0L24.31,36.9a7,7,0,0,0,0,9.85l53.3,53.31-53.3,53.3a7,7,0,0,0,0,9.86l12.53,12.53a7,7,0,0,0,9.86,0L100,122.44l53.3,53.31a7,7,0,0,0,9.86,0l12.53-12.53a7,7,0,0,0,0-9.86Z"
            />
        </g>
    </svg>
</template>
<script>
export default {
    name: "ExitApp",
    methods: {
        onClick() {
            this.$emit("onClick");
        },
    },
};
</script>
