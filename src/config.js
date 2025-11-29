import fs from 'fs';
import path from 'path';
import { projectRoot } from './utils.js';

const configPath = path.join(projectRoot, 'config.json');

const defaultConfig = {
    downloadPath: path.join(process.env.USERPROFILE || process.env.HOME, 'Downloads'),
    showDebugCommand: false
};

export function loadConfig() {
    if (!fs.existsSync(configPath)) {
        saveConfig(defaultConfig);
        return defaultConfig;
    }
    try {
        return { ...defaultConfig, ...JSON.parse(fs.readFileSync(configPath, 'utf-8')) };
    } catch (e) {
        return defaultConfig;
    }
}

export function saveConfig(config) {
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
}
