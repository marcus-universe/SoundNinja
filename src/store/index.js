import { createStore } from "vuex";

import JsonHandeling from "./modules/JsonHandeling";

export default createStore({
  state: {
    navbar: ["upload", "folder", "reset", "settings", "about"],
    currentTab: "All",
    PopupActive: false,
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
    PopupActive(state, payload) {
      state.PopupActive = payload;
    },
    RenameContent(state, { name, popup }) {
      if ((name === undefined, name === null, name === "")) {
        state.ErrorMessage = "Field is empty";
      } else {
        state.RenameContent = name;
        state.PopupActive = popup;
        state.ErrorMessage = "";
      }
    },

    ErrorActive(state, payload) {
      state.ErrorActive = true;
      state.ErrorMessage = payload;
    },
    SearchOpen(state, payload) {
      state.Searchbar.SearchbarActive = payload;
    },

    // writeJsonFileFn(state, payload) {

    //       writeTextFile({
    //             path: 'config.json',
    //             contents: JSON.stringify(payload, null, 2)
    //         }, {
    //             dir: BaseDirectory.App
    //       });
    //       console.log("write Json with" + payload)
    // },

    // readJsonFileFn(state) {
    //   async function readJson(state){
    //     state.JSONFile = JSON.parse(await readTextFile('config.json', { dir: BaseDirectory.App }));
    //   }
    //   readJson(state)

    // }
  },

  actions: {
    setPopupActive({ commit }, val) {
      commit("PopupActive", val);
    },
    setRenameContent({ commit }, { name, popup }) {
      commit("RenameContent", { name, popup });
    },
    setErrorActive({ commit }, val) {
      commit("ErrorActive", val);
    },
    setSearchOpen({ commit }, val) {
      commit("SearchOpen", val);
    },

    // writeJsonFile({ commit, dispatch }, val) {
    //   commit('writeJsonFileFn', val);
    //   dispatch('setSoundsContent', val)
    // },
    // readJsonFile({ commit, dispatch }) {
    //   try {
    //     commit('readJsonFileFn');
    //   } catch (e){
    //     dispatch('setErrorActive', e)
    //     dispatch('writeJsonFile', { "tabs": undefined})
    //   }

    // }
  },
  modules: {
    JsonHandeling,
  },
});
