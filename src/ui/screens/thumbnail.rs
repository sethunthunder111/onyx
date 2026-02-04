use ratatui::prelude::*;
use crossterm::event::KeyCode;

use crate::ui::colors;
use super::{InputField, Spinner};

/// Thumbnail download screen state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThumbnailState {
    UrlInput,
    Fetching,
    SelectResolution,
    Downloading,
    Done,
}

/// Thumbnail resolution options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThumbnailResolution {
    MaxRes,   // 1280x720
    High,     // 480x360
    Default,  // 120x90
    All,      // All resolutions
}

impl ThumbnailResolution {
    pub fn label(&self) -> &str {
        match self {
            ThumbnailResolution::MaxRes => "🖼️  Max Resolution (1280x720)",
            ThumbnailResolution::High => "🖼️  High Quality (480x360)",
            ThumbnailResolution::Default => "🖼️  Default (120x90)",
            ThumbnailResolution::All => "📦 All Resolutions",
        }
    }

    pub fn all() -> Vec<ThumbnailResolution> {
        vec![
            ThumbnailResolution::MaxRes,
            ThumbnailResolution::High,
            ThumbnailResolution::Default,
            ThumbnailResolution::All,
        ]
    }
}

/// Thumbnail download screen
pub struct ThumbnailScreen {
    pub state: ThumbnailState,
    pub input: InputField,
    pub video_title: Option<String>,
    pub resolutions: Vec<ThumbnailResolution>,
    pub selected: usize,
    pub downloaded_files: Vec<String>,
    pub spinner: Spinner,
    pub status_message: Option<(String, bool)>,
}

impl Default for ThumbnailScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl ThumbnailScreen {
    pub fn new() -> Self {
        Self {
            state: ThumbnailState::UrlInput,
            input: InputField::new("🖼️  Enter Video URL:"),
            video_title: None,
            resolutions: ThumbnailResolution::all(),
            selected: 0,
            downloaded_files: Vec::new(),
            spinner: Spinner::new(),
            status_message: None,
        }
    }

    pub fn reset(&mut self) {
        self.state = ThumbnailState::UrlInput;
        self.input.clear();
        self.video_title = None;
        self.selected = 0;
        self.downloaded_files.clear();
        self.status_message = None;
    }

    pub fn set_video_title(&mut self, title: String) {
        self.video_title = Some(title);
        self.state = ThumbnailState::SelectResolution;
    }

    #[allow(dead_code)]
    pub fn selected_resolution(&self) -> ThumbnailResolution {
        self.resolutions[self.selected]
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.resolutions.len();
    }

    pub fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.resolutions.len() - 1
        } else {
            self.selected - 1
        };
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            ThumbnailState::UrlInput => {
                self.input.handle_key(key);
            }
            ThumbnailState::SelectResolution => match key {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.state {
            ThumbnailState::UrlInput => self.render_input(area, buf),
            ThumbnailState::Fetching => self.render_fetching(area, buf),
            ThumbnailState::SelectResolution => self.render_resolution(area, buf),
            ThumbnailState::Downloading => self.render_downloading(area, buf),
            ThumbnailState::Done => self.render_done(area, buf),
        }
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        self.input.render(area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "Enter to fetch thumbnail • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_fetching(&self, area: Rect, buf: &mut Buffer) {
        self.spinner.render(area, buf, "Fetching thumbnail information...");
    }

    fn render_resolution(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref title) = self.video_title {
            let display_title = format!("🎬 {}", title);
            let max_len = (area.width - 4) as usize;
            let display_title = if display_title.len() > max_len {
                format!("{}...", &display_title[..max_len])
            } else {
                display_title
            };
            buf.set_string(
                area.x,
                area.y,
                &display_title,
                Style::default().fg(colors::WHITE).bold(),
            );
        }

        buf.set_string(
            area.x,
            area.y + 2,
            "Select thumbnail resolution:",
            Style::default().fg(colors::CYAN).bold(),
        );

        // Resolution options
        for (i, res) in self.resolutions.iter().enumerate() {
            let y = area.y + 4 + i as u16;
            let is_selected = i == self.selected;

            let indicator = if is_selected { " ▸ " } else { "   " };
            let style = if is_selected {
                Style::default().fg(colors::CYAN).bold()
            } else {
                Style::default().fg(colors::WHITE)
            };

            buf.set_string(area.x, y, indicator, style);
            buf.set_string(area.x + 3, y, res.label(), style);
        }

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to download • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_downloading(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "📥 Downloading thumbnails...",
            Style::default().fg(colors::CYAN).bold(),
        );
    }

    fn render_done(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "✅ Thumbnail Download Complete!",
            Style::default().fg(colors::GREEN).bold(),
        );

        // Show downloaded files
        if !self.downloaded_files.is_empty() {
            buf.set_string(
                area.x,
                area.y + 2,
                "Downloaded files:",
                Style::default().fg(colors::CYAN),
            );

            for (i, file) in self.downloaded_files.iter().enumerate() {
                let y = area.y + 3 + i as u16;
                if y >= area.y + area.height - 2 {
                    break;
                }
                let display_file = if file.len() > 60 {
                    format!("...{}", &file[file.len() - 57..])
                } else {
                    file.clone()
                };
                buf.set_string(
                    area.x + 2,
                    y,
                    &format!("• {}", display_file),
                    Style::default().fg(colors::WHITE),
                );
            }
        }

        if let Some((ref msg, is_error)) = self.status_message {
            let color = if is_error { colors::RED } else { colors::WHITE };
            buf.set_string(area.x, area.y + area.height - 3, msg, Style::default().fg(color));
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
