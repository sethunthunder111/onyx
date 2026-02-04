use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Format {
    pub format_id: String,
    #[serde(default)]
    pub format_note: Option<String>,
    #[serde(default)]
    pub ext: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub fps: Option<f64>,
    #[serde(default)]
    pub vcodec: Option<String>,
    #[serde(default)]
    pub acodec: Option<String>,
    #[serde(default)]
    pub abr: Option<f64>,
    #[serde(default)]
    pub vbr: Option<f64>,
    #[serde(default)]
    pub tbr: Option<f64>,
    #[serde(default)]
    pub filesize: Option<u64>,
    #[serde(default)]
    pub filesize_approx: Option<u64>,
}

impl Format {
    pub fn has_video(&self) -> bool {
        self.vcodec.as_ref().map(|v| v != "none").unwrap_or(false)
    }

    pub fn has_audio(&self) -> bool {
        self.acodec.as_ref().map(|a| a != "none").unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn is_video_only(&self) -> bool {
        self.has_video() && !self.has_audio()
    }

    pub fn is_audio_only(&self) -> bool {
        !self.has_video() && self.has_audio()
    }

    #[allow(dead_code)]
    pub fn get_resolution_label(&self) -> String {
        if let Some(h) = self.height {
            match h {
                h if h >= 4320 => "8K".to_string(),
                h if h >= 2160 => "4K".to_string(),
                h if h >= 1440 => "2K".to_string(),
                h if h >= 1080 => "1080p".to_string(),
                h if h >= 720 => "720p".to_string(),
                h if h >= 480 => "480p".to_string(),
                h if h >= 360 => "360p".to_string(),
                h if h >= 240 => "240p".to_string(),
                _ => format!("{}p", h),
            }
        } else if self.is_audio_only() {
            if let Some(abr) = self.abr {
                format!("{}kbps", abr as u32)
            } else {
                "Audio".to_string()
            }
        } else {
            self.resolution.clone().unwrap_or_else(|| "Unknown".to_string())
        }
    }

    pub fn get_audio_bitrate(&self) -> Option<u32> {
        self.abr.map(|b| b as u32)
    }

    pub fn get_size_str(&self) -> String {
        let size = self.filesize.or(self.filesize_approx);
        match size {
            Some(s) if s >= 1_073_741_824 => format!("{:.1} GB", s as f64 / 1_073_741_824.0),
            Some(s) if s >= 1_048_576 => format!("{:.1} MB", s as f64 / 1_048_576.0),
            Some(s) if s >= 1024 => format!("{:.1} KB", s as f64 / 1024.0),
            Some(s) => format!("{} B", s),
            None => "~".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct VideoInfo {
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
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub formats: Vec<Format>,
    #[serde(default)]
    pub description: Option<String>,
}

impl VideoInfo {
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
            let hours = (dur / 3600.0) as u64;
            let minutes = ((dur % 3600.0) / 60.0) as u64;
            let seconds = (dur % 60.0) as u64;
            if hours > 0 {
                return format!("{}:{:02}:{:02}", hours, minutes, seconds);
            }
            return format!("{}:{:02}", minutes, seconds);
        }
        "?:??".to_string()
    }

    /// Get video formats grouped by resolution
    pub fn get_video_formats(&self) -> Vec<(String, Vec<&Format>)> {
        let mut grouped: HashMap<u32, Vec<&Format>> = HashMap::new();

        for fmt in &self.formats {
            if fmt.has_video() {
                if let Some(h) = fmt.height {
                    grouped.entry(h).or_default().push(fmt);
                }
            }
        }

        let mut sorted: Vec<_> = grouped.into_iter().collect();
        sorted.sort_by(|a, b| b.0.cmp(&a.0)); // Sort by height descending

        sorted
            .into_iter()
            .map(|(h, fmts)| {
                let label = match h {
                    h if h >= 4320 => "8K".to_string(),
                    h if h >= 2160 => "4K".to_string(),
                    h if h >= 1440 => "2K".to_string(),
                    h if h >= 1080 => "1080p".to_string(),
                    h if h >= 720 => "720p".to_string(),
                    h if h >= 480 => "480p".to_string(),
                    h if h >= 360 => "360p".to_string(),
                    h if h >= 240 => "240p".to_string(),
                    _ => format!("{}p", h),
                };
                (label, fmts)
            })
            .collect()
    }

    /// Get audio-only formats sorted by bitrate
    pub fn get_audio_formats(&self) -> Vec<&Format> {
        let mut audio: Vec<_> = self.formats.iter().filter(|f| f.is_audio_only()).collect();
        audio.sort_by(|a, b| {
            let abr_a = a.abr.unwrap_or(0.0);
            let abr_b = b.abr.unwrap_or(0.0);
            abr_b.partial_cmp(&abr_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        audio
    }

    /// Get the best format for a given resolution cap
    #[allow(dead_code)]
    pub fn get_best_format_for_resolution(&self, max_height: Option<u32>) -> Option<String> {
        let mut video_formats: Vec<_> = self
            .formats
            .iter()
            .filter(|f| f.has_video() && f.height.is_some())
            .collect();

        video_formats.sort_by(|a, b| {
            let ha = a.height.unwrap_or(0);
            let hb = b.height.unwrap_or(0);
            hb.cmp(&ha)
        });

        let best = if let Some(max) = max_height {
            video_formats.iter().find(|f| f.height.unwrap_or(0) <= max)
        } else {
            video_formats.first()
        };

        best.map(|f| format!("bestvideo[height<={}]+bestaudio/best", f.height.unwrap_or(1080)))
    }
}

fn get_executable() -> String {
    crate::deps::check_ytdlp()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "yt-dlp".to_string())
}

/// Fetch video information including available formats
pub fn get_video_info(url: &str) -> Result<VideoInfo> {
    let output = Command::new(get_executable())
        .args(["-j", "--no-warnings", url])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to fetch video info: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let info: VideoInfo = serde_json::from_str(&stdout)?;
    Ok(info)
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PlaylistEntry {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub duration: Option<f64>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct PlaylistInfo {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub entries: Vec<PlaylistEntry>,
    #[serde(default)]
    pub playlist_count: Option<u32>,
}

/// Fetch playlist information
pub fn get_playlist_info(url: &str) -> Result<PlaylistInfo> {
    let output = Command::new(get_executable())
        .args(["-j", "--flat-playlist", "--no-warnings", url])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to fetch playlist info: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    let mut title = String::from("Playlist");

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<PlaylistEntry>(line) {
            entries.push(entry);
        }
    }

    // Try to get playlist title from first line's metadata
    if let Some(first_line) = stdout.lines().next() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(first_line) {
            if let Some(t) = val.get("playlist_title").and_then(|v| v.as_str()) {
                title = t.to_string();
            }
        }
    }

    Ok(PlaylistInfo {
        id: None,
        title,
        entries,
        playlist_count: None,
    })
}

/// Quality preset for playlist downloads
#[derive(Debug, Clone, Copy)]
pub enum QualityPreset {
    God,    // Maximum available
    Ultra,  // 4K cap
    Pro,    // 2K cap
    High,   // 1080p cap
    Medium, // 720p cap
    Low,    // 480p cap
}

impl QualityPreset {
    pub fn get_format_string(&self) -> String {
        match self {
            QualityPreset::God => "bestvideo+bestaudio/best".to_string(),
            QualityPreset::Ultra => "bestvideo[height<=2160]+bestaudio/best[height<=2160]".to_string(),
            QualityPreset::Pro => "bestvideo[height<=1440]+bestaudio/best[height<=1440]".to_string(),
            QualityPreset::High => "bestvideo[height<=1080]+bestaudio/best[height<=1080]".to_string(),
            QualityPreset::Medium => "bestvideo[height<=720]+bestaudio/best[height<=720]".to_string(),
            QualityPreset::Low => "bestvideo[height<=480]+bestaudio/best[height<=480]".to_string(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            QualityPreset::God => "👑 GOD (Maximum Quality)",
            QualityPreset::Ultra => "🔥 Ultra (4K)",
            QualityPreset::Pro => "💎 PRO (2K)",
            QualityPreset::High => "⭐ High (1080p)",
            QualityPreset::Medium => "📺 Medium (720p)",
            QualityPreset::Low => "📱 Low (480p)",
        }
    }
}

/// Audio format for conversion
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Flac,
    M4a,
    Opus,
}

impl AudioFormat {
    pub fn extension(&self) -> &str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Wav => "wav",
            AudioFormat::Ogg => "ogg",
            AudioFormat::Flac => "flac",
            AudioFormat::M4a => "m4a",
            AudioFormat::Opus => "opus",
        }
    }

    pub fn all() -> Vec<AudioFormat> {
        vec![
            AudioFormat::Mp3,
            AudioFormat::M4a,
            AudioFormat::Ogg,
            AudioFormat::Flac,
            AudioFormat::Wav,
            AudioFormat::Opus,
        ]
    }
}
