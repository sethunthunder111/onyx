import express from "express";
import { createServer } from "http";
import { Server } from "socket.io";
import open from "open";
import path from "path";
import { fileURLToPath } from "url";
import { searchVideos, getVideoInfo, download } from "./youtube.js";
import { loadConfig } from "./config.js";
import { projectRoot } from "./utils.js";
import chalk from "chalk";
import fs from "fs";

// Embed assets for the compiled binary
// These will be bundled into the EXE by Bun
import indexHtml from "../public/index.html" with { type: "text" };
import styleCss from "../public/css/style.css" with { type: "text" };
import appJs from "../public/js/app.js" with { type: "text" };

const app = express();
const httpServer = createServer(app);
const io = new Server(httpServer);

// 1. Try serving from physical 'public' folder (Dev mode)
const publicPath = path.join(projectRoot, "public");
if (fs.existsSync(publicPath)) {
  app.use(express.static(publicPath));
}

// 2. Fallbacks for compiled mode (Serving from embedded strings)
app.get("/", (req, res) => {
  res.setHeader("Content-Type", "text/html");
  res.send(indexHtml);
});

app.get("/css/style.css", (req, res) => {
  res.setHeader("Content-Type", "text/css");
  res.send(styleCss);
});

app.get("/js/app.js", (req, res) => {
  res.setHeader("Content-Type", "application/javascript");
  res.send(appJs);
});

app.use(express.json());

// API Endpoints
app.get("/api/search", async (req, res) => {
  try {
    const query = req.query.q;
    if (!query) return res.status(400).json({ error: "Query required" });
    const results = await searchVideos(query);
    res.json(results);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.get("/api/info", async (req, res) => {
  try {
    const url = req.query.url;
    if (!url) return res.status(400).json({ error: "URL required" });
    const info = await getVideoInfo(url);
    res.json(info);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

app.post("/api/download", async (req, res) => {
  try {
    const { url, type, quality, audioFormat } = req.body;
    const config = loadConfig();

    const args = [url];

    // Add output template based on type
    if (type === "playlist") {
      args.push(
        "--yes-playlist",
        "-o",
        "playlist/%(playlist_title)s/%(title)s.%(ext)s"
      );
    } else {
      args.push("-o", "%(title)s.%(ext)s");
    }

    if (type === "audio") {
      let fmt = audioFormat || "mp3";
      if (fmt === "ogg") fmt = "vorbis";
      args.push("-x", "--audio-format", fmt, "--audio-quality", "0");
    } else if (type === "thumbnail") {
      args.push("--write-thumbnail", "--skip-download");
    } else {
      // Video (or Playlist Video)
      let formatArg = "bestvideo+bestaudio/best";

      if (quality && quality !== "max") {
        if (quality === "mid-max") {
          // Handle legacy/CLI term if passed, though UI sends numbers
          formatArg = "bestvideo[height<=1080]+bestaudio/best[height<=1080]";
        } else {
          formatArg = `bestvideo[height<=${quality}]+bestaudio/best[height<=${quality}]`;
        }
      }

      // Force AAC audio for compatibility with Windows Media Player
      args.push(
        "-f",
        formatArg,
        "--merge-output-format",
        "mp4",
        "--postprocessor-args",
        "merger+ffmpeg:-c:a aac"
      );
    }

    const options = {
      path: config.downloadPath,
      onProgress: (progress) => {
        io.emit("download-progress", progress);
      },
    };

    download(args, options)
      .then(() => io.emit("download-complete", { url }))
      .catch((err) => io.emit("download-error", { error: err.message }));

    res.json({ status: "started" });
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});

export function startServer() {
  const PORT = 3000;
  httpServer.listen(PORT, () => {
    console.log(chalk.gray(` • GUI Server running at`) + chalk.cyan(" http://localhost:" + PORT));
    open(`http://localhost:${PORT}`);
  });
}
