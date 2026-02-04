#![allow(dead_code)]

use axum::{
    Router,
    extract::Query,
    http::{header, StatusCode, Uri},
    response::{Html, IntoResponse, Json},
    routing::{get, post},
};
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use crate::ytdlp;

/// Embedded static files
#[derive(Embed)]
#[folder = "src/web/static/"]
struct Assets;

/// Download progress state
#[derive(Clone, Debug, Serialize)]
pub struct DownloadState {
    pub id: String,
    pub title: String,
    pub status: String,
    pub progress: f32,
    pub eta: Option<String>,
}

/// Shared application state
pub struct AppState {
    pub downloads: Mutex<HashMap<String, DownloadState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            downloads: Mutex::new(HashMap::new()),
        }
    }
}

/// Start the web server
pub async fn start_server(port: u16) -> anyhow::Result<()> {
    let state = Arc::new(AppState::new());

    let app = Router::new()
        // Static files
        .route("/", get(index_handler))
        .route("/static/*file", get(static_handler))
        // API routes
        .route("/api/search", get(search_handler))
        .route("/api/video-info", get(video_info_handler))
        .route("/api/playlist-info", get(playlist_info_handler))
        .route("/api/download", post(download_handler))
        .route("/api/download-audio", post(download_audio_handler))
        .route("/api/download-thumbnail", post(download_thumbnail_handler))
        .route("/api/download-playlist", post(download_playlist_handler))
        .route("/api/progress", get(progress_handler))
        .with_state(state);

    let (listener, actual_port) = {
        let mut current_port = port;
        loop {
            let addr = SocketAddr::from(([0, 0, 0, 0], current_port));
            match tokio::net::TcpListener::bind(addr).await {
                Ok(l) => break (l, current_port),
                Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                    if current_port >= port + 10 {
                        return Err(e.into());
                    }
                    current_port += 1;
                }
                Err(e) => return Err(e.into()),
            }
        }
    };

    println!("\n🌐 ONYX Web Server running at http://localhost:{}\n", actual_port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve index.html
async fn index_handler() -> impl IntoResponse {
    match Assets::get("index.html") {
        Some(content) => Html(content.data.into_owned()).into_response(),
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

/// Serve static files
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches("/static/");
    
    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

// === API Types ===

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
}

#[derive(Serialize)]
struct SearchResult {
    id: String,
    title: String,
    channel: String,
    duration: String,
    thumbnail: String,
    url: String,
}

#[derive(Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize)]
struct UrlQuery {
    url: String,
}

#[derive(Serialize)]
struct FormatOption {
    format_string: String,
    label: String,
    size: String,
}

#[derive(Serialize)]
struct VideoInfoResponse {
    title: String,
    channel: String,
    duration: String,
    thumbnail: String,
    formats: Vec<FormatOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct PlaylistInfoResponse {
    title: String,
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize)]
struct DownloadRequest {
    url: String,
    format: String,
}

#[derive(Deserialize)]
struct PlaylistDownloadRequest {
    url: String,
    preset: String,
}

#[derive(Serialize)]
struct DownloadResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize)]
struct ProgressQuery {
    id: String,
}

#[derive(Serialize)]
struct ProgressResponse {
    status: String,
    progress: f32,
    eta: Option<String>,
}

// === API Handlers ===

async fn search_handler(Query(query): Query<SearchQuery>) -> Json<SearchResponse> {
    match ytdlp::search::search_youtube(&query.q, 20) {
        Ok(results) => {
            let results = results
                .into_iter()
                .map(|r| {
                    let id = r.id.clone();
                    let thumbnail = r.thumbnail.clone().unwrap_or_else(|| {
                        format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", id)
                    });
                    SearchResult {
                        id: id.clone(),
                        title: r.title.clone(),
                        channel: r.get_channel(),
                        duration: r.get_duration(),
                        thumbnail,
                        url: r.get_url(),
                    }
                })
                .collect();
            Json(SearchResponse { results, error: None })
        }
        Err(e) => Json(SearchResponse {
            results: vec![],
            error: Some(e.to_string()),
        }),
    }
}

