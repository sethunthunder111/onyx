import YTDlpWrap from 'yt-dlp-wrap';
import { projectRoot, ffmpegPath } from './utils.js';
import path from 'path';
import fs from 'fs';
import ora from 'ora';
import chalk from 'chalk';

const binaryPath = path.join(projectRoot, 'yt-dlp.exe');
const ytDlpWrap = new (YTDlpWrap.default || YTDlpWrap)(binaryPath);

export async function ensureBinary() {
    if (!fs.existsSync(binaryPath)) {
        const spinner = ora('Downloading latest yt-dlp binary...').start();
        try {
            await YTDlpWrap.downloadFromGithub(binaryPath);
            spinner.succeed('yt-dlp binary downloaded successfully!');
        } catch (error) {
            spinner.fail('Failed to download yt-dlp binary.');
            throw error;
        }
    }
}

export async function searchVideos(query, limit = 10) {
    const spinner = ora(`Searching for "${query}"...`).start();
    try {
        // Use yt-dlp's internal search
        // We use --dump-json to get structured data
        // --flat-playlist to avoid downloading the whole playlist if search returns one
        const args = [
            `ytsearch${limit}:${query}`,
            '--dump-json',
            '--default-search', 'ytsearch',
            '--no-playlist',
            '--flat-playlist' 
        ];

        const stdout = await ytDlpWrap.execPromise(args);
        spinner.stop();

        // Output is line-delimited JSON
        const results = stdout.trim().split('\n').map(line => {
            try {
                return JSON.parse(line);
            } catch (e) {
                return null;
            }
        }).filter(Boolean);

        return results;
    } catch (error) {
        spinner.fail('Search failed');
        throw error;
    }
}

export async function getVideoInfo(url) {
    const spinner = ora('Fetching video metadata...').start();
    try {
        const metadata = await ytDlpWrap.getVideoInfo(url);
        spinner.stop();
        return metadata;
    } catch (error) {
        spinner.fail('Failed to fetch metadata');
        throw error;
    }
}

export async function getPlaylistVideos(url) {
    const spinner = ora('Fetching playlist videos...').start();
    try {
        const args = [
            '--flat-playlist',
            '--dump-json',
            url
        ];
        const stdout = await ytDlpWrap.execPromise(args);
        spinner.stop();

        return stdout.trim().split('\n').map(line => {
            try {
                return JSON.parse(line);
            } catch (e) {
                return null;
            }
        }).filter(Boolean);
    } catch (error) {
        spinner.fail('Failed to fetch playlist videos');
        throw error;
    }
}

export function download(args, options = {}) {
    // Add ffmpeg location to args
    args.push('--ffmpeg-location', ffmpegPath);

    if (options.path) {
        // Ensure directory exists
        if (!fs.existsSync(options.path)) {
            fs.mkdirSync(options.path, { recursive: true });
        }
        // Modify output template to include path
        // We need to find the -o argument and prepend the path
        const outputIndex = args.indexOf('-o');
        if (outputIndex !== -1) {
            args[outputIndex + 1] = path.join(options.path, args[outputIndex + 1]);
        } else {
            args.push('-o', path.join(options.path, '%(title)s.%(ext)s'));
        }
    }

    if (options.showDebugCommand) {
        console.log(chalk.gray(`\nRunning: yt-dlp ${args.join(' ')}\n`));
    }
    
    let downloadSpinner;
    if (!options.silent) {
        downloadSpinner = ora('Starting download...').start();
    }
    
    let progress = 0;
    let speed = 'N/A';
    let eta = 'N/A';
    
    const eventEmitter = ytDlpWrap.exec(args);

    eventEmitter.on('ytDlpEvent', (eventType, eventData) => {
        if (eventType === 'download') {
            // Custom parsing because yt-dlp-wrap sometimes fails to parse speed/eta
            // Example: [download]  98.8% of ~ 280.90KiB at 173.13KiB/s ETA 00:00
            
            const percentMatch = eventData.match(/(\d+(?:\.\d+)?)%/);
            const speedMatch = eventData.match(/at\s+(\S+)/);
            const etaMatch = eventData.match(/ETA\s+(\S+)/);

            if (percentMatch) progress = parseFloat(percentMatch[1]);
            if (speedMatch) speed = speedMatch[1];
            if (etaMatch) eta = etaMatch[1];

            if (downloadSpinner) {
                downloadSpinner.text = `Downloading... ${progress}% | Speed: ${speed} | ETA: ${eta}`;
            }
            
            if (options.onProgress) {
                options.onProgress({ percent: progress, speed, eta });
            }
        }
    });

    eventEmitter.on('progress', (p) => {
        // Fallback or additional updates if needed, but custom parsing above is more robust for text output
        // We can ignore this or use it to fill gaps if custom parsing fails
        if (!speed || speed === 'N/A') {
             if (p.currentSpeed) speed = p.currentSpeed;
        }
        if (!eta || eta === 'N/A') {
             if (p.eta) eta = p.eta;
        }
    });

    return new Promise((resolve, reject) => {
        eventEmitter.on('error', (error) => {
            if (downloadSpinner) downloadSpinner.fail('Download failed!');
            // console.error(chalk.red(error.message)); // Suppress automatic error logging in silent mode or let caller handle? 
            // Better to keep it unless extremely verbose
            if (!options.silent) console.error(chalk.red(error.message));
            reject(error);
        });

        eventEmitter.on('close', () => {
            if (downloadSpinner) downloadSpinner.succeed('Download complete! 🎉');
            resolve();
        });
    });
}
