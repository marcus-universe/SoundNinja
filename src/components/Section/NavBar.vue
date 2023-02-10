<template>
  <div class="navbar flex_c_v">
    <div class="iconContainer flex_c_v flex_space_evenly gap1">
      <Icons
        :icon="'search'"
        :customClass="[
          'icon searchButton',
          { active: Searchbar.SearchbarActive },
        ]"
        @triggered="IconClicked"
      />
      <transition name="slideIn">
        <div
          v-if="Searchbar.SearchbarActive"
          class="searchBar flex_c_h align_c flex_start gap1"
        >
          <input type="text" placeholder="Search" />
          <Icons
            :icon="'check'"
            :customClass="'icon'"
            @triggered="IconClicked"
          />
        </div>
      </transition>

      <Icons
        v-for="(navelm, index) in navbarList"
        :key="navelm"
        :icon="navbarList[index]"
        @triggered="IconClicked"
      />
    </div>
  </div>
</template>

<script>
import { open } from "@tauri-apps/api/dialog";
import Icons from "@/components/Assets/Icons.vue";
export default {
  name: "NavBar",
  components: {
    Icons,
  },
  computed: {
    JSONFile() {
      return this.$store.state.JsonHandeling.JSONFile;
    },
    navbarList() {
      return this.$store.state.navbar;
    },
    currentTab() {
      return this.$store.state.currentTab;
    },
    Searchbar() {
      return this.$store.state.Searchbar;
    },
    FileStruct() {
      return this.$store.state.JsonHandeling.FileStruct;
    },
  },
  methods: {
    async uploadFiles() {
      // this.$store.dispatch("readJsonFile");
      var self = this;
      const selected = await open({
        multiple: true,
        title: "Select files to upload",
        filters: [
          {
            name: "Add Sounds",
            extensions: ["mp3", "wav", "ogg"],
          },
        ],
      });

      if (Array.isArray(selected)) {
        // const contents = this.JSONFile;

        const soundlist = [];
        selected.forEach((file) => {
          const tabs = ["All"];
          if (self.currentTab !== "All") {
            tabs.push(self.currentTab);
          }
          soundlist.push({
            name: file
              .replace(/^.*[\\]/, "")
              .replace(".wav", "")
              .replace(".mp3", "")
              .replace(".ogg", "")
              .replaceAll("_", " ")
              .replace(/([A-Z])/g, " $1")
              .trim(),
            path: file,
            volume: 0.4,
            tabs: tabs,
            active: false,
          });
        });

        try {
          self.$store.dispatch("addFiles", soundlist);
        } catch (err) {
          console.log(err);
        }
      }
    },

    async ResetAll() {
      this.$store.dispatch("resetAll");
    },

    OpenSearch() {
      this.$store.dispatch("setSearchOpen", !this.Searchbar.SearchbarActive);
    },

    IconClicked(icon) {
      if (icon === "upload") {
        this.uploadFiles();
      } else if (icon === "search") {
        this.OpenSearch();
      } else if (icon === "check") {
        console.log("check");
      } else if (icon === "reset") {
        this.ResetAll();
      } else if (icon === "settings") {
        if (this.$route.path !== "/settings") {
          this.$router.push("/settings");
        } else {
          this.$router.push("/");
        }
      }
    },
  },
};
</script>

<style lang="scss">
@import "@/sass/navbar.sass";
</style>