async fn video_info_handler(Query(query): Query<UrlQuery>) -> Json<serde_json::Value> {
    match ytdlp::formats::get_video_info(&query.url) {
        Ok(info) => {
            let formats: Vec<FormatOption> = info
                .formats
                .iter()
                .filter(|f| f.vcodec.as_deref() != Some("none") && f.height.is_some())
                .filter_map(|f| {
                    let height = f.height?;
                    let ext = f.ext.as_deref().unwrap_or("mp4");
                    let size = f.filesize
                        .or(f.filesize_approx.map(|s| s as u64))
                        .map(|s| format_size(s))
                        .unwrap_or_else(|| "Unknown".to_string());
                    
                    Some(FormatOption {
                        format_string: f.format_id.clone(),
                        label: format!("{}p {}", height, ext.to_uppercase()),
                        size,
                    })
                })
                .collect();

            // Deduplicate by height, keeping highest quality
            let mut seen_heights = std::collections::HashSet::new();
            let formats: Vec<FormatOption> = formats
                .into_iter()
                .filter(|f| {
                    let height = f.label.split('p').next().unwrap_or("");
                    seen_heights.insert(height.to_string())
                })
                .take(8)
                .collect();

            Json(serde_json::json!({
                "title": info.title,
                "channel": info.channel.or(info.uploader).unwrap_or_else(|| "Unknown".to_string()),
                "duration": format_duration(info.duration.unwrap_or(0.0) as u64),
                "thumbnail": info.thumbnail.unwrap_or_default(),
                "formats": formats
            }))
        }
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn playlist_info_handler(Query(query): Query<UrlQuery>) -> Json<serde_json::Value> {
    match ytdlp::formats::get_playlist_info(&query.url) {
        Ok(info) => Json(serde_json::json!({
            "title": info.title,
            "count": info.entries.len()
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

async fn download_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<DownloadRequest>,
) -> Json<DownloadResponse> {
    let id = uuid::Uuid::new_v4().to_string();
    let download_id = id.clone();
    
    // Get video title first
    let title = match ytdlp::formats::get_video_info(&req.url) {
        Ok(info) => info.title,
        Err(_) => "Unknown".to_string(),
    };

    // Initialize download state
    {
        let mut downloads = state.downloads.lock().unwrap();
        downloads.insert(id.clone(), DownloadState {
            id: id.clone(),
            title: title.clone(),
            status: "Starting".to_string(),
            progress: 0.0,
            eta: None,
        });
    }

    // Start download in background
    let state_clone = state.clone();
    let url = req.url.clone();
    let format = req.format.clone();
    
    tokio::spawn(async move {
        let output_path = get_download_path();
        
        // Run download
        let std_tx = std::sync::mpsc::channel().0;
        let _ = ytdlp::download::download_video(&url, &output_path, Some(&format), Some(std_tx));
        
        // Mark as completed
        let mut downloads = state_clone.downloads.lock().unwrap();
        if let Some(state) = downloads.get_mut(&download_id) {
            if state.status != "Completed" && !state.status.starts_with("Error") {
                state.status = "Completed".to_string();
                state.progress = 100.0;
            }
        }
    });

    Json(DownloadResponse {
        id: Some(id),
        error: None,
    })
}

async fn download_audio_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<DownloadRequest>,
) -> Json<DownloadResponse> {
    let id = uuid::Uuid::new_v4().to_string();
    let download_id = id.clone();
    
    let title = match ytdlp::formats::get_video_info(&req.url) {
        Ok(info) => info.title,
        Err(_) => "Unknown".to_string(),
    };

    {
        let mut downloads = state.downloads.lock().unwrap();
        downloads.insert(id.clone(), DownloadState {
            id: id.clone(),
            title: format!("{} (Audio)", title),
            status: "Starting".to_string(),
            progress: 0.0,
            eta: None,
        });
    }

    let state_clone = state.clone();
    let url = req.url.clone();
    let format = req.format.clone();
    
    tokio::spawn(async move {
        let output_path = get_download_path();
        let audio_format = match format.as_str() {
            "mp3" => ytdlp::formats::AudioFormat::Mp3,
            "m4a" => ytdlp::formats::AudioFormat::M4a,
            "flac" => ytdlp::formats::AudioFormat::Flac,
            "wav" => ytdlp::formats::AudioFormat::Wav,
            _ => ytdlp::formats::AudioFormat::Mp3,
        };
        
        let _ = ytdlp::download::download_audio(&url, &output_path, audio_format, None, None);
        
        let mut downloads = state_clone.downloads.lock().unwrap();
        if let Some(state) = downloads.get_mut(&download_id) {
            state.status = "Completed".to_string();
            state.progress = 100.0;
        }
    });

    Json(DownloadResponse {
        id: Some(id),
        error: None,
    })
}

async fn download_thumbnail_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<DownloadRequest>,
) -> Json<DownloadResponse> {
    let id = uuid::Uuid::new_v4().to_string();
    let download_id = id.clone();
    
    {
        let mut downloads = state.downloads.lock().unwrap();
        downloads.insert(id.clone(), DownloadState {
            id: id.clone(),
            title: "Thumbnail".to_string(),
            status: "Downloading".to_string(),
            progress: 50.0,
            eta: None,
        });
    }

    let state_clone = state.clone();
    let url = req.url.clone();
    
    tokio::spawn(async move {
        let output_path = get_download_path();
        let _ = ytdlp::download::download_thumbnail(&url, &output_path);
        
        let mut downloads = state_clone.downloads.lock().unwrap();
        if let Some(state) = downloads.get_mut(&download_id) {
            state.status = "Completed".to_string();
            state.progress = 100.0;
        }
    });

    Json(DownloadResponse {
        id: Some(id),
        error: None,
    })
}

async fn download_playlist_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Json(req): Json<PlaylistDownloadRequest>,
) -> Json<DownloadResponse> {
    let id = uuid::Uuid::new_v4().to_string();
    let download_id = id.clone();
    
    let title = match ytdlp::formats::get_playlist_info(&req.url) {
        Ok(info) => info.title,
        Err(_) => "Playlist".to_string(),
    };

    {
        let mut downloads = state.downloads.lock().unwrap();
        downloads.insert(id.clone(), DownloadState {
            id: id.clone(),
            title,
            status: "Starting".to_string(),
            progress: 0.0,
            eta: None,
        });
    }

    let state_clone = state.clone();
    let url = req.url.clone();
    let preset = req.preset.clone();
    
    tokio::spawn(async move {
        let output_path = get_download_path();
        let quality_preset = match preset.as_str() {
            "god" => ytdlp::formats::QualityPreset::God,
            "ultra" => ytdlp::formats::QualityPreset::Ultra,
            "pro" => ytdlp::formats::QualityPreset::Pro,
            "high" => ytdlp::formats::QualityPreset::High,
            "medium" => ytdlp::formats::QualityPreset::Medium,
            _ => ytdlp::formats::QualityPreset::High,
        };
        
        let audio_only = preset == "audio";
        let audio_format = if audio_only {
            Some(ytdlp::formats::AudioFormat::Mp3)
        } else {
            None
        };
        
        let _ = ytdlp::download::download_playlist(
            &url,
            &output_path,
            "Playlist",
            quality_preset,
            audio_only,
            audio_format,
            4, // parallel downloads
        );
        
        let mut downloads = state_clone.downloads.lock().unwrap();
        if let Some(state) = downloads.get_mut(&download_id) {
            state.status = "Completed".to_string();
            state.progress = 100.0;
        }
    });

    Json(DownloadResponse {
        id: Some(id),
        error: None,
    })
}

async fn progress_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    Query(query): Query<ProgressQuery>,
) -> Json<ProgressResponse> {
    let downloads = state.downloads.lock().unwrap();
    
    if let Some(download) = downloads.get(&query.id) {
        Json(ProgressResponse {
            status: download.status.clone(),
            progress: download.progress,
            eta: download.eta.clone(),
        })
    } else {
        Json(ProgressResponse {
            status: "Unknown".to_string(),
            progress: 0.0,
            eta: None,
        })
    }
}

// === Helpers ===

fn get_download_path() -> String {
    dirs::download_dir()
        .or_else(|| dirs::home_dir().map(|p| p.join("Downloads")))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ONYX")
        .to_string_lossy()
        .to_string()
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}
