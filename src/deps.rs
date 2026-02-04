use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Get the path where we store dependencies
fn deps_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("onyx")
        .join("deps")
}

/// Get the yt-dlp executable path
pub fn ytdlp_path() -> PathBuf {
    let deps = deps_dir();
    if cfg!(windows) {
        deps.join("yt-dlp.exe")
    } else {
        deps.join("yt-dlp")
    }
}

/// Get the ffmpeg executable path
pub fn ffmpeg_path() -> PathBuf {
    let deps = deps_dir();
    if cfg!(windows) {
        deps.join("ffmpeg.exe")
    } else {
        deps.join("ffmpeg")
    }
}

/// Check if yt-dlp is available (either in PATH or in deps)
pub fn check_ytdlp() -> Option<PathBuf> {
    // First check our deps directory
    let local = ytdlp_path();
    if local.exists() {
        return Some(local);
    }

    // Check ~/.local/bin (common user-installed location)
    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local").join("bin").join("yt-dlp");
        if local_bin.exists() {
            return Some(local_bin);
        }
    }

    // Then check system PATH
    let cmd = if cfg!(windows) { "where" } else { "which" };
    let output = Command::new(cmd).arg("yt-dlp").output().ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()?
            .trim()
            .to_string();
        Some(PathBuf::from(path))
    } else {
        None
    }
}

/// Check if ffmpeg is available
pub fn check_ffmpeg() -> Option<PathBuf> {
    let local = ffmpeg_path();
    if local.exists() {
        return Some(local);
    }

    let cmd = if cfg!(windows) { "where" } else { "which" };
    let output = Command::new(cmd).arg("ffmpeg").output().ok()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()?
            .trim()
            .to_string();
        Some(PathBuf::from(path))
    } else {
        None
    }
}

/// Download yt-dlp binary
pub fn download_ytdlp(progress_callback: impl Fn(&str)) -> Result<PathBuf> {
    let deps = deps_dir();
    fs::create_dir_all(&deps)?;

    let target = ytdlp_path();

    progress_callback("Downloading yt-dlp...");

    // Determine the correct download URL based on OS and architecture
    let url = get_ytdlp_download_url();

    // Use curl or wget to download
    let result = if cfg!(windows) {
        // On Windows, use PowerShell
        Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Invoke-WebRequest -Uri '{}' -OutFile '{}'",
                    url,
                    target.display()
                ),
            ])
            .output()
    } else {
        // On Unix, try curl first, then wget
        let curl_result = Command::new("curl")
            .args(["-sL", "-o", target.to_str().unwrap(), &url])
            .output();

        if curl_result.is_err() || !curl_result.as_ref().unwrap().status.success() {
            Command::new("wget")
                .args(["-q", "-O", target.to_str().unwrap(), &url])
                .output()
        } else {
            curl_result
        }
    };

    match result {
        Ok(output) if output.status.success() => {
            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&target)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&target, perms)?;
            }

            progress_callback("yt-dlp downloaded successfully!");
            Ok(target)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to download yt-dlp: {}", stderr)
        }
        Err(e) => anyhow::bail!("Failed to download yt-dlp: {}", e),
    }
}

/// Get the appropriate yt-dlp download URL for the current platform
fn get_ytdlp_download_url() -> String {
    let base_url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download";

    if cfg!(target_os = "windows") {
        format!("{}/yt-dlp.exe", base_url)
    } else if cfg!(target_os = "macos") {
        format!("{}/yt-dlp_macos", base_url)
    } else {
        // Linux
        let arch = env::consts::ARCH;
        if arch == "aarch64" || arch == "arm" {
            format!("{}/yt-dlp_linux_aarch64", base_url)
        } else {
            format!("{}/yt-dlp_linux", base_url)
        }
    }
}

/// Dependency status
#[derive(Debug, Clone)]
pub struct DependencyStatus {
    pub ytdlp: Option<PathBuf>,
    pub ytdlp_version: Option<String>,
    #[allow(dead_code)]
    pub ffmpeg: Option<PathBuf>,
}

impl DependencyStatus {
    pub fn check() -> Self {
        let ytdlp = check_ytdlp();
        let ytdlp_version = ytdlp.as_ref().and_then(|path| {
            Command::new(path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
        });

        Self {
            ytdlp,
            ytdlp_version,
            ffmpeg: check_ffmpeg(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ytdlp.is_some()
    }

    pub fn ytdlp_binary(&self) -> String {
        self.ytdlp
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "yt-dlp".to_string())
    }
}

/// Ensure dependencies are available, downloading if necessary
pub fn ensure_dependencies(progress_callback: impl Fn(&str)) -> Result<DependencyStatus> {
    let mut status = DependencyStatus::check();

    if status.ytdlp.is_none() {
        progress_callback("yt-dlp not found. Downloading...");
        status.ytdlp = Some(download_ytdlp(&progress_callback)?);
    } else if let Some(ref ver) = status.ytdlp_version {
        // Update if version is older than Nov 2024 (YouTube often breaks older versions)
        if ver.as_str() < "2024.11.00" {
            progress_callback(&format!("Found outdated yt-dlp version: {}. Updating...", ver));
            status.ytdlp = Some(download_ytdlp(&progress_callback)?);
        }
    }

    // After potential download/update, recheck version if we downloaded
    if status.ytdlp_version.as_ref().map(|v| v.as_str() < "2024.11.00").unwrap_or(true) && status.ytdlp.is_some() {
         // Re-read version from the (possibly new) binary
         if let Some(ref path) = status.ytdlp {
            status.ytdlp_version = Command::new(path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                });
         }
    }

    if !status.is_ready() {
        anyhow::bail!("Failed to set up required dependencies");
    }

    Ok(status)
}
