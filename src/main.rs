mod app;
mod config;
mod deps;
mod ui;
mod web;
mod ytdlp;

use std::env;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::*,
};

use app::{App, Screen};
use ui::banner::BannerWidget;
use ui::colors;
use ytdlp::download::DownloadProgress;
use ui::screens::video::VideoState;
use ui::screens::audio::AudioState;
use ui::screens::playlist::PlaylistState;
use ui::screens::thumbnail::ThumbnailState;

#[derive(Debug)]
pub enum AppEvent {
    Deps(Result<crate::deps::DependencyStatus>),
    Progress(DownloadProgress),
    SearchResults(Result<Vec<ytdlp::search::SearchResult>>),
    VideoInfo(Result<ytdlp::formats::VideoInfo>),
    PlaylistInfo(Result<ytdlp::formats::PlaylistInfo>),
    ThumbnailInfo(Result<ytdlp::formats::VideoInfo>),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check for web mode
    if args.iter().any(|a| a == "--web" || a == "-w") {
        // Get port from args or default to 3000
        let port = args.iter()
            .position(|a| a == "--port" || a == "-p")
            .and_then(|i| args.get(i + 1))
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        
        return web::start_server(port).await;
    }
    
    // Run TUI mode
    run_tui()
}

fn run_tui() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Create app and run

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut ratatui::Terminal<B>, app: &mut App) -> Result<()> {
    // Event channel
    let (tx, rx) = mpsc::channel();

    // Initial dependency check in background
    let deps_tx = tx.clone();
    thread::spawn(move || {
        let result = deps::ensure_dependencies(|_| {});
        let _ = deps_tx.send(AppEvent::Deps(result));
    });

    // Track previous screen for detecting transitions
    let mut prev_screen = app.screen;

    loop {
        // Check if we just switched to WebServer screen
        if app.screen == Screen::WebServer && prev_screen != Screen::WebServer && !app.web_server_running {
            let port = app.web_server_port;
            app.web_server_running = true;
            
            // Start web server in a separate thread with its own runtime
            thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let _ = web::start_server(port).await;
                });
            });
        }
        prev_screen = app.screen;

        // Draw UI
        terminal.draw(|f| draw_ui(f, app))?;

        // Check for events
        while let Ok(event) = rx.try_recv() {
            match event {
                AppEvent::Deps(result) => {
                    if app.screen == Screen::Loading {
                        match result {
                            Ok(deps) => app.set_deps(deps),
                            Err(e) => app.set_loading_message(&format!("Error: {}", e)),
                        }
                    }
                }
                AppEvent::Progress(progress) => {
                    handle_progress(app, progress);
                }
                AppEvent::SearchResults(result) => {
                    match result {
                        Ok(results) => app.search.set_results(results),
                        Err(e) => {
                            app.search.is_loading = false;
                            app.search.status_message = Some((format!("Search failed: {}", e), true));
                        }
                    }
                }
                AppEvent::VideoInfo(result) => {
                    match result {
                        Ok(info) => {
                            match app.screen {
                                Screen::Video => app.video.set_video_info(info),
                                Screen::Audio => app.audio.set_video_info(info),
                                Screen::Search => {
                                    app.search.is_loading = false;
                                    let options = build_quality_options(&info);
                                    app.search.set_quality_options(options);
                                }
                                _ => {}
                            }
                        }
                        Err(e) => {
                            match app.screen {
                                Screen::Video => {
                                    app.video.state = ui::screens::video::VideoState::UrlInput;
                                    app.video.status_message = Some((format!("Failed: {}", e), true));
                                }
                                Screen::Audio => {
                                    app.audio.state = ui::screens::audio::AudioState::UrlInput;
                                    app.audio.status_message = Some((format!("Failed: {}", e), true));
                                }
                                Screen::Search => {
                                    app.search.is_loading = false;
                                    app.search.status_message = Some((format!("Failed: {}", e), true));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                AppEvent::PlaylistInfo(result) => {
                    match result {
                        Ok(info) => app.playlist.set_playlist_info(info),
                        Err(e) => {
                            app.playlist.state = ui::screens::playlist::PlaylistState::UrlInput;
                            app.playlist.status_message = Some((format!("Failed: {}", e), true));
                        }
                    }
                }
                AppEvent::ThumbnailInfo(result) => {
                    match result {
                        Ok(info) => app.thumbnail.set_video_title(info.title),
                        Err(e) => {
                            app.thumbnail.state = ui::screens::thumbnail::ThumbnailState::UrlInput;
                            app.thumbnail.status_message = Some((format!("Failed: {}", e), true));
                        }
                    }
                }
            }
        }

        // Tick app for animations
        app.tick();

        // Handle events with timeout for async operations
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Check for Ctrl+C first
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        app.should_quit = true;
                        return Ok(());
                    }

                    // Check if we are in a loading state
                    let is_loading = match app.screen {
                        Screen::Loading => true,
                        Screen::Search => app.search.is_loading,
                        Screen::Video => app.video.state == VideoState::FetchingInfo,
                        Screen::Audio => app.audio.state == AudioState::FetchingInfo,
                        Screen::Playlist => app.playlist.state == PlaylistState::FetchingInfo,
                        Screen::Thumbnail => app.thumbnail.state == ThumbnailState::Fetching,
                        _ => false,
                    };

                    if is_loading {
                        // Block all other keys
                        continue;
                    }

                    // Handle special async operations
                    if !handle_async_operations(app, key.code, tx.clone())? {
                        app.handle_key(key.code);
                    }

                    if app.should_quit {
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// Handle operations that require async execution
fn handle_async_operations(app: &mut App, key: KeyCode, tx: mpsc::Sender<AppEvent>) -> Result<bool> {
    use ui::screens::search::SearchState;
    use ui::screens::video::VideoState;
    use ui::screens::audio::AudioState;
    use ui::screens::playlist::PlaylistState;
    use ui::screens::thumbnail::ThumbnailState;

    if key != KeyCode::Enter {
        return Ok(false);
    }

    match app.screen {
        Screen::Search => {
            match app.search.state {
                SearchState::Input if !app.search.input.value.is_empty() => {
                    let query = app.search.input.value.clone();
                    app.search.is_loading = true;

                    // Perform search in background
                    let tx = tx.clone();
                    thread::spawn(move || {
                        let result = ytdlp::search::search_youtube(&query, 10);
                        let _ = tx.send(AppEvent::SearchResults(result));
                    });
                    return Ok(true);
                }
                SearchState::Results => {
                    if let Some(video) = app.search.selected_video() {
                        let url = video.get_url();
                        app.search.is_loading = true;

                        let tx = tx.clone();
                        thread::spawn(move || {
                            let result = ytdlp::formats::get_video_info(&url);
                            let _ = tx.send(AppEvent::VideoInfo(result));
                        });
                    }
                    return Ok(true);
                }
                SearchState::QualitySelect => {
                    let format_opt = app.search.quality_options.selected_item().cloned();
                    let video_url = app.search.selected_video().map(|v| v.get_url());
                    let output_path = app.config.download_path.clone();

                    if let (Some(format), Some(url)) = (format_opt, video_url) {
                        start_download(app, &url, &output_path, Some(&format), tx)?;
                    }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Screen::Video => {
            match app.video.state {
                VideoState::UrlInput if !app.video.input.value.is_empty() => {
                    let url = app.video.input.value.clone();
                    app.video.state = VideoState::FetchingInfo;

                    let tx = tx.clone();
                    thread::spawn(move || {
                        let result = ytdlp::formats::get_video_info(&url);
                        let _ = tx.send(AppEvent::VideoInfo(result));
                    });
                    return Ok(true);
                }
                VideoState::QualitySelect => {
                    let format_opt = app.video.quality_options.selected_item().cloned();
                    let url = app.video.input.value.clone();
                    let output_path = app.config.download_path.clone();

                    if let Some(format) = format_opt {
                        if format == "mp3" {
                            start_audio_download(app, &url, &output_path, ytdlp::formats::AudioFormat::Mp3, tx)?;
                        } else if format != "separator" {
                            start_download(app, &url, &output_path, Some(&format), tx)?;
                        }
                    }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Screen::Audio => {
            match app.audio.state {
                AudioState::UrlInput if !app.audio.input.value.is_empty() => {
                    let url = app.audio.input.value.clone();
                    app.audio.state = AudioState::FetchingInfo;

                    let tx = tx.clone();
                    thread::spawn(move || {
                        let result = ytdlp::formats::get_video_info(&url);
                        let _ = tx.send(AppEvent::VideoInfo(result));
                    });
                    return Ok(true);
                }
                AudioState::QualitySelect => {
                    if let Some(quality) = app.audio.quality_options.selected_item() {
                        app.audio.selected_quality = Some(quality.clone());
                        app.audio.state = AudioState::FormatSelect;
                    }
                    return Ok(true);
                }
                AudioState::FormatSelect => {
                    if let Some(format) = app.audio.selected_format() {
                        let url = app.audio.input.value.clone();
                        let output_path = app.config.download_path.clone();
                        start_audio_download(app, &url, &output_path, format, tx)?;
                    }
                    return Ok(true);
                }
                _ => {}
            }
        }
        Screen::Playlist => {
            match app.playlist.state {
                PlaylistState::UrlInput if !app.playlist.input.value.is_empty() => {
                    let url = app.playlist.input.value.clone();
                    app.playlist.state = PlaylistState::FetchingInfo;

                    let tx = tx.clone();
                    thread::spawn(move || {
                        let result = ytdlp::formats::get_playlist_info(&url);
                        let _ = tx.send(AppEvent::PlaylistInfo(result));
                    });
                    return Ok(true);
                }
                PlaylistState::PresetSelect => {
                    // Check if audio option selected (last item with Audio in label)
                    let selected_idx = app.playlist.preset_options.selected;
                    if selected_idx == 7 {
                        // Audio only option
                        app.playlist.is_audio_mode = true;
                        app.playlist.state = PlaylistState::AudioFormatSelect;
                    } else if selected_idx != 6 {
                        // Not separator
                        if let Some(preset) = app.playlist.selected_preset() {
                            let url = app.playlist.input.value.clone();
                            let output_path = app.config.download_path.clone();
                            let playlist_name = app.playlist.playlist_info
                                .as_ref()
                                .map(|p| p.title.clone())
                                .unwrap_or_else(|| "Playlist".to_string());

                            start_playlist_download(app, &url, &output_path, &playlist_name, preset, tx)?;
                        }
                    }
                    return Ok(true);
                }
                PlaylistState::AudioFormatSelect => {
                    let url = app.playlist.input.value.clone();
                    let output_path = app.config.download_path.clone();
                    let playlist_name = app.playlist.playlist_info
                        .as_ref()
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| "Playlist".to_string());

                    start_playlist_audio_download(app, &url, &output_path, &playlist_name, tx)?;
                    return Ok(true);
                }
                _ => {}
            }
        }
        Screen::Thumbnail => {
            match app.thumbnail.state {
                ThumbnailState::UrlInput if !app.thumbnail.input.value.is_empty() => {
                    let url = app.thumbnail.input.value.clone();
                    app.thumbnail.state = ThumbnailState::Fetching;

                    let tx = tx.clone();
                    thread::spawn(move || {
                        let result = ytdlp::formats::get_video_info(&url);
                        let _ = tx.send(AppEvent::ThumbnailInfo(result));
                    });
                    return Ok(true);
                }
                ThumbnailState::SelectResolution => {
                    let url = app.thumbnail.input.value.clone();
                    let output_path = app.config.download_path.clone();
                    app.thumbnail.state = ThumbnailState::Downloading;

                    match ytdlp::download::download_thumbnail(&url, &output_path) {
                        Ok(files) => {
                            app.thumbnail.downloaded_files = files;
                            app.thumbnail.state = ThumbnailState::Done;
                        }
                        Err(e) => {
                            app.thumbnail.state = ThumbnailState::SelectResolution;
                            app.thumbnail.status_message = Some((format!("Failed: {}", e), true));
                        }
                    }
                    return Ok(true);
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(false)
}

fn build_quality_options(info: &ytdlp::formats::VideoInfo) -> Vec<(String, String)> {
    let mut options = Vec::new();

    let video_formats = info.get_video_formats();
    for (label, formats) in &video_formats {
        if let Some(fmt) = formats.first() {
            let size = fmt.get_size_str();
            let option_label = format!("🎥 {} ({})", label, size);
            let format_string = format!("bestvideo[height<={}]+bestaudio/best", fmt.height.unwrap_or(1080));
            options.push((option_label, format_string));
        }
    }

    // Add MP3 option
    options.push(("───────────────".to_string(), "separator".to_string()));
    options.push(("🎵 MP3 (Best Quality)".to_string(), "mp3".to_string()));

    options
}

fn start_download(
    app: &mut App, 
    url: &str, 
    output_path: &str, 
    format: Option<&str>,
    tx: mpsc::Sender<AppEvent>
) -> Result<()> {
    use ui::screens::video::VideoState;

    let url = url.to_string();
    let output_path = output_path.to_string();
    let format = format.map(|s| s.to_string());

    // Update UI state
    if app.screen == Screen::Video {
        app.video.state = VideoState::Downloading;
        app.video.progress.progress = 0.0;
        app.video.status_message = None;
    } else if app.screen == Screen::Search {
        app.search.is_loading = true;
        app.search.status_message = None;
    }

    // Spawn download thread
    thread::spawn(move || {
        let (prog_tx, prog_rx) = mpsc::channel();
        
        // Forward progress events
        let event_tx = tx.clone();
        thread::spawn(move || {
            while let Ok(progress) = prog_rx.recv() {
                let _ = event_tx.send(AppEvent::Progress(progress));
            }
        });

        let result = ytdlp::download::download_video(
            &url,
            &output_path,
            format.as_deref(),
            Some(prog_tx)
        );

        if let Err(e) = result {
            let _ = tx.send(AppEvent::Progress(DownloadProgress::Error(e.to_string())));
        }
    });

    Ok(())
}

fn start_audio_download(
    app: &mut App, 
    url: &str, 
    output_path: &str, 
    format: ytdlp::formats::AudioFormat,
    tx: mpsc::Sender<AppEvent>
) -> Result<()> {
    use ui::screens::audio::AudioState;
    use ui::screens::video::VideoState;

    // Update state
    if app.screen == Screen::Audio {
        app.audio.state = AudioState::Downloading;
        app.audio.progress.progress = 0.0;
        app.audio.status_message = None;
    } else if app.screen == Screen::Video {
        app.video.state = VideoState::Downloading;
        app.video.progress.progress = 0.0;
        app.video.status_message = None;
    }

    let url = url.to_string();
    let output_path = output_path.to_string();

    thread::spawn(move || {
        let (prog_tx, prog_rx) = mpsc::channel();
        let event_tx = tx.clone();
        
        thread::spawn(move || {
            while let Ok(progress) = prog_rx.recv() {
                let _ = event_tx.send(AppEvent::Progress(progress));
            }
        });

        let result = ytdlp::download::download_audio(
            &url,
            &output_path,
            format,
            None,
            Some(prog_tx)
        );

        if let Err(e) = result {
            let _ = tx.send(AppEvent::Progress(DownloadProgress::Error(e.to_string())));
        }
    });

    Ok(())
}

fn start_playlist_download(
    app: &mut App,
    url: &str,
    output_path: &str,
    playlist_name: &str,
    preset: ytdlp::formats::QualityPreset,
    tx: mpsc::Sender<AppEvent>
) -> Result<()> {
    use ui::screens::playlist::PlaylistState;

    app.playlist.state = PlaylistState::Downloading;
    app.playlist.status_message = None;

    let url = url.to_string();
    let output_path = output_path.to_string();
    let playlist_name = playlist_name.to_string();

    thread::spawn(move || {
        // Playlists don't support granular progress yet in this impl, but we can simulate start/end
        // Or technically we could wrap it. For now, assume simple blocking within thread
        
        let result = ytdlp::download::download_playlist(
            &url,
            &output_path,
            &playlist_name,
            preset,
            false,
            None,
            4
        );

        if let Err(e) = result {
            let _ = tx.send(AppEvent::Progress(DownloadProgress::Error(e.to_string())));
        } else {
            let _ = tx.send(AppEvent::Progress(DownloadProgress::Finished("Playlist".to_string())));
        }
    });

    Ok(())
}

fn start_playlist_audio_download(
    app: &mut App,
    url: &str,
    output_path: &str,
    playlist_name: &str,
    tx: mpsc::Sender<AppEvent>
) -> Result<()> {
    use ui::screens::playlist::PlaylistState;

    app.playlist.state = PlaylistState::Downloading;
    app.playlist.status_message = None;

    let url = url.to_string();
    let output_path = output_path.to_string();
    let playlist_name = playlist_name.to_string();

    thread::spawn(move || {
        let result = ytdlp::download::download_playlist(
            &url,
            &output_path,
            &playlist_name,
            ytdlp::formats::QualityPreset::God, // Dummy
            true,
            Some(ytdlp::formats::AudioFormat::Mp3),
            4
        );

        if let Err(e) = result {
            let _ = tx.send(AppEvent::Progress(DownloadProgress::Error(e.to_string())));
        } else {
            let _ = tx.send(AppEvent::Progress(DownloadProgress::Finished("Playlist".to_string())));
        }
    });

    Ok(())
}

fn handle_progress(app: &mut App, progress: DownloadProgress) {
    use ui::screens::video::VideoState;
    use ui::screens::audio::AudioState;
    use ui::screens::playlist::PlaylistState;
    use ui::screens::search::SearchState;

    match progress {
        DownloadProgress::Starting(_) => {
            // Already handled in start_X
        }
        DownloadProgress::Downloading { percent, speed, eta } => {
            let status = format!("Speed: {} • ETA: {}", speed, eta);
            if app.screen == Screen::Video {
                app.video.progress.set_progress(percent);
                app.video.progress.status = status;
            } else if app.screen == Screen::Audio {
                app.audio.progress.set_progress(percent);
                app.audio.progress.status = status;
            }
        }
        DownloadProgress::PostProcessing => {
             if app.screen == Screen::Video {
                app.video.progress.status = "Processing...".to_string();
            } else if app.screen == Screen::Audio {
                app.audio.progress.status = "Converting...".to_string();
            }
        }
        DownloadProgress::Finished(name) => {
             if app.screen == Screen::Video {
                app.video.state = VideoState::Done;
                app.video.progress.progress = 100.0;
                app.video.status_message = Some((format!("Saved to: {}", name), false));
            } else if app.screen == Screen::Audio {
                app.audio.state = AudioState::Done;
                app.audio.progress.progress = 100.0;
                app.audio.status_message = Some((format!("Saved to: {}", name), false));
            } else if app.screen == Screen::Playlist {
                app.playlist.state = PlaylistState::Done;
                 app.playlist.progress.progress = 100.0;
                app.playlist.status_message = Some(("Playlist download complete".to_string(), false));
            } else if app.screen == Screen::Search {
                app.search.state = SearchState::Results;
                app.search.is_loading = false;
                app.search.status_message = Some(("Download complete".to_string(), false));
            }
        }
        DownloadProgress::Error(msg) => {
             if app.screen == Screen::Video {
                app.video.state = VideoState::Done;
                app.video.status_message = Some((format!("Error: {}", msg), true));
            } else if app.screen == Screen::Audio {
                app.audio.state = AudioState::Done;
                app.audio.status_message = Some((format!("Error: {}", msg), true));
            } else if app.screen == Screen::Playlist {
                app.playlist.state = PlaylistState::Done;
                app.playlist.status_message = Some((format!("Error: {}", msg), true));
            } else if app.screen == Screen::Search {
                 app.search.state = SearchState::Results;
                 app.search.is_loading = false;
                 app.search.status_message = Some((format!("Error: {}", msg), true));
            }
        }
    }
}


fn draw_ui(f: &mut Frame, app: &App) {
    let area = f.area();

    // Clear with dark background
    f.render_widget(
        Block::default().style(Style::default().bg(colors::DARKER)),
        area,
    );

    match app.screen {
        Screen::Loading => draw_loading(f, app, area),
        Screen::Menu => draw_menu(f, app, area),
        Screen::Search => draw_screen(f, app, area, "Search & Download"),
        Screen::Video => draw_screen(f, app, area, "Download Video"),
        Screen::Audio => draw_screen(f, app, area, "Download Audio"),
        Screen::Playlist => draw_screen(f, app, area, "Download Playlist"),
        Screen::Thumbnail => draw_screen(f, app, area, "Download Thumbnail"),
        Screen::Settings => draw_screen(f, app, area, "Settings"),
        Screen::WebServer => draw_web_server_screen(f, app, area),
    }
}

fn draw_web_server_screen(f: &mut Frame, app: &App, area: Rect) {
    // Draw banner
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(area);

    // Banner
    let banner = BannerWidget;
    f.render_widget(banner, chunks[0]);

    // Main content
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(5),
            Constraint::Length(2),
            Constraint::Min(3),
        ])
        .margin(2)
        .split(chunks[1]);

    // Title
    let title = Paragraph::new(Span::styled(
        "🌐 Web Interface",
        Style::default().fg(colors::CYAN).bold().add_modifier(Modifier::BOLD),
    ));
    f.render_widget(title, content_chunks[0]);

    // Server status
    let status_text = if app.web_server_running {
        format!("✅ Server is running on port {}", app.web_server_port)
    } else {
        "⏳ Starting web server...".to_string()
    };
    let status = Paragraph::new(Span::styled(
        status_text,
        Style::default().fg(colors::GREEN),
    ));
    f.render_widget(status, content_chunks[1]);

    // URL box
    let url = format!("http://localhost:{}", app.web_server_port);
    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::PINK))
        .title(Span::styled(" Open in Browser ", Style::default().fg(colors::PINK).bold()));
    let url_text = Paragraph::new(vec![
        Line::from(Span::styled(&url, Style::default().fg(colors::WHITE).bold())),
        Line::from(""),
        Line::from(Span::styled("Copy this URL and paste in your browser", Style::default().fg(colors::GRAY))),
    ])
    .block(url_block)
    .alignment(Alignment::Center);
    f.render_widget(url_text, content_chunks[2]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(Span::styled("The web interface provides:", Style::default().fg(colors::WHITE))),
        Line::from(Span::styled("  • Beautiful search with thumbnails", Style::default().fg(colors::GRAY))),
        Line::from(Span::styled("  • Click-to-download with quality selection", Style::default().fg(colors::GRAY))),
        Line::from(Span::styled("  • Video, Audio & Thumbnail downloads", Style::default().fg(colors::GRAY))),
        Line::from(Span::styled("  • Playlist support", Style::default().fg(colors::GRAY))),
    ]);
    f.render_widget(instructions, content_chunks[4]);

    // Footer
    let footer = Paragraph::new(Span::styled(
        "Press ESC or 'q' to go back (server will keep running)",
        Style::default().fg(colors::GRAY),
    ))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn draw_loading(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(10),
            Constraint::Min(1),
        ])
        .split(area);

    // Banner
    f.render_widget(BannerWidget, chunks[1]);

    // Loading message
    let loading = Paragraph::new(app.loading_message.clone())
        .style(Style::default().fg(colors::YELLOW))
        .alignment(Alignment::Center);

    // Use the third chunk for the loading message, with some padding if possible
    let loading_area = chunks[2];
    f.render_widget(loading, loading_area);
}

fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(area);

    // Banner
    f.render_widget(BannerWidget, chunks[0]);

    // Menu
    let menu_area = Rect {
        x: chunks[1].x + 2,
        y: chunks[1].y + 1,
        width: chunks[1].width.saturating_sub(4),
        height: chunks[1].height.saturating_sub(2),
    };
    f.render_widget(&app.menu, menu_area);

    // Footer
    let footer = Paragraph::new("↑↓ navigate  •  ⏎ select")
        .style(Style::default().fg(colors::GRAY))
        .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}

fn draw_screen(f: &mut Frame, app: &App, area: Rect, title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
        ])
        .split(area);

    // Title bar
    let title_block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(colors::BLUE));
    let title_text = Paragraph::new(format!("  {}  ", title))
        .style(Style::default().fg(colors::CYAN).bold())
        .block(title_block);
    f.render_widget(title_text, chunks[0]);

    // Content area
    let content_area = Rect {
        x: chunks[1].x + 2,
        y: chunks[1].y + 1,
        width: chunks[1].width.saturating_sub(4),
        height: chunks[1].height.saturating_sub(2),
    };

    // Render screen-specific content
    match app.screen {
        Screen::Search => app.search.render(content_area, f.buffer_mut()),
        Screen::Video => app.video.render(content_area, f.buffer_mut()),
        Screen::Audio => app.audio.render(content_area, f.buffer_mut()),
        Screen::Playlist => app.playlist.render(content_area, f.buffer_mut()),
        Screen::Thumbnail => app.thumbnail.render(content_area, f.buffer_mut()),
        Screen::Settings => app.settings.render(content_area, f.buffer_mut()),
        _ => {}
    }
}
