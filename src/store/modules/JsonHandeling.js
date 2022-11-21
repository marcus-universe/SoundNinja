import {
    writeTextFile,
    readTextFile,
    BaseDirectory
} from '@tauri-apps/api/fs'

export default {
    state: {
        Sounds: [],
        JSONFile: null
    },

    mutations: {
        writeJsonFileFn(state, payload) {

            writeTextFile({
                path: 'config.json',
                contents: JSON.stringify(payload, null, 2)
            }, {
                dir: BaseDirectory.App
            });
        },

        readJsonFileFn(state) {
            async function readJson(state) {
                state.JSONFile = JSON.parse(await readTextFile('config.json', {
                    dir: BaseDirectory.App
                }));
            }
            readJson(state)

        },

        SoundsContent(state, payload) {
            state.Sounds = "";
            state.Sounds = payload;
            //add to each sound a new property names active
            state.Sounds.tabs.forEach(tab => {
                tab.sounds.forEach(sound => {
                    sound.active = false;
                })
            })
        },



    },

    actions: {
        writeJsonFile({
            commit,
            dispatch
        }, val) {
            commit('writeJsonFileFn', val);
            dispatch('setSoundsContent', val)
        },
        readJsonFile({
            commit,
            dispatch
        }) {
            try {
                commit('readJsonFileFn');
            } catch (e) {
                dispatch('setErrorActive', e)
                dispatch('writeJsonFile', {
                    "tabs": undefined
                })
            }

        },

        setSoundsContent({
            commit
        }, val) {
            commit('SoundsContent', val);
        },

    }

}