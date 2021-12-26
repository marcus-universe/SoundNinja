<template>
    <li class="flex-c-w">
        <div class="flex-center">
            <img src="../../Assets/IMG/social_icons/Instagram.svg" alt="Instagram Icon" />
            <a
                @click="clickLink"
                data="https://instagram.com/marcustutorials"
                href=""
                title="Instagram"
                >Instagram</a
            >
        </div>
    </li>
</template>
<script>
export default {
    name: "Instagram",
    methods: {
        clickLink(e) {
            this.$emit("clickLink", e);
        },
    },
};
</script>
