import { writeTextFile, BaseDirectory } from "@tauri-apps/api/fs";

// import config from "";
export default {
  state: {
    NewJsonData: {},
    FileStruct: [],
    TabList: [],
    Settings: {
      hue: 0,
    },
    JSONFile: null,
    path: null,
    configFile: {
      settings: {
        hue: 0,
      },
      files: [],
    },
  },

  getters: {
    getJsonFile(state) {
      return state.JSONFile;
    },
  },

  mutations: {
    updateConfigFile(state, updatedConfig) {
      state.configFile = updatedConfig;
    },

    writeConfig(state) {
      writeTextFile(
        {
          path: "config.json",
          contents: JSON.stringify(state.configFile, null, 2),
        },
        {
          dir: BaseDirectory.App,
        }
      );
    },

    HueSet(state, payload) {
      state.configFile.settings.hue = payload;
    },

    addFiles(state, payload) {
      state.configFile.files = [...state.configFile.files, ...payload];
    },

    SoundsContent(state, payload) {
      state.Sounds = "";
      state.Sounds = payload;
      //add to each sound a new property names active
      state.Sounds.tabs.forEach((tab) => {
        tab.sounds.forEach((sound) => {
          sound.active = false;
        });
      });
    },
    Reset(state) {
      state.FileStruct = [];
      state.TabList = [];
      state.Settings = {
        hue: 189,
      };
    },
    updateConfig(state, updatedConfig) {
      state.configFile = updatedConfig;
    },
  },

  actions: {
    writeJsonFile({ commit }) {
      commit("writeConfig");
    },

    readConfig({ commit, dispatch }) {
      try {
        commit("readConfig");
      } catch (e) {
        dispatch("setErrorActive", e);
      }
    },

    setSoundsContent({ commit }, val) {
      commit("SoundsContent", val);
    },
    addFiles({ commit }, val) {
      commit("addFiles", val);
      commit("writeConfig");
    },
    setHue({ commit }, val) {
      commit("HueSet", val);
      commit("writeConfig");
    },
    resetAll({ commit }) {
      commit("Reset");
      commit("writeConfig");
    },

    updateConfigFile({ commit, dispatch }, contents) {
      try {
        commit("updateConfig", contents);
        console.log(contents);
      } catch (e) {
        console.log(e);
        dispatch("setErrorActive", e);
      }
    },
  },
};
