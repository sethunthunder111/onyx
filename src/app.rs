use crossterm::event::KeyCode;

use crate::config::Config;
use crate::deps::DependencyStatus;
use crate::ui::menu::{MainMenu, MenuAction};
use crate::ui::screens::search::SearchScreen;
use crate::ui::screens::video::VideoScreen;
use crate::ui::screens::audio::AudioScreen;
use crate::ui::screens::playlist::PlaylistScreen;
use crate::ui::screens::thumbnail::ThumbnailScreen;
use crate::ui::screens::settings::SettingsScreen;

/// Current application screen
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Loading,
    Menu,
    Search,
    Video,
    Audio,
    Playlist,
    Thumbnail,
    Settings,
    WebServer,
}

/// Main application state
pub struct App {
    pub screen: Screen,
    pub config: Config,
    pub deps: Option<DependencyStatus>,
    pub loading_message: String,
    pub should_quit: bool,

    // Screen states
    pub menu: MainMenu,
    pub search: SearchScreen,
    pub video: VideoScreen,
    pub audio: AudioScreen,
    pub playlist: PlaylistScreen,
    pub thumbnail: ThumbnailScreen,
    pub settings: SettingsScreen,
    pub web_server_running: bool,
    pub web_server_port: u16,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();

        Self {
            screen: Screen::Loading,
            config: config.clone(),
            deps: None,
            loading_message: "Initializing...".to_string(),
            should_quit: false,

            menu: MainMenu::new(),
            search: SearchScreen::new(),
            video: VideoScreen::new(),
            audio: AudioScreen::new(),
            playlist: PlaylistScreen::new(),
            thumbnail: ThumbnailScreen::new(),
            settings: SettingsScreen::new(config),
            web_server_running: false,
            web_server_port: 3000,
        }
    }

    pub fn set_deps(&mut self, deps: DependencyStatus) {
        self.deps = Some(deps);
        self.screen = Screen::Menu;
    }

    pub fn set_loading_message(&mut self, msg: &str) {
        self.loading_message = msg.to_string();
    }

    pub fn go_back(&mut self) {
        match self.screen {
            Screen::Search | Screen::Video | Screen::Audio | 
            Screen::Playlist | Screen::Thumbnail | Screen::Settings | Screen::WebServer => {
                self.screen = Screen::Menu;
                // Reset screens
                self.search.reset();
                self.video.reset();
                self.audio.reset();
                self.playlist.reset();
                self.thumbnail.reset();
                self.settings.reset();
            }
            _ => {}
        }
    }

    pub fn handle_menu_select(&mut self) {
        match self.menu.selected_action() {
            MenuAction::SearchDownload => {
                self.screen = Screen::Search;
                self.search.reset();
            }
            MenuAction::DownloadVideo => {
                self.screen = Screen::Video;
                self.video.reset();
            }
            MenuAction::DownloadAudio => {
                self.screen = Screen::Audio;
                self.audio.reset();
            }
            MenuAction::DownloadPlaylist => {
                self.screen = Screen::Playlist;
                self.playlist.reset();
            }
            MenuAction::DownloadThumbnail => {
                self.screen = Screen::Thumbnail;
                self.thumbnail.reset();
            }
            MenuAction::Settings => {
                self.screen = Screen::Settings;
                self.settings.update_config(self.config.clone());
            }
            MenuAction::WebInterface => {
                self.screen = Screen::WebServer;
            }
            MenuAction::Exit => {
                self.should_quit = true;
            }
            MenuAction::Separator => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.screen {
            Screen::Loading => {
                // No input during loading
            }
            Screen::Menu => match key {
                KeyCode::Up | KeyCode::Char('k') => self.menu.previous(),
                KeyCode::Down | KeyCode::Char('j') => self.menu.next(),
                KeyCode::Enter => self.handle_menu_select(),
                KeyCode::Esc | KeyCode::Char('q') => self.should_quit = true,
                _ => {}
            },
            Screen::Search => self.handle_search_key(key),
            Screen::Video => self.handle_video_key(key),
            Screen::Audio => self.handle_audio_key(key),
            Screen::Playlist => self.handle_playlist_key(key),
            Screen::Thumbnail => self.handle_thumbnail_key(key),
            Screen::Settings => self.handle_settings_key(key),
            Screen::WebServer => {
                if key == KeyCode::Esc || key == KeyCode::Char('q') {
                    self.go_back();
                }
            }
        }
    }

    fn handle_search_key(&mut self, key: KeyCode) {
        use crate::ui::screens::search::SearchState;

        match key {
            KeyCode::Esc => {
                match self.search.state {
                    SearchState::Input => self.go_back(),
                    SearchState::Results => self.search.state = SearchState::Input,
                    SearchState::QualitySelect => self.search.state = SearchState::Results,
                }
            }
            KeyCode::Enter => {
                // Handle enters based on state - actual logic will be in main.rs
            }
            _ => self.search.handle_key(key),
        }
    }

    fn handle_video_key(&mut self, key: KeyCode) {
        use crate::ui::screens::video::VideoState;

        match key {
            KeyCode::Esc => {
                match self.video.state {
                    VideoState::UrlInput => self.go_back(),
                    VideoState::QualitySelect => {
                        self.video.state = VideoState::UrlInput;
                        self.video.video_info = None;
                    }
                    VideoState::Done => self.go_back(),
                    _ => {}
                }
            }
            KeyCode::Enter if self.video.state == VideoState::Done => {
                self.go_back();
            }
            _ => self.video.handle_key(key),
        }
    }

    fn handle_audio_key(&mut self, key: KeyCode) {
        use crate::ui::screens::audio::AudioState;

        match key {
            KeyCode::Esc => {
                match self.audio.state {
                    AudioState::UrlInput => self.go_back(),
                    AudioState::QualitySelect => {
                        self.audio.state = AudioState::UrlInput;
                        self.audio.video_info = None;
                    }
                    AudioState::FormatSelect => {
                        self.audio.state = AudioState::QualitySelect;
                    }
                    AudioState::Done => self.go_back(),
                    _ => {}
                }
            }
            KeyCode::Enter if self.audio.state == AudioState::Done => {
                self.go_back();
            }
            _ => self.audio.handle_key(key),
        }
    }

    fn handle_playlist_key(&mut self, key: KeyCode) {
        use crate::ui::screens::playlist::PlaylistState;

        match key {
            KeyCode::Esc => {
                match self.playlist.state {
                    PlaylistState::UrlInput => self.go_back(),
                    PlaylistState::PresetSelect => {
                        self.playlist.state = PlaylistState::UrlInput;
                        self.playlist.playlist_info = None;
                    }
                    PlaylistState::AudioFormatSelect => {
                        self.playlist.state = PlaylistState::PresetSelect;
                    }
                    PlaylistState::Done => self.go_back(),
                    _ => {}
                }
            }
            KeyCode::Enter if self.playlist.state == PlaylistState::Done => {
                self.go_back();
            }
            _ => self.playlist.handle_key(key),
        }
    }

    fn handle_thumbnail_key(&mut self, key: KeyCode) {
        use crate::ui::screens::thumbnail::ThumbnailState;

        match key {
            KeyCode::Esc => {
                match self.thumbnail.state {
                    ThumbnailState::UrlInput => self.go_back(),
                    ThumbnailState::SelectResolution => {
                        self.thumbnail.state = ThumbnailState::UrlInput;
                    }
                    ThumbnailState::Done => self.go_back(),
                    _ => {}
                }
            }
            KeyCode::Enter if self.thumbnail.state == ThumbnailState::Done => {
                self.go_back();
            }
            _ => self.thumbnail.handle_key(key),
        }
    }

    fn handle_settings_key(&mut self, key: KeyCode) {
        use crate::ui::screens::settings::{SettingsOption, SettingsState};

        match key {
            KeyCode::Esc => {
                match self.settings.state {
                    SettingsState::Menu => self.go_back(),
                    _ => {
                        self.settings.state = SettingsState::Menu;
                    }
                }
            }
            KeyCode::Enter => {
                match self.settings.state {
                    SettingsState::Menu => {
                        match self.settings.selected_option() {
                            SettingsOption::DownloadPath => {
                                self.settings.state = SettingsState::EditingPath;
                            }
                            SettingsOption::ParallelDownloads => {
                                self.settings.state = SettingsState::EditingParallel;
                                self.settings.parallel_value = self.config.parallel_downloads;
                            }
                            SettingsOption::DebugMode => {
                                self.config.debug_mode = !self.config.debug_mode;
                                let _ = self.config.save();
                                self.settings.update_config(self.config.clone());
                                self.settings.status_message = Some((
                                    format!("Debug mode {}", if self.config.debug_mode { "enabled" } else { "disabled" }),
                                    false,
                                ));
                            }
                            SettingsOption::Back => {
                                self.go_back();
                            }
                        }
                    }
                    SettingsState::EditingPath => {
                        self.config.download_path = self.settings.path_input.value.clone();
                        let _ = self.config.save();
                        self.settings.state = SettingsState::Menu;
                        self.settings.update_config(self.config.clone());
                        self.settings.status_message = Some(("Download path updated!".to_string(), false));
                    }
                    SettingsState::EditingParallel => {
                        self.config.parallel_downloads = self.settings.parallel_value;
                        let _ = self.config.save();
                        self.settings.state = SettingsState::Menu;
                        self.settings.update_config(self.config.clone());
                        self.settings.status_message = Some(("Parallel downloads updated!".to_string(), false));
                    }
                }
            }
            _ => self.settings.handle_key(key),
        }
    }

    pub fn tick(&mut self) {
        match self.screen {
            Screen::Search => self.search.spinner.tick(),
            Screen::Video => self.video.spinner.tick(),
            Screen::Audio => self.audio.spinner.tick(),
            Screen::Playlist => self.playlist.spinner.tick(),
            Screen::Thumbnail => self.thumbnail.spinner.tick(),
            _ => {}
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
