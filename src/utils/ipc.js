import { ipcMain } from "electron";
import ConfigManager from "./ConfigManager";

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
