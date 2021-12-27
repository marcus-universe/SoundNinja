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
        console.log(data);
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
        console.log(db);
        await this.set(db);
    }

    async addFile(tab, file) {
        console.log("Adding file");
        let db = await this.get();
        const folder = db.folder.find((folder) => folder.name === tab);
        if (!folder) return;
        if (this.fileExists(folder.audiofiles, file)) return;
        folder.audiofiles.push(file);
        file.id = folder.audiofiles.length - 1;
        await this.set(db);
    }

    fileExists(audiofiles, file) {
        console.log("Checking if file exists");
        const fileExists = audiofiles.find((audio) => audio.audioname === file.name);
        console.log({ fileExists, file });
        if (!fileExists) return;
        console.log("File exists");
        return true;
    }
}
