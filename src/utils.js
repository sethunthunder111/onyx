import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import path from 'path';

const require = createRequire(import.meta.url);
const ffmpegPath = require('ffmpeg-static');

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Get project root (one level up from src)
const projectRoot = path.join(__dirname, '..');

export {
    require,
    ffmpegPath,
    __dirname,
    projectRoot
};
