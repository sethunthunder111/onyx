import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import path from 'path';
import fs from 'fs';

// Bun-friendly require
const require = createRequire(import.meta.url);

// In compiled binaries, process.execPath is the .exe
const binaryDir = path.dirname(process.execPath);
const localFfmpeg = path.join(binaryDir, 'ffmpeg.exe');

let ffmpegPath;
if (fs.existsSync(localFfmpeg)) {
    ffmpegPath = localFfmpeg;
} else {
    try {
        // This will only work in dev mode where node_modules exists
        ffmpegPath = require('ffmpeg-static');
    } catch (e) {
        // Fallback to system path
        ffmpegPath = 'ffmpeg';
    }
}

// In Bun compiled binaries, import.meta.url refers to a virtual path.
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Determine project root:
// 1. If we are in a 'src' folder, we are in dev mode.
// 2. Otherwise, we use the binary's directory.
let projectRoot;
if (__dirname.endsWith('src')) {
    projectRoot = path.join(__dirname, '..');
} else {
    projectRoot = binaryDir;
}

export {
    require,
    ffmpegPath,
    __dirname,
    projectRoot
};
