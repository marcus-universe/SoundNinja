import fs from "fs";

export default class DatabaseManager {
    get() {
        return new Promise((resolve, reject) => {
            fs.readFile("./db.json", "utf8", (err, data) => {
                if (err) reject(err);
                resolve(JSON.parse(data));
            });
        });
    }
    set(data) {
        return new Promise((resolve, reject) => {
            fs.writeFile("./db.json", JSON.stringify(data), "utf8", (err) => {
                if (err) reject(err);
                resolve();
            });
        });
    }
    async merge(data) {
        let db = await this.get();
        db = Object.assign(db, data);
        await this.set(db);
    }

    async addFile(tab, file) {
        let db = await this.get();
        const folder = db.folder.find((folder) => folder.name === tab);
        if (!folder) return;
        const { audiofiles } = folder;
        if (this.fileExists(audiofiles, file)) return;
        audiofiles.push(file);
        file.id = audiofiles.length - 1;
        await this.set(db);
        return db;
    }

    fileExists(audiofiles, file) {
        const fileExists = audiofiles.find((audio) => audio.audioname === file.audioname);
        if (!fileExists) return;
        return true;
    }
}
