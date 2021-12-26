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
}
