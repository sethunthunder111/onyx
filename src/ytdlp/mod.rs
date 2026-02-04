pub mod download;
pub mod formats;
pub mod search;

use anyhow::{bail, Result};
use std::process::Command;

/// Check if yt-dlp is available on the system
#[allow(dead_code)]
pub fn check_ytdlp() -> Result<String> {
    let output = Command::new("yt-dlp").arg("--version").output();

    match output {
        Ok(out) if out.status.success() => {
            let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(version)
        }
        _ => bail!("yt-dlp is not installed or not in PATH. Please install it first."),
    }
}

/// Run a yt-dlp command and return stdout
#[allow(dead_code)]
pub fn run_ytdlp(args: &[&str]) -> Result<String> {
    let output = Command::new("yt-dlp").args(args).output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("yt-dlp error: {}", stderr)
    }
}

/// Run yt-dlp with JSON output
#[allow(dead_code)]
pub fn run_ytdlp_json<T: serde::de::DeserializeOwned>(args: &[&str]) -> Result<T> {
    let mut full_args = vec!["-j"];
    full_args.extend_from_slice(args);
    let output = run_ytdlp(&full_args)?;
    let parsed: T = serde_json::from_str(&output)?;
    Ok(parsed)
}
