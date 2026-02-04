use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::KeyCode;

use crate::ui::colors;
use crate::ytdlp::formats::VideoInfo;
use super::{InputField, SelectionList, ProgressBar, Spinner};

/// Video download screen state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoState {
    UrlInput,
    FetchingInfo,
    QualitySelect,
    Downloading,
    Done,
}

/// Video download screen
pub struct VideoScreen {
    pub state: VideoState,
    pub input: InputField,
    pub video_info: Option<VideoInfo>,
    pub quality_options: SelectionList<String>,
    pub progress: ProgressBar,
    pub spinner: Spinner,
    pub status_message: Option<(String, bool)>,
}

impl Default for VideoScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoScreen {
    pub fn new() -> Self {
        Self {
            state: VideoState::UrlInput,
            input: InputField::new("🎬 Enter Video URL:"),
            video_info: None,
            quality_options: SelectionList::new("Select Quality"),
            progress: ProgressBar::new("Downloading..."),
            spinner: Spinner::new(),
            status_message: None,
        }
    }

    pub fn reset(&mut self) {
        self.state = VideoState::UrlInput;
        self.input.clear();
        self.video_info = None;
        self.quality_options.items.clear();
        self.progress.progress = 0.0;
        self.status_message = None;
    }

    pub fn set_video_info(&mut self, info: VideoInfo) {
        // Build quality options
        let mut options = Vec::new();

        // Group by resolution
        let video_formats = info.get_video_formats();
        for (label, formats) in &video_formats {
            // Find best format in this resolution group
            if let Some(fmt) = formats.first() {
                let size = fmt.get_size_str();
                let option_label = format!("🎥 {} ({})", label, size);
                let format_string = format!("bestvideo[height<={}]+bestaudio/best", fmt.height.unwrap_or(1080));
                options.push((option_label, format_string));
            }
        }

        // Add audio options
        let audio_formats = info.get_audio_formats();
        if !audio_formats.is_empty() {
            options.push(("───────────────".to_string(), "separator".to_string()));
            for fmt in audio_formats.iter().take(3) {
                if let Some(abr) = fmt.get_audio_bitrate() {
                    let label = format!("🎵 Audio {}kbps", abr);
                    let format_string = format!("bestaudio[abr<={}]/bestaudio", abr);
                    options.push((label, format_string));
                }
            }
        }

        // Add MP3 option
        options.push(("🎵 MP3 (Best Quality)".to_string(), "mp3".to_string()));

        self.quality_options.set_items(options);
        self.video_info = Some(info);
        self.state = VideoState::QualitySelect;
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            VideoState::UrlInput => {
                self.input.handle_key(key);
            }
            VideoState::QualitySelect => match key {
                KeyCode::Up => self.quality_options.previous(),
                KeyCode::Down => self.quality_options.next(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.state {
            VideoState::UrlInput => self.render_input(area, buf),
            VideoState::FetchingInfo => self.render_fetching(area, buf),
            VideoState::QualitySelect => self.render_quality(area, buf),
            VideoState::Downloading => self.render_downloading(area, buf),
            VideoState::Done => self.render_done(area, buf),
        }
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        self.input.render(area, buf);

        // Show status message (error) if any
        if let Some((ref msg, is_error)) = self.status_message {
            let color = if is_error { colors::RED } else { colors::GREEN };
            buf.set_string(
                area.x,
                area.y + 5,
                msg,
                Style::default().fg(color),
            );
        }

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "Enter to fetch video info • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_fetching(&self, area: Rect, buf: &mut Buffer) {
        self.spinner.render(area, buf, "Fetching video information...");
    }

    fn render_quality(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref info) = self.video_info {
            // Video info display
            let title = format!("🎬 {}", info.title);
            let max_len = (area.width - 4) as usize;
            let title = if title.len() > max_len {
                format!("{}...", &title[..max_len])
            } else {
                title
            };
            buf.set_string(
                area.x,
                area.y,
                &title,
                Style::default().fg(colors::WHITE).bold(),
            );

            let meta = format!("⏱️ {} • 👤 {}", info.get_duration(), info.get_channel());
            buf.set_string(
                area.x,
                area.y + 1,
                &meta,
                Style::default().fg(colors::GRAY),
            );
        }

        // Quality selection
        let list_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width.min(50),
            height: area.height.saturating_sub(5),
        };
        self.quality_options.render(list_area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to download • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_downloading(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref info) = self.video_info {
            buf.set_string(
                area.x,
                area.y,
                &format!("📥 Downloading: {}", info.title),
                Style::default().fg(colors::CYAN).bold(),
            );
        }

        let progress_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: 4,
        };
        self.progress.render(progress_area, buf);
    }

    fn render_done(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "✅ Download Complete!",
            Style::default().fg(colors::GREEN).bold(),
        );

        if let Some((ref msg, is_error)) = self.status_message {
            let color = if is_error { colors::RED } else { colors::WHITE };
            let status_area = Rect {
                x: area.x,
                y: area.y + 2,
                width: area.width,
                height: area.height.saturating_sub(4),
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
