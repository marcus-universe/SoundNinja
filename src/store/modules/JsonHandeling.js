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
            tabList: [],
            settings: {
                hue: 0,
                outputSource: "default",
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

        setOutSource(state, payload) {
            state.configFile.settings.outputSource = payload;
        },

        addFiles(state, payload) {
            state.configFile.files = [...state.configFile.files, ...payload];
        },

        setActiveSound(state, { soundindex, status }) {
            state.configFile.files[soundindex].active = status;
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
            state.configFile = {
                tabList: [],
                settings: {
                    hue: 189,
                    outputSource: "default",
                },
                files: [],
            };
        },
        updateConfig(state, updatedConfig) {
            state.configFile = updatedConfig;
        },

        ReturnStatusAll(state) {
            state.configFile.files.forEach((file) => {
                file.active = false;
            });
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

        setOutSource({ commit }, val) {
            commit("setOutSource", val);
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
        setActiveSound({ commit, dispatch }, { soundindex, status }) {
            try {
                commit("setActiveSound", { soundindex, status });
                commit("writeConfig");
            } catch (e) {
                dispatch("setErrorActive", e);
            }
        },
        ReturnStatusAll({ commit, dispatch }) {
            try {
                commit("ReturnStatusAll");
                commit("writeConfig");
            } catch (e) {
                dispatch("setErrorActive", e);
            }
        },
    },
};
