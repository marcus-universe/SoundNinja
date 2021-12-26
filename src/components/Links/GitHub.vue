<template>
    <li class="flex-c-w">
        <div class="flex-center">
            <img src="../../Assets/IMG/social_icons/Github.svg" alt="Github Icon" />
            <a @click="clickLink" data="https://github.com/MarcusTutorials" href="" title="GitHub"
                >GitHub</a
            >
        </div>
    </li>
</template>
<script>
export default {
    name: "GitHub",
    methods: {
        clickLink(e) {
            this.$emit("clickLink", e);
        },
    },
};
</script>
