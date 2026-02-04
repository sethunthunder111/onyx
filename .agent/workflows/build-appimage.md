---
description: Build a Linux AppImage for Onyx
---

This workflow builds a standalone Linux AppImage for Onyx, which includes a custom icon and desktop entry.

### Prerequisites
- Rust and Cargo installed
- Internet connection (to download `appimagetool`)

### Steps

1. **Build the release binary**
// turbo
```bash
cargo build --release
```

2. **Prepare the AppDir structure**
```bash
mkdir -p Onyx.AppDir
cp target/release/onyx Onyx.AppDir/onyx
```

3. **Ensure AppRun and Desktop files exist**
If they don't exist, create them in `Onyx.AppDir/`:
- `AppRun`: The entry point script.
- `onyx.desktop`: Metadata for the desktop environment.
- `onyx.png`: The application icon.

4. **Download appimagetool (if not present)**
// turbo
```bash
if [ ! -f appimagetool ]; then
    curl -L -o appimagetool "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x appimagetool
fi
```

5. **Generate the AppImage**
// turbo
```bash
./appimagetool Onyx.AppDir Onyx-x86_64.AppImage --appimage-extract-and-run
```

The resulting `Onyx-x86_64.AppImage` can be shared and run on most Linux distributions.
