import fs from "fs";
import { join } from "path";
import DatabaseManager from "./DatabaseManager";

const databaseManager = new DatabaseManager();

export default class FileManager {
    static async getDestination(tab, fileName) {
        const db = await databaseManager.get();
        const folders = db.folder;

        // Find the folder that matches the tab, or pick the last one
        const dest = folders.find((folder) => folder.name === tab) || folders[folders.length - 1];
        if (!dest) return;

        // Return path where the file will be saved
        return join(dest.path, fileName);
    }
    static copyFile(origin, dest) {
        return new Promise((resolve, reject) => {
            fs.copyFile(origin, dest, (err) => {
                if (err) reject(err);
                resolve();
            });
        });
    }
}
