#!/bin/bash

# Onyx - The Ultimate YouTube Downloader
# Linux Installation Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Configuration
INSTALL_DIR="${HOME}/.local/bin"
ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
DESKTOP_DIR="${HOME}/.local/share/applications"
APP_NAME="onyx"
REPO_URL="https://github.com/sethunthunder111/onyx"

# Print banner
print_banner() {
    echo -e "${PURPLE}"
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║                                                       ║"
    echo "║     ██████╗ ███╗   ██╗██╗   ██╗██╗  ██╗              ║"
    echo "║    ██╔═══██╗████╗  ██║╚██╗ ██╔╝╚██╗██╔╝              ║"
    echo "║    ██║   ██║██╔██╗ ██║ ╚████╔╝  ╚███╔╝               ║"
    echo "║    ██║   ██║██║╚██╗██║  ╚██╔╝   ██╔██╗               ║"
    echo "║    ╚██████╔╝██║ ╚████║   ██║   ██╔╝ ██╗              ║"
    echo "║     ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝              ║"
    echo "║                                                       ║"
    echo "║          The Ultimate YouTube Downloader              ║"
    echo "║                                                       ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Print colored message
info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check dependencies
check_dependencies() {
    info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for yt-dlp (required for Onyx to function)
    if ! command_exists yt-dlp; then
        missing_deps+=("yt-dlp")
    fi
    
    # Check for ffmpeg (required for video processing)
    if ! command_exists ffmpeg; then
        missing_deps+=("ffmpeg")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        warning "Missing dependencies: ${missing_deps[*]}"
        echo ""
        echo -e "${CYAN}To install the missing dependencies:${NC}"
        echo ""
        echo -e "${BOLD}Debian/Ubuntu:${NC}"
        echo "  sudo apt install yt-dlp ffmpeg"
        echo ""
        echo -e "${BOLD}Fedora:${NC}"
        echo "  sudo dnf install yt-dlp ffmpeg"
        echo ""
        echo -e "${BOLD}Arch Linux:${NC}"
        echo "  sudo pacman -S yt-dlp ffmpeg"
        echo ""
        echo -e "${BOLD}Or install yt-dlp via pip:${NC}"
        echo "  pip install yt-dlp"
        echo ""
        
        read -p "Continue installation anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            error "Installation cancelled"
        fi
    else
        success "All dependencies are installed!"
    fi
}

# Install from source using Cargo
install_from_source() {
    info "Installing from source using Cargo..."
    
    if ! command_exists cargo; then
        error "Cargo is not installed. Please install Rust first: https://rustup.rs"
    fi
    
    # Check if we're in the project directory
    if [ -f "Cargo.toml" ]; then
        info "Building from local source..."
        cargo build --release
        
        # Create install directory if it doesn't exist
        mkdir -p "$INSTALL_DIR"
        
        # Copy binary
        cp "target/release/${APP_NAME}" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/${APP_NAME}"
        
        success "Binary installed to $INSTALL_DIR/${APP_NAME}"
    else
        # Clone and build
        info "Cloning repository..."
        local temp_dir=$(mktemp -d)
        git clone "$REPO_URL" "$temp_dir"
        cd "$temp_dir"
        
        cargo build --release
        
        mkdir -p "$INSTALL_DIR"
        cp "target/release/${APP_NAME}" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/${APP_NAME}"
        
        # Cleanup
        rm -rf "$temp_dir"
        
        success "Binary installed to $INSTALL_DIR/${APP_NAME}"
    fi
}

# Install desktop entry and icon
install_desktop_integration() {
    info "Setting up desktop integration..."
    
    # Create directories
    mkdir -p "$ICON_DIR"
    mkdir -p "$DESKTOP_DIR"
    
    # Copy icon if available
    if [ -f "assets/icon.png" ]; then
        cp "assets/icon.png" "$ICON_DIR/${APP_NAME}.png"
        success "Icon installed"
    elif [ -f "assets/logo.png" ]; then
        cp "assets/logo.png" "$ICON_DIR/${APP_NAME}.png"
        success "Icon installed"
    fi
    
    # Create desktop entry
    cat > "$DESKTOP_DIR/${APP_NAME}.desktop" << EOF
[Desktop Entry]
Name=Onyx
GenericName=YouTube Downloader
Comment=The Ultimate YouTube Downloader
Exec=${INSTALL_DIR}/${APP_NAME}
Icon=${APP_NAME}
Type=Application
Categories=Video;Audio;Network;
Terminal=true
Keywords=youtube;video;download;audio;music;
StartupNotify=true
EOF
    
    # Update desktop database
    if command_exists update-desktop-database; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
    
    success "Desktop entry created"
}

# Add install directory to PATH if needed
setup_path() {
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        warning "$INSTALL_DIR is not in your PATH"
        echo ""
        echo -e "${CYAN}Add this line to your shell config (~/.bashrc, ~/.zshrc, etc.):${NC}"
        echo ""
        echo -e "  ${BOLD}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
        echo ""
        echo "Then restart your terminal or run: source ~/.bashrc"
        echo ""
    fi
}

# Main installation
main() {
    print_banner
    
    echo -e "${BOLD}Welcome to the Onyx installer!${NC}"
    echo ""
    echo "This script will:"
    echo "  1. Check for required dependencies (yt-dlp, ffmpeg)"
    echo "  2. Build and install Onyx to ~/.local/bin"
    echo "  3. Set up desktop integration (icon & launcher)"
    echo ""
    
    read -p "Continue with installation? (Y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        error "Installation cancelled"
    fi
    
    echo ""
    check_dependencies
    echo ""
    install_from_source
    echo ""
    install_desktop_integration
    echo ""
    setup_path
    
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║          Installation Complete! 🎉                    ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "Run ${BOLD}${CYAN}onyx${NC} to start the application!"
    echo ""
}

# Run uninstall if --uninstall flag is passed
if [ "$1" = "--uninstall" ] || [ "$1" = "-u" ]; then
    print_banner
    info "Uninstalling Onyx..."
    
    rm -f "$INSTALL_DIR/${APP_NAME}"
    rm -f "$ICON_DIR/${APP_NAME}.png"
    rm -f "$DESKTOP_DIR/${APP_NAME}.desktop"
    
    success "Onyx has been uninstalled"
    exit 0
fi

# Run help if --help flag is passed
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    print_banner
    echo "Usage: ./install.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help       Show this help message"
    echo "  -u, --uninstall  Uninstall Onyx"
    echo ""
    echo "Requirements:"
    echo "  - Rust/Cargo (https://rustup.rs)"
    echo "  - yt-dlp (for downloading videos)"
    echo "  - ffmpeg (for video processing)"
    echo ""
    exit 0
fi

main
