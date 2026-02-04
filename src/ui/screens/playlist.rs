use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::KeyCode;

use crate::ui::colors;
use crate::ytdlp::formats::{AudioFormat, PlaylistInfo, QualityPreset};
use super::{InputField, SelectionList, ProgressBar, Spinner};

/// Playlist download screen state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaylistState {
    UrlInput,
    FetchingInfo,
    PresetSelect,
    AudioFormatSelect,
    Downloading,
    Done,
}

/// Playlist download screen
pub struct PlaylistScreen {
    pub state: PlaylistState,
    pub input: InputField,
    pub playlist_info: Option<PlaylistInfo>,
    pub preset_options: SelectionList<QualityPreset>,
    pub audio_options: SelectionList<AudioFormat>,
    pub progress: ProgressBar,
    pub current_video: usize,
    pub total_videos: usize,
    pub is_audio_mode: bool,
    pub spinner: Spinner,
    pub status_message: Option<(String, bool)>,
}

impl Default for PlaylistScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistScreen {
    pub fn new() -> Self {
        let mut preset_options = SelectionList::new("Select Quality Preset");
        preset_options.set_items(vec![
            (QualityPreset::God.label().to_string(), QualityPreset::God),
            (QualityPreset::Ultra.label().to_string(), QualityPreset::Ultra),
            (QualityPreset::Pro.label().to_string(), QualityPreset::Pro),
            (QualityPreset::High.label().to_string(), QualityPreset::High),
            (QualityPreset::Medium.label().to_string(), QualityPreset::Medium),
            (QualityPreset::Low.label().to_string(), QualityPreset::Low),
            ("───────────────".to_string(), QualityPreset::God), // separator
            ("🎵 Audio Only (MP3)".to_string(), QualityPreset::God),
        ]);

        let mut audio_options = SelectionList::new("Select Audio Quality");
        audio_options.set_items(vec![
            ("🔊 320kbps (Best)".to_string(), AudioFormat::Mp3),
            ("🔊 256kbps".to_string(), AudioFormat::Mp3),
            ("🔊 192kbps".to_string(), AudioFormat::Mp3),
            ("🔊 128kbps".to_string(), AudioFormat::Mp3),
        ]);

        Self {
            state: PlaylistState::UrlInput,
            input: InputField::new("📦 Enter Playlist URL:"),
            playlist_info: None,
            preset_options,
            audio_options,
            progress: ProgressBar::new("Downloading playlist..."),
            current_video: 0,
            total_videos: 0,
            is_audio_mode: false,
            spinner: Spinner::new(),
            status_message: None,
        }
    }

    pub fn reset(&mut self) {
        self.state = PlaylistState::UrlInput;
        self.input.clear();
        self.playlist_info = None;
        self.progress.progress = 0.0;
        self.current_video = 0;
        self.total_videos = 0;
        self.is_audio_mode = false;
        self.status_message = None;
    }

    pub fn set_playlist_info(&mut self, info: PlaylistInfo) {
        self.total_videos = info.entries.len();
        self.playlist_info = Some(info);
        self.state = PlaylistState::PresetSelect;
    }

    pub fn selected_preset(&self) -> Option<QualityPreset> {
        self.preset_options.selected_item().copied()
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            PlaylistState::UrlInput => {
                self.input.handle_key(key);
            }
            PlaylistState::PresetSelect => match key {
                KeyCode::Up => self.preset_options.previous(),
                KeyCode::Down => self.preset_options.next(),
                _ => {}
            },
            PlaylistState::AudioFormatSelect => match key {
                KeyCode::Up => self.audio_options.previous(),
                KeyCode::Down => self.audio_options.next(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.state {
            PlaylistState::UrlInput => self.render_input(area, buf),
            PlaylistState::FetchingInfo => self.render_fetching(area, buf),
            PlaylistState::PresetSelect => self.render_preset(area, buf),
            PlaylistState::AudioFormatSelect => self.render_audio_format(area, buf),
            PlaylistState::Downloading => self.render_downloading(area, buf),
            PlaylistState::Done => self.render_done(area, buf),
        }
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        self.input.render(area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "Enter to fetch playlist • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_fetching(&self, area: Rect, buf: &mut Buffer) {
        self.spinner.render(area, buf, "Fetching playlist information...");
    }

    fn render_preset(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref info) = self.playlist_info {
            // Playlist info
            let title = format!("📦 {}", info.title);
            buf.set_string(
                area.x,
                area.y,
                &title,
                Style::default().fg(colors::WHITE).bold(),
            );

            buf.set_string(
                area.x,
                area.y + 1,
                &format!("📊 {} videos", info.entries.len()),
                Style::default().fg(colors::GRAY),
            );

            // Show first 5 videos
            let preview_y = area.y + 3;
            let max_show = 5.min(info.entries.len());
            for (i, entry) in info.entries.iter().take(max_show).enumerate() {
                let title = if entry.title.len() > 40 {
                    format!("{}...", &entry.title[..40])
                } else {
                    entry.title.clone()
                };
                buf.set_string(
                    area.x + 2,
                    preview_y + i as u16,
                    &format!("• {}", title),
                    Style::default().fg(colors::GRAY),
                );
            }

            if info.entries.len() > 5 {
                buf.set_string(
                    area.x + 2,
                    preview_y + 5,
                    &format!("  ... and {} more", info.entries.len() - 5),
                    Style::default().fg(colors::GRAY).italic(),
                );
            }
        }

        // Quality preset selection
        let list_area = Rect {
            x: area.x,
            y: area.y + 10,
            width: area.width.min(40),
            height: area.height.saturating_sub(12),
        };
        self.preset_options.render(list_area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to download • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_audio_format(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "🎵 Select audio quality:",
            Style::default().fg(colors::CYAN).bold(),
        );

        let list_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width.min(30),
            height: area.height.saturating_sub(4),
        };
        self.audio_options.render(list_area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to download • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_downloading(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref info) = self.playlist_info {
            buf.set_string(
                area.x,
                area.y,
                &format!("📥 Downloading: {}", info.title),
                Style::default().fg(colors::CYAN).bold(),
            );

            buf.set_string(
                area.x,
                area.y + 1,
                &format!("Video {}/{}", self.current_video, self.total_videos),
                Style::default().fg(colors::GRAY),
            );
        }

        let progress_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width,
            height: 4,
        };
        self.progress.render(progress_area, buf);
    }

    fn render_done(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "✅ Playlist Download Complete!",
            Style::default().fg(colors::GREEN).bold(),
        );

        buf.set_string(
            area.x,
            area.y + 2,
            &format!("Downloaded {} videos", self.total_videos),
            Style::default().fg(colors::WHITE),
        );

        if let Some((ref msg, is_error)) = self.status_message {
            let color = if is_error { colors::RED } else { colors::WHITE };
            let status_area = Rect {
                x: area.x,
                y: area.y + 3,
                width: area.width,
                height: area.height.saturating_sub(5),
            };
            let p = Paragraph::new(msg.clone())
                .style(Style::default().fg(color))
                .wrap(Wrap { trim: true });
            p.render(status_area, buf);
        }

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "Press Enter to continue",
            Style::default().fg(colors::GRAY),
        );
    }
}
