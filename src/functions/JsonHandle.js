import {
    writeTextFile,
    readTextFile,
    BaseDirectory
} from '@tauri-apps/api/fs'

export default {
    install: function (app) {


        const readJsonFile = async () =>{
            try {
                var contents = JSON.parse(await readTextFile('config.json', {
                    dir: BaseDirectory.App
                }));
                return contents
            }catch (e) {
                this.$store.dispatch('setErrorActive', e)
                this.writeJsonFile({ "tabs": [] })
                window.location.reload();
            }   
        }

        
        function writeJsonFile(contents) {
            writeTextFile({
                path: 'config.json',
                contents: JSON.stringify(contents, null, 2)
            }, {
                dir: BaseDirectory.App
            });

            this.$store.dispatch('setSoundsContent', contents)
        }


        app.provide('readJsonFile', readJsonFile);

        app.provide('writeJsonFile', writeJsonFile);
    }
}