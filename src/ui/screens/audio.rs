use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::KeyCode;

use crate::ui::colors;
use crate::ytdlp::formats::{AudioFormat, VideoInfo};
use super::{InputField, SelectionList, ProgressBar, Spinner};

/// Audio download screen state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioState {
    UrlInput,
    FetchingInfo,
    QualitySelect,
    FormatSelect,
    Downloading,
    Done,
}

/// Audio download screen
pub struct AudioScreen {
    pub state: AudioState,
    pub input: InputField,
    pub video_info: Option<VideoInfo>,
    pub quality_options: SelectionList<String>,
    pub format_options: SelectionList<AudioFormat>,
    pub selected_quality: Option<String>,
    pub progress: ProgressBar,
    pub spinner: Spinner,
    pub status_message: Option<(String, bool)>,
}

impl Default for AudioScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioScreen {
    pub fn new() -> Self {
        let mut format_options = SelectionList::new("Select Output Format");
        format_options.set_items(
            AudioFormat::all()
                .into_iter()
                .map(|f| (format!("🎵 {}", f.extension().to_uppercase()), f))
                .collect(),
        );

        Self {
            state: AudioState::UrlInput,
            input: InputField::new("🎵 Enter Video/Audio URL:"),
            video_info: None,
            quality_options: SelectionList::new("Select Audio Quality"),
            format_options,
            selected_quality: None,
            progress: ProgressBar::new("Downloading..."),
            spinner: Spinner::new(),
            status_message: None,
        }
    }

    pub fn reset(&mut self) {
        self.state = AudioState::UrlInput;
        self.input.clear();
        self.video_info = None;
        self.quality_options.items.clear();
        self.selected_quality = None;
        self.progress.progress = 0.0;
        self.status_message = None;
    }

    pub fn set_video_info(&mut self, info: VideoInfo) {
        // Build quality options from audio formats
        let mut options = Vec::new();

        let audio_formats = info.get_audio_formats();
        for fmt in audio_formats.iter() {
            if let Some(abr) = fmt.get_audio_bitrate() {
                let label = format!("🔊 {}kbps", abr);
                options.push((label, format!("{}", abr)));
            }
        }

        // Add "Best" option at top
        options.insert(0, ("⭐ Best Available".to_string(), "best".to_string()));

        self.quality_options.set_items(options);
        self.video_info = Some(info);
        self.state = AudioState::QualitySelect;
    }

    pub fn selected_format(&self) -> Option<AudioFormat> {
        self.format_options.selected_item().copied()
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            AudioState::UrlInput => {
                self.input.handle_key(key);
            }
            AudioState::QualitySelect => match key {
                KeyCode::Up => self.quality_options.previous(),
                KeyCode::Down => self.quality_options.next(),
                _ => {}
            },
            AudioState::FormatSelect => match key {
                KeyCode::Up => self.format_options.previous(),
                KeyCode::Down => self.format_options.next(),
                _ => {}
            },
            _ => {}
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.state {
            AudioState::UrlInput => self.render_input(area, buf),
            AudioState::FetchingInfo => self.render_fetching(area, buf),
            AudioState::QualitySelect => self.render_quality(area, buf),
            AudioState::FormatSelect => self.render_format(area, buf),
            AudioState::Downloading => self.render_downloading(area, buf),
            AudioState::Done => self.render_done(area, buf),
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
            "Enter to fetch audio info • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_fetching(&self, area: Rect, buf: &mut Buffer) {
        self.spinner.render(area, buf, "Fetching audio information...");
    }

    fn render_quality(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ref info) = self.video_info {
            let title = format!("🎵 {}", info.title);
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

        let list_area = Rect {
            x: area.x,
            y: area.y + 3,
            width: area.width.min(40),
            height: area.height.saturating_sub(5),
        };
        self.quality_options.render(list_area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to select format • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_format(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y,
            "📁 Select output format:",
            Style::default().fg(colors::CYAN).bold(),
        );

        let list_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width.min(30),
            height: area.height.saturating_sub(4),
        };
        self.format_options.render(list_area, buf);

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
            "✅ Audio Download Complete!",
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
