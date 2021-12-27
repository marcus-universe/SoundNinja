import { ipcMain } from "electron";
import fs from "fs";
import DatabaseManager from "./DatabaseManager";
import FileManager from "./FileManager";

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

// Save files
ipcMain.on("saveFile", async (_event, file, tab) => {
    const destination = await FileManager.getDestination(tab, file.name);
    if (!destination) return;
    FileManager.copyFile(file.path, destination)
        .then(() => {
            console.log(file.name + " saved");
        })
        .catch((err) => {
            console.error(err);
        });

    databaseManager.addFile(tab, file);
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
