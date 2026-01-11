import { require } from './utils.js';
import chalk from 'chalk';
import gradient from 'gradient-string';

import boxen from 'boxen';
import inquirer from 'inquirer';

// Static ASCII art for ONYX to avoid figlet runtime issues
const ONYX_BANNER = `
 ██████╗ ███╗   ██╗██╗   ██╗██╗  ██╗
██╔═══██╗████╗  ██║╚██╗ ██╔╝╚██╗██╔╝
██║   ██║██╔██╗ ██║ ╚████╔╝  ╚███╔╝ 
██║   ██║██║╚██╗██║  ╚██╔╝   ██╔██╗ 
╚██████╔╝██║ ╚████║   ██║   ██╔╝ ██╗
 ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝
`;

export function displayBanner() {
    console.clear();
    const banner = gradient.pastel.multiline(ONYX_BANNER);
    
    console.log(boxen(banner, {
        padding: 1,
        margin: 1,
        borderStyle: 'round',
        borderColor: 'cyan',
        title: 'The Ultimate YouTube Downloader',
        titleAlignment: 'center'
    }));
}

export async function showMainMenu() {
    const { action } = await inquirer.prompt([
        {
            type: 'select',
            name: 'action',
            message: 'What would you like to do?',
            choices: [
                '🔍 Search & Download',
                '📥 Download Video (Best Quality)',
                '🎬 Download Video (Select Resolution)',
                '🎵 Download Audio (MP3/OGG/WAV)',
                '📦 Download Playlist',
                '🖼️  Download Thumbnail',
                new inquirer.Separator(chalk.gray('─'.repeat(40))),
                '🌐 GUI Version',
                '⚙️  Settings',
                '🚪 Exit'
            ],
            pageSize: 15
        }
    ]);
    return action;
}

export async function promptUrl() {
    const { url } = await inquirer.prompt([
        {
            type: 'input',
            name: 'url',
            message: 'Enter the YouTube URL:',
            validate: input => input.length > 0 ? true : 'Please enter a valid URL'
        }
    ]);
    return url;
}

export async function promptSearchQuery() {
    const { query } = await inquirer.prompt([
        {
            type: 'input',
            name: 'query',
            message: 'Enter search query:',
            validate: input => input.length > 0 ? true : 'Please enter a query'
        }
    ]);
    return query;
}

export async function selectSearchResult(results) {
    const choices = results.map(r => ({
        name: `${chalk.bold(r.title)} ${chalk.gray(`(${r.duration_string || 'N/A'})`)} - ${chalk.cyan(r.uploader)}`,
        value: r
    }));

    const { selected } = await inquirer.prompt([
        {
            type: 'select',
            name: 'selected',
            message: 'Select a video to download:',
            choices: [...choices, new inquirer.Separator(chalk.gray('─'.repeat(20))), 'Cancel'],
            pageSize: 10
        }
    ]);
    return selected;
}

export async function selectResolution(formats) {
    // Filter for video formats
    // This logic needs to be robust. 
    // We want to group by resolution (height) and FPS.
    
    // 1. Get unique heights
    const uniqueResolutions = [...new Set(formats
        .filter(f => f.vcodec !== 'none' && f.height)
        .map(f => f.height)
    )].sort((a, b) => b - a); // Descending

    const choices = uniqueResolutions.map(height => {
        // Find best format for this height to check FPS if possible, 
        // but formats is a list of all formats.
        // Let's check if any format with this height has fps > 30
        const hasHighFps = formats.some(f => f.height === height && f.fps > 30);
        
        let name = `${height}p`;
        if (hasHighFps) {
            name += ` ${chalk.yellow('60fps')}`;
        }

        // Highlight high resolutions (>= 1080p)
        if (height >= 1080) {
            name = chalk.red(name);
        }

        return {
            name: name,
            value: height
        };
    });

    // Add separator and Audio Only option
    choices.push(new inquirer.Separator(chalk.gray('─'.repeat(20))));
    choices.push({ name: '🎵 Audio Only (Best Quality)', value: 'audio' });

    const { resolution } = await inquirer.prompt([
        {
            type: 'select',
            name: 'resolution',
            message: 'Select Resolution / Format:',
            choices: choices
        }
    ]);
    return resolution;
}

export async function selectAudioFormat() {
    const { format } = await inquirer.prompt([
        {
            type: 'select',
            name: 'format',
            message: 'Select Audio Format:',
            choices: ['mp3', 'ogg', 'wav', 'm4a']
        }
    ]);
    return format;
}

