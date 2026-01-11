import {
  ensureBinary,
  searchVideos,
  getVideoInfo,
  getPlaylistVideos,
  download,
} from "./youtube.js";
import {
  displayBanner,
  showMainMenu,
  promptUrl,
  promptSearchQuery,
  selectSearchResult,
  selectResolution,
  selectAudioFormat,
  showSettingsMenu,
  promptDownloadPath,
  promptPlaylistQuality,
  promptConcurrencyLimit,
  PlaylistProgress
} from "./ui.js";
import { loadConfig, saveConfig } from "./config.js";
import { startServer } from "./server.js";
import chalk from "chalk";
import ora from "ora";

async function main() {
  let config = loadConfig();
  displayBanner();

  try {
    await ensureBinary();
  } catch (e) {
    console.error(chalk.red("Critical Error: Could not setup yt-dlp."));
    process.exit(1);
  }

  while (true) {
    const action = await showMainMenu();

    if (action.includes("Exit")) {
      console.log(chalk.green("Bye! 👋"));
      process.exit(0);
    }

    if (action.includes("GUI Version")) {
      startServer();
      // Keep the CLI running but maybe pause interaction or just loop back
      // For now, let's just loop back so they can still use CLI if they want
      // But the server log "GUI Server running..." will appear.
      continue;
    }

    if (action.includes("Settings")) {
      while (true) {
        const setting = await showSettingsMenu(config);
        if (setting.includes("Back")) break;

        if (setting.includes("Download Path")) {
          const newPath = await promptDownloadPath(config.downloadPath);
          config.downloadPath = newPath;
          saveConfig(config);
        } else if (setting.includes("Debug Command")) {
          config.showDebugCommand = !config.showDebugCommand;
          saveConfig(config);
        }
      }
      displayBanner();
      continue;
    }

    try {
      await handleAction(action, config);
    } catch (error) {
      console.error(chalk.red("\nAn error occurred:"), error.message);
      // Wait a bit so user can read error
      await new Promise((r) => setTimeout(r, 2000));
    }

    // Clear and show banner again for next loop
    displayBanner();
  }
}

async function handleAction(action, config) {
  const downloadOptions = {
    path: config.downloadPath,
    showDebugCommand: config.showDebugCommand,
  };

  let url;
  let title;

  if (action.includes("Search")) {
    const query = await promptSearchQuery();
    const results = await searchVideos(query);

    if (results.length === 0) {
      console.log(chalk.yellow("No results found."));
      return;
    }

    const selected = await selectSearchResult(results);
    if (selected === "Cancel") return;

    url = selected.url;
    title = selected.title;
  } else if (action.includes("Download")) {
    url = await promptUrl();
  }

  if (!url) return;

  // Determine download mode
  if (action.includes("Audio")) {
    let format = await selectAudioFormat();
    // Map 'ogg' to 'vorbis' for yt-dlp compatibility
    if (format === "ogg") {
      format = "vorbis";
    }
      await download(
      [
        url,
        "-x",
        "--audio-format",
        format,
        "--audio-quality",
        "0",
        "-N", "8",
        "-o",
        "%(title)s.%(ext)s",
      ],
      downloadOptions
    );
  } else if (action.includes("Thumbnail")) {
    await download(
      [url, "--write-thumbnail", "--skip-download", "-o", "%(title)s.%(ext)s"],
      downloadOptions
    );
  } else if (action.includes("Playlist")) {
    // Playlist logic
    const quality = await promptPlaylistQuality();
    const concurrency = await promptConcurrencyLimit();
    
    // 1. Fetch playlist items
    const videos = await getPlaylistVideos(url);
    if (!videos || videos.length === 0) {
        console.log(chalk.red('No videos found in playlist.'));
        return;
    }
    
    console.log(chalk.cyan(`Found ${videos.length} videos. Starting parallel download (Limit: ${concurrency})...`));
    
    // 2. Prepare base args
    const baseArgs = [];
    if (quality === "audio") {
      baseArgs.push("-x", "--audio-format", "mp3", "--audio-quality", "0");
    } else {
      let formatArg;
      if (quality === "max") {
        formatArg = "bestvideo+bestaudio/best";
      } else if (quality === "mid-max") {
        formatArg = "bestvideo[height<=1080]+bestaudio/best[height<=1080]";
      } else {
        formatArg = `bestvideo[height<=${quality}]+bestaudio/best[height<=${quality}]`;
      }
      baseArgs.push(
        "--format",
        formatArg,
        "--merge-output-format",
        "mp4",
        "--postprocessor-args",
        "merger+ffmpeg:-c:a aac"
      );
    }
    baseArgs.push("-N", "8"); // Speed boost per file

    // 3. Parallel Execution Loop
    const progressTracker = new PlaylistProgress(videos.length);
    progressTracker.render();

    // Simple concurrency implementation
    const executing = new Set();
    for (const video of videos) {
        const videoUrl = video.url || `https://www.youtube.com/watch?v=${video.id}`;
        const title = video.title;
        const videoId = video.id || videoUrl; // Unique ID
        
        // Construct args for this specific video
        // We use the playlist output template structure but applied to individual video
        const args = [
            videoUrl,
            ...baseArgs,
            "-o",
            `playlist/${video.playlist_title || 'playlist'}/%(title)s.%(ext)s`
        ];

        // Initial status update to show it's pending/starting
        progressTracker.update(videoId, title, { percent: 0, speed: 'Starting...', eta: '--:--' });

        const p = download(args, { 
            ...downloadOptions, 
            silent: true,
            onProgress: (stats) => {
                progressTracker.update(videoId, title, stats);
            }
        })
            .then(() => {
                progressTracker.complete(videoId);
            })
            .catch(err => {
                progressTracker.fail(videoId);
                // Optionally log error to file or show in summary?
                // For now, fail() just removes it from active list
            });
            
        // Wrap p to handle removal from the set
        const wrapper = p.then(() => executing.delete(wrapper));
        
        executing.add(wrapper);
        
        // If we reached the limit, wait for one to finish
        if (executing.size >= concurrency) {
            await Promise.race(executing);
        }
    }
    
    // Wait for remaining
    await Promise.all(executing);
    
  } else {
    // Video Download
    let formatArg = "bestvideo+bestaudio/best"; // Default Best

    if (action.includes("Select Resolution") || action.includes("Search")) {
      // For search results, we might want to offer resolution selection too
      const metadata = await getVideoInfo(url);
      const choice = await selectResolution(metadata.formats);

      if (choice === "audio") {
        // Switch to audio mode
        let format = await selectAudioFormat();
        if (format === "ogg") format = "vorbis";

        await download(
          [
            url,
            "-x",
            "--audio-format",
            format,
            "--audio-quality",
            "0",
            "-N", "8",
            "-o",
            "%(title)s.%(ext)s",
          ],
          downloadOptions
        );
        return;
      } else {
        // Specific resolution
        // We use height comparison
        formatArg = `bestvideo[height<=${choice}]+bestaudio/best[height<=${choice}]`;
      }
    }

    await download(
      [
        url,
        "-f",
        formatArg,
        "--merge-output-format",
        "mp4",
        "--postprocessor-args",
        "merger+ffmpeg:-c:a aac",
        "-N", "8",
        "-o",
        "%(title)s.%(ext)s",
      ],
      downloadOptions
    );
  }

  console.log(chalk.green("\nDone! Returning to menu..."));
  await new Promise((r) => setTimeout(r, 2000));
}

main();
