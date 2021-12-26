import fs from "fs";

export default class ConfigManager {
    constructor() {
        this.config = {};
    }

    get() {
        return new Promise((resolve, reject) => {
            fs.readFile("./config.json", "utf8", (err, data) => {
                if (err) reject(err);
                const config = JSON.parse(data);
                console.log(config);
                try {
                    resolve(config);
                } catch (e) {
                    reject(e);
                }
            });
        });
    }

    set(config) {
        return new Promise((resolve, reject) => {
            fs.writeFile("./config.json", JSON.stringify(config), (err) => {
                if (err) reject(err);
                resolve();
            });
        });
    }

    async merge(config) {
        let data = await this.get();
        data = Object.assign(data, config);
        this.set(data);
    }
}
