# 🎬 Onyx - The Ultimate YouTube Downloader

<p align="center">
  <img src="assets/logo.png" alt="Onyx Logo" width="200">
</p>

<p align="center">
  <strong>A beautiful, blazing-fast TUI application for downloading YouTube videos</strong>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#features">Features</a> •
  <a href="#usage">Usage</a> •
  <a href="#requirements">Requirements</a>
</p>

---

## ✨ Features

- 🎨 **Beautiful TUI** - Modern, colorful terminal interface
- ⚡ **Blazing Fast** - Built with Rust for maximum performance
- 📺 **Multiple Formats** - Download in various video and audio formats
- 🔊 **Audio Extraction** - Extract audio in MP3, AAC, and more
- 📊 **Progress Tracking** - Real-time download progress
- 🎯 **Quality Selection** - Choose your preferred video quality

## 📦 Installation

### Linux

#### Option 1: Install Script (Recommended)

```bash
# Clone the repository
git clone https://github.com/sethunthunder111/onyx.git
cd onyx

# Run the install script
./install.sh
```

The install script will:
- ✅ Check for required dependencies
- ✅ Build Onyx from source
- ✅ Install to `~/.local/bin`
- ✅ Set up desktop integration

To uninstall:
```bash
./install.sh --uninstall
```

#### Option 2: Cargo Install

```bash
# From crates.io (when published)
cargo install onyx

# From GitHub
cargo install --git https://github.com/sethunthunder111/onyx.git
```

#### Option 3: AppImage

Download the latest AppImage from the [Releases](https://github.com/sethunthunder111/onyx/releases) page:

```bash
chmod +x Onyx-x86_64.AppImage
./Onyx-x86_64.AppImage
```

#### Option 4: Build from Source

```bash
git clone https://github.com/sethunthunder111/onyx.git
cd onyx
cargo build --release

# The binary will be at target/release/onyx
./target/release/onyx
```

### Windows

Download the latest `.exe` from the [Releases](https://github.com/sethunthunder111/onyx/releases) page.

## 🔧 Requirements

Before using Onyx, ensure you have the following installed:

### Required Dependencies

| Dependency | Purpose | Installation |
|------------|---------|--------------|
| **yt-dlp** | Video downloading engine | `pip install yt-dlp` or via package manager |
| **ffmpeg** | Video/audio processing | `sudo apt install ffmpeg` |

### Installing Dependencies

**Debian/Ubuntu:**
```bash
sudo apt install yt-dlp ffmpeg
```

**Fedora:**
```bash
sudo dnf install yt-dlp ffmpeg
```

**Arch Linux:**
```bash
sudo pacman -S yt-dlp ffmpeg
```

**macOS:**
```bash
brew install yt-dlp ffmpeg
```

## 🚀 Usage

Launch Onyx from your terminal:

```bash
onyx
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate menu |
| `Enter` | Select option |
| `Esc` | Go back / Cancel |
| `q` | Quit application |
| `?` | Show help |

## 🛠️ Development

```bash
# Clone the repo
git clone https://github.com/sethunthunder111/onyx.git
cd onyx

# Run in development mode
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - The powerful video downloading engine
- [ratatui](https://github.com/ratatui-org/ratatui) - The amazing TUI library
- [ffmpeg](https://ffmpeg.org/) - For media processing

---

<p align="center">
  Made with ❤️ by the ONYX Team
</p>
