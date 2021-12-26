import { ipcMain } from "electron";
import fs from "fs";
import DatabaseManager from "./DatabaseManager";

const databaseManager = new DatabaseManager();

// Database IPC
ipcMain.handle("getData", () => {
    return databaseManager.get();
});

ipcMain.handle("setData", (_event, data) => {
    return databaseManager.set(data);
});

ipcMain.handle("mergeData", (_event, data) => {
    return databaseManager.merge(data);
});

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
