use anyhow::Result;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::io::{BufRead, BufReader};

use super::formats::{AudioFormat, QualityPreset};

/// Download progress update
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum DownloadProgress {
    Starting(String),
    Downloading { percent: f32, speed: String, eta: String },
    PostProcessing,
    Finished(String),
    Error(String),
}

fn get_executable() -> String {
    crate::deps::check_ytdlp()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "yt-dlp".to_string())
}

/// Download a single video with specified format
#[allow(dead_code)]
pub fn download_video(
    url: &str,
    output_path: &str,
    format: Option<&str>,
    progress_tx: Option<Sender<DownloadProgress>>,
) -> Result<String> {
    let output_template = format!("{}/%(title)s.%(ext)s", output_path);
    let format_arg = format.unwrap_or("bestvideo+bestaudio/best").to_string();

    if let Some(tx) = &progress_tx {
        let _ = tx.send(DownloadProgress::Starting(url.to_string()));
    }

    let mut child = Command::new(get_executable())
        .args([
            "--no-warnings",
            "--newline",
            "-o", &output_template,
            "-f", &format_arg,
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to get stdout");
    let reader = BufReader::new(stdout);

    let mut output_file = String::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(tx) = &progress_tx {
                // Check for destination/filename
                if line.contains("Destination:") {
                    if let Some(dest) = line.split("Destination:").nth(1) {
                        output_file = dest.trim().to_string();
                    }
                } else if line.contains("has already been downloaded") {
                    if let Some(dest) = line.split("download]").nth(1) {
                         // Extract filename from "[download] filename has already..."
                         let part = dest.split("has already").next().unwrap_or("").trim();
                         output_file = part.to_string();
                    }
                } else if line.contains("Merging formats into") {
                     if let Some(dest) = line.split("into").nth(1) {
                        output_file = dest.trim().trim_matches('"').to_string();
                     }
                }

                // Check for progress
                if line.contains("[download]") {
                    if let Some(percent) = parse_progress(&line) {
                        let _ = tx.send(percent);
                    }
                } else if line.contains("[Merger]") || line.contains("[ExtractAudio]") {
                    let _ = tx.send(DownloadProgress::PostProcessing);
                }
            }
        }
    }

    let status = child.wait()?;

    if status.success() {
        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadProgress::Finished(output_file.clone()));
        }
        Ok(output_file)
    } else {
        let msg = "Download failed".to_string();
        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadProgress::Error(msg.clone()));
        }
        anyhow::bail!(msg)
    }
}

/// Download audio with conversion
#[allow(dead_code)]
pub fn download_audio(
    url: &str,
    output_path: &str,
    format: AudioFormat,
    quality: Option<&str>,
    progress_tx: Option<Sender<DownloadProgress>>,
) -> Result<String> {
    let output_template = format!("{}/%(title)s.%(ext)s", output_path);
    let format_ext = format.extension();

    if let Some(tx) = &progress_tx {
        let _ = tx.send(DownloadProgress::Starting(url.to_string()));
    }

    let mut cmd = Command::new(get_executable());
    cmd.args([
        "--no-warnings",
        "--newline",
        "-x",
        "--audio-format", format_ext,
        "-o", &output_template,
    ]);

    if let Some(q) = quality {
        cmd.args(["--audio-quality", q]);
    }

    cmd.arg(url);

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().expect("Failed to get stdout");
    let reader = BufReader::new(stdout);

    let mut output_file = String::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(tx) = &progress_tx {
                 if line.contains("Destination:") {
                    if let Some(dest) = line.split("Destination:").nth(1) {
                        output_file = dest.trim().to_string();
                    }
                }

                if line.contains("[download]") {
                    if let Some(percent) = parse_progress(&line) {
                        let _ = tx.send(percent);
                    }
                } else if line.contains("[ExtractAudio]") {
                    let _ = tx.send(DownloadProgress::PostProcessing);
                }
            }
        }
    }

    let status = child.wait()?;

    if status.success() {
        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadProgress::Finished(output_file.clone()));
        }
        Ok(output_file)
    } else {
        let msg = "Audio download failed".to_string();
        if let Some(tx) = &progress_tx {
            let _ = tx.send(DownloadProgress::Error(msg.clone()));
        }
        anyhow::bail!(msg)
    }
}

