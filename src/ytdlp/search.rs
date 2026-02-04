use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub duration_string: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub uploader: Option<String>,
    #[serde(default)]
    pub view_count: Option<u64>,
    #[serde(default)]
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub webpage_url: Option<String>,
}

impl SearchResult {
    pub fn get_channel(&self) -> String {
        self.channel
            .clone()
            .or_else(|| self.uploader.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn get_duration(&self) -> String {
        if let Some(dur_str) = &self.duration_string {
            return dur_str.clone();
        }
        if let Some(dur) = self.duration {
            let minutes = (dur / 60.0) as u64;
            let seconds = (dur % 60.0) as u64;
            return format!("{}:{:02}", minutes, seconds);
        }
        "?:??".to_string()
    }

    pub fn get_url(&self) -> String {
        self.webpage_url
            .clone()
            .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", self.id))
    }
}

fn get_executable() -> String {
    crate::deps::check_ytdlp()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "yt-dlp".to_string())
}

/// Search YouTube for videos
pub fn search_youtube(query: &str, max_results: u32) -> Result<Vec<SearchResult>> {
    let search_query = format!("ytsearch{}:{}", max_results, query);

    let output = Command::new(get_executable())
        .args([
            "-j",
            "--flat-playlist",
            "--no-warnings",
            &search_query,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Search failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(result) = serde_json::from_str::<SearchResult>(line) {
            results.push(result);
        }
    }

    Ok(results)
}
