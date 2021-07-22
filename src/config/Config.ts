import Defaults from "./Defaults";
import json from "../helpers/json";
import fs from "fs";
import path from "path";
import { Yin } from "Yin";

const configLocation = "../../config.json";
export class Config {
    private yin: Yin;
    private config: Object;
    constructor(yin: Yin) {
        this.yin = yin;
        this.updateConfig();
    }

    async updateConfig() {
        let config: Object;

        try {
            console.log("File exists");
            config = require(configLocation);
        } catch (e) {
            // this.yin.log.warn("Config was invalid json or didn't exist. Creating new file from defaults.");
            console.log("erred");
            config = {};
        }

        // Put values into an object and ignore paths that are a number
        // ^ Ignore number due to how enums work
        let defaults: Record<string, any> = {};
        Object.entries(Defaults)
            .filter((k) => isNaN(Number(k[0])))
            .forEach((prop) => (defaults[prop[0]] = prop[1]));

        // Loop over all the default values and ensure they are set in the current config.
        for (const [key, value] of Object.entries(defaults)) {
            const currentValue = json.getPath(config, key);
            if (currentValue || currentValue === "" || currentValue === 0 || typeof currentValue === "boolean") continue;
            // this.yin.log.warn(`${key} was not set. setting default ${value}`);
            config = json.setPath(config, key, value);
        }

        // Write edited json to file
        fs.writeFileSync(path.resolve(__dirname, configLocation), JSON.stringify(config, null, 4), "utf-8");
        this.config = config;
    }
}
