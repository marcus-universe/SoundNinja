import { ipcMain } from "electron";
import ConfigManager from "./ConfigManager";
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

    databaseManager.addFile(tab, { audiopath: destination, audioname: file.name });
});

const manager = new ConfigManager();

ipcMain.handle("getColor", async () => {
    return (await manager.get()).settings[0].color;
});

ipcMain.handle("getConfig", () => {
    return manager.get();
});

ipcMain.handle("setConfig", (_event, config) => {
    return manager.set(config);
});

ipcMain.handle("mergeConfig", (_event, config) => {
    if (typeof config !== "object") return;
    return manager.merge(config);
});
