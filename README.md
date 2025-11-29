# [ONYX 💎](https://sethunthunder111.github.io/onyx)

> **The Ultimate YouTube Downloader.**  
> *Simple. Fast. Elegant.*

![Onyx Banner](https://via.placeholder.com/1200x400/0f0f0f/ffffff?text=ONYX)

**Onyx** is a powerful, production-ready YouTube downloader built with Node.js. It features a stunning CLI and a beautiful, modern Web GUI for the ultimate downloading experience.

## ✨ Features

- **🌐 Modern Web GUI**: A sleek, shadcn-inspired monochromatic interface.
- **🖥️ Powerful CLI**: Robust command-line interface for power users.
- **🎬 High Quality**: Download videos in up to 4K/8K resolution.
- **🎵 Audio Extraction**: Convert videos to MP3, OGG (Vorbis), M4A, or WAV.
- **📦 Playlist Support**: Download entire playlists with organized folder structures.
- **🖼️ Thumbnails**: Grab high-quality video thumbnails.
- **⚡ Real-time Progress**: Accurate speed and ETA tracking via Socket.io.
- **🎨 Theming**: Built-in Dark and Light modes.

## 🚀 Installation

1. **Clone the repository:**

    ```bash
    git clone https://github.com/sethunthunder111/onyx.git
    cd onyx
    ```

2. **Install dependencies:**

    ```bash
    npm install
    ```

3. **Start the application:**

    ```bash
    npm start
    ```

## 📖 Usage

### CLI Mode

Follow the interactive prompts to search for videos, paste URLs, and select download formats.

### GUI Mode

Select **"🌐 GUI Version"** from the main menu. The application will launch a local server and open `http://localhost:3000` in your default browser.

## 🛠️ Tech Stack

- **Runtime**: Node.js
- **Backend**: Express, Socket.io
- **Core**: yt-dlp (via yt-dlp-wrap)
- **Frontend**: HTML5, CSS3 (Variables), Vanilla JS
- **CLI**: Inquirer, Chalk, Ora, Figlet

## 📄 License

This project is licensed under the [ISC License](LICENSE).

---

<p align="center">
  Made with ❤️ by <a href="https://github.com/sethunthunder111">SethunThunder</a>
</p>
