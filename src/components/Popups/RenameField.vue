<template>
    <div class="renameBox">
        <div class="renameContainer flex_c_h flex_c gap1">
            <div class="ErrorMsg">{{ store.ErrorMessage }}</div>
            <Icons :icon="'exit'" :customClass="'exit'" @triggered="Exit" />
            <input v-model="typedName" type="text" placeholder="Type here a name" />
            <Icons :icon="'check'" :customClass="'icon'" @triggered="setName" />
        </div>
        <BlurBG />
    </div>
</template>
<script>
import BlurBG from "@/components/Assets/BlurBG.vue";
import Icons from "@/components/Assets/Icons.vue";
import { onKeyStroke } from "@vueuse/core";

export default {
    data() {
        return {
            typedName: "",
        };
    },
    components: {
        BlurBG,
        Icons,
    },
    computed: {
        store() {
            return this.$store.state;
        },
    },
    methods: {
        Exit() {
            this.$store.dispatch("setPopupActive", { active: false, type: "addTab" });
        },
        setName() {
            if (this.store.PopupActive.type === "addTab") {
                this.$store.dispatch("setPopupActive", { active: false, type: "addTab" });
            }
            this.$store.dispatch("setRenameContent", { name: this.typedName });
        },
    },
    mounted() {
        onKeyStroke("Enter", () => {
            this.setName();
        });
        onKeyStroke("Escape", () => {
            this.Exit();
        });
    },
};
</script>
<style lang=""></style>