/// Download playlist with quality preset
#[allow(dead_code)]
pub fn download_playlist(
    url: &str,
    output_path: &str,
    playlist_name: &str,
    preset: QualityPreset,
    audio_only: bool,
    audio_format: Option<AudioFormat>,
    _parallel: u8,
) -> Result<()> {
    let playlist_dir = Path::new(output_path).join(sanitize_filename(playlist_name));
    std::fs::create_dir_all(&playlist_dir)?;

    let output_template = format!("{}/%(title)s.%(ext)s", playlist_dir.display());
    let format_str = preset.get_format_string();

    let mut cmd = Command::new(get_executable());
    cmd.args([
        "--no-warnings",
        "-o", &output_template,
        "--yes-playlist",
    ]);

    if audio_only {
        cmd.arg("-x");
        if let Some(fmt) = audio_format {
            cmd.args(["--audio-format", fmt.extension()]);
        }
    } else {
        cmd.args(["-f", &format_str]);
    }

    cmd.arg(url);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Playlist download failed: {}", stderr);
    }

    Ok(())
}

/// Download thumbnail
pub fn download_thumbnail(url: &str, output_path: &str) -> Result<Vec<String>> {
    let output = Command::new(get_executable())
        .args([
            "-j",
            "--no-warnings",
            url,
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!("Failed to fetch video info");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let info: serde_json::Value = serde_json::from_str(&stdout)?;

    let video_id = info["id"].as_str().unwrap_or("video");
    let title = info["title"].as_str().unwrap_or("thumbnail");

    // Download different resolutions
    let resolutions = vec![
        ("maxresdefault", "1280x720"),
        ("hqdefault", "480x360"),
        ("default", "120x90"),
    ];

    let mut downloaded = Vec::new();

    for (res_name, _res_size) in resolutions {
        let thumb_url = format!("https://img.youtube.com/vi/{}/{}.jpg", video_id, res_name);
        let output_file = Path::new(output_path)
            .join(format!("{}_{}.jpg", sanitize_filename(title), res_name));

        // Use curl or wget to download
        let result = Command::new("curl")
            .args(["-sL", "-o", output_file.to_str().unwrap(), &thumb_url])
            .output();

        if result.is_ok() {
            downloaded.push(output_file.to_string_lossy().to_string());
        }
    }

    Ok(downloaded)
}

/// Parse progress from yt-dlp output line
#[allow(dead_code)]
fn parse_progress(line: &str) -> Option<DownloadProgress> {
    // Example: [download]  45.2% of 123.45MiB at  5.67MiB/s ETA 00:15
    let parts: Vec<&str> = line.split_whitespace().collect();
    if let Some(percent_str) = parts.iter().find(|p| p.ends_with('%')) {
        let val = percent_str.trim_end_matches('%');
        if let Ok(percent) = val.parse::<f32>() {
            let speed = extract_between(line, "at", "ETA").unwrap_or_default();
            let eta = extract_after(line, "ETA").unwrap_or_default();
            return Some(DownloadProgress::Downloading {
                percent,
                speed: speed.trim().to_string(),
                eta: eta.trim().to_string(),
            });
        }
    }
    None
}

#[allow(dead_code)]
fn extract_between(s: &str, start: &str, end: &str) -> Option<String> {
    let start_idx = s.find(start)? + start.len();
    let end_idx = s[start_idx..].find(end)? + start_idx;
    Some(s[start_idx..end_idx].to_string())
}

#[allow(dead_code)]
fn extract_after(s: &str, marker: &str) -> Option<String> {
    let idx = s.find(marker)? + marker.len();
    Some(s[idx..].to_string())
}

/// Sanitize filename for filesystem
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}
