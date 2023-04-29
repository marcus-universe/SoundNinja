import { createStore } from "vuex";

import JsonHandeling from "./modules/JsonHandeling";

export default createStore({
    state: {
        navbar: ["upload", "folder", "reset", "settings", "about"],
        currentTab: "All",
        PopupActive: { active: false, type: "addTab" },
        RenameContent: "",
        ErrorMessage: "",
        ErrorActive: false,
        Searchbar: {
            SearchbarActive: false,
            SearchbarContent: "",
        },
        // Sounds: [],
        // JSONFile: null
    },
    getters: {},
    mutations: {
        PopupActive(state, { active, type }) {
            state.PopupActive.active = active;
            state.PopupActive.type = type;
        },
        RenameContent(state, { name }) {
            if ((name === undefined, name === null, name === "")) {
                state.ErrorMessage = "Field is empty";
            } else {
                state.RenameContent = name;
                state.ErrorMessage = "";
                if (state.PopupActive.type === "addTab") {
                    state.JsonHandeling.configFile.tabList.push(state.RenameContent);
                }
            }
        },

        ErrorActive(state, payload) {
            state.ErrorActive = true;
            state.ErrorMessage = payload;
        },
        SearchOpen(state, payload) {
            state.Searchbar.SearchbarActive = payload;
        },
        setCurrentTab(state, payload) {
            state.currentTab = payload;
        },
    },

    actions: {
        setPopupActive({ commit }, { active, type }) {
            commit("PopupActive", { active, type });
        },
        setRenameContent({ commit }, { name }) {
            commit("RenameContent", { name });
            commit("writeConfig");
        },
        setErrorActive({ commit }, val) {
            commit("ErrorActive", val);
        },
        setSearchOpen({ commit }, val) {
            commit("SearchOpen", val);
        },
        setCurrentTab({ commit }, val) {
            commit("setCurrentTab", val);
        },
    },
    modules: {
        JsonHandeling,
    },
});
