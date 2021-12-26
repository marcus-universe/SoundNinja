import { ipcMain } from "electron";
import fs from "fs";

ipcMain.handle("getColor", () => {
    return new Promise((resolve, reject) => {
        fs.readFile("./config.json", "utf8", (err, data) => {
            if (err) reject(err);
            const config = JSON.parse(data);
            const color = config.settings[0].color;
            try {
                resolve(color);
            } catch (e) {
                reject(e);
            }
        });
    });
});
