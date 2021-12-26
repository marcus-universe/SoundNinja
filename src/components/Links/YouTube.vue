<template>
    <li class="flex-c-w">
        <div class="flex-center">
            <img src="../../Assets/IMG/social_icons/Youtube.svg" alt="YouTube Icon" />
            <a
                @click="clickLink"
                data="https://youtube.com/user/MarcusTutorials1"
                href=""
                title="YouTube"
                >YouTube</a
            >
        </div>
    </li>
</template>
<script>
export default {
    name: "YouTube",
    methods: {
        clickLink(e) {
            this.$emit("clickLink", e);
        },
    },
};
</script>