export async function promptPlaylistQuality() {
    const choices = [
        { name: `🎵 Audio Only (${chalk.green('MP3 Best Quality')})`, value: 'audio' },
        new inquirer.Separator(chalk.gray('─'.repeat(20))),
        { name: `🎬 Video - ${chalk.red('MAX')} (Best Available)`, value: 'max' },
        { name: `🎬 Video - ${chalk.yellow('Mid-MAX')} (1080p/720p)`, value: 'mid-max' },
        new inquirer.Separator(chalk.gray('─'.repeat(20))),
        { name: '🎬 Video - 360p', value: '360' },
        { name: '🎬 Video - 240p', value: '240' },
        { name: '🎬 Video - 144p', value: '144' }
    ];

    const { quality } = await inquirer.prompt([
        {
            type: 'select',
            name: 'quality',
            message: 'Select Playlist Format/Quality:',
            choices: choices
        }
    ]);
    return quality;
}

export async function showSettingsMenu(currentConfig) {
    const choices = [
        `📂 Download Path: ${chalk.cyan(currentConfig.downloadPath)}`,
        `🐛 Show Debug Command: ${currentConfig.showDebugCommand ? chalk.green('ON') : chalk.red('OFF')}`,
        new inquirer.Separator(chalk.gray('─'.repeat(40))),
        '🔙 Back to Main Menu'
    ];

    const { setting } = await inquirer.prompt([
        {
            type: 'select',
            name: 'setting',
            message: 'Settings:',
            choices: choices
        }
    ]);

    return setting;
}

export async function promptDownloadPath(currentPath) {
    const { path } = await inquirer.prompt([
        {
            type: 'input',
            name: 'path',
            message: 'Enter new download path:',
            default: currentPath,
            validate: input => input.length > 0 ? true : 'Please enter a valid path'
        }
    ]);
    return path;
}

export async function promptConcurrencyLimit() {
    const { limit } = await inquirer.prompt([
        {
            type: 'input',
            name: 'limit',
            message: 'Enter number of parallel downloads (1-10):',
            default: '3',
            validate: input => {
                const num = parseInt(input);
                if (isNaN(num) || num < 1 || num > 10) return 'Please enter a number between 1 and 10';
                return true;
            }
        }
    ]);
    return parseInt(limit);
}

export class PlaylistProgress {
    constructor(total) {
        this.total = total;
        this.completed = 0;
        this.active = new Map(); // id -> { title, percent, speed, eta }
        this.startTime = Date.now();
    }

    update(id, title, stats) {
        this.active.set(id, { title, ...stats });
        this.render();
    }

    complete(id) {
        this.active.delete(id);
        this.completed++;
        this.render();
    }

    fail(id) {
        this.active.delete(id);
        this.completed++;
        this.render();
    }

    render() {
        console.clear();
        
        // Header
        const percentTotal = Math.round((this.completed / this.total) * 100);
        console.log(boxen(
            chalk.bold(`Playlist Download in Progress\n`) +
            `Completed: ${this.completed}/${this.total} (${percentTotal}%)\n` +
            `Active Downloads: ${this.active.size}`,
            { padding: 1, borderStyle: 'round', borderColor: 'cyan' }
        ));

        // Active Downloads
        if (this.active.size > 0) {
            console.log(chalk.bold('Currently Downloading:'));
            console.log(chalk.gray('─'.repeat(50)));
            
            this.active.forEach((data, id) => {
                const percent = data.percent || 0;
                const speed = data.speed || 'Waiting...';
                const eta = data.eta || '--:--';
                
                // Truncate title
                let title = data.title;
                if (title.length > 40) title = title.substring(0, 37) + '...';
                
                // Progress Bar
                const barWidth = 20;
                const filled = Math.round((percent / 100) * barWidth);
                const bar = '█'.repeat(filled) + chalk.gray('░'.repeat(barWidth - filled));
                
                console.log(`${chalk.whiteBright(title)}`);
                console.log(`${chalk.cyan(bar)} ${chalk.yellow(Math.round(percent))}%`);
                console.log(`${chalk.gray('Speed:')} ${chalk.green(speed)} | ${chalk.gray('ETA:')} ${chalk.magenta(eta)}`);
                console.log(''); // Empty line
            });
        }
    }
}

