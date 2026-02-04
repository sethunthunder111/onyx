pub mod search;
pub mod video;
pub mod audio;
pub mod playlist;
pub mod thumbnail;
pub mod settings;

use ratatui::prelude::*;
use ratatui::widgets::*;
use crossterm::event::KeyCode;

use super::colors;

/// Common input field widget
pub struct InputField {
    pub label: String,
    pub value: String,
    pub cursor_position: usize,
    pub is_active: bool,
}

impl InputField {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            cursor_position: 0,
            is_active: true,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.value.remove(self.cursor_position);
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.value.len() {
                    self.value.remove(self.cursor_position);
                }
            }
            KeyCode::Left => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.value.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_position = 0;
            }
            KeyCode::End => {
                self.cursor_position = self.value.len();
            }
            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_position = 0;
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Label
        let label_style = Style::default().fg(colors::CYAN).bold();
        buf.set_string(area.x, area.y, &self.label, label_style);

        // Input box
        let input_y = area.y + 1;
        let input_area = Rect {
            x: area.x,
            y: input_y,
            width: area.width,
            height: 3,
        };

        let border_color = if self.is_active {
            colors::CYAN
        } else {
            colors::GRAY
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        block.render(input_area, buf);

        // Value
        let value_x = area.x + 2;
        let value_y = input_y + 1;
        let display_value = if self.value.is_empty() && !self.is_active {
            "Enter value...".to_string()
        } else {
            self.value.clone()
        };

        let value_style = if self.value.is_empty() {
            Style::default().fg(colors::GRAY)
        } else {
            Style::default().fg(colors::WHITE)
        };

        buf.set_string(value_x, value_y, &display_value, value_style);

        // Cursor
        if self.is_active {
            let cursor_x = value_x + self.cursor_position as u16;
            if cursor_x < area.x + area.width - 2 {
                buf.set_string(cursor_x, value_y, "█", Style::default().fg(colors::CYAN));
            }
        }
    }
}

/// List selection widget for formats/quality
pub struct SelectionList<T: Clone> {
    pub items: Vec<(String, T)>,
    pub selected: usize,
    pub title: String,
}

impl<T: Clone> SelectionList<T> {
    pub fn new(title: &str) -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            title: title.to_string(),
        }
    }

    pub fn set_items(&mut self, items: Vec<(String, T)>) {
        self.items = items;
        self.selected = 0;
    }

    pub fn next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn selected_item(&self) -> Option<&T> {
        self.items.get(self.selected).map(|(_, item)| item)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors::BLUE))
            .title(Span::styled(
                format!(" {} ", self.title),
                Style::default().fg(colors::CYAN).bold(),
            ));

        let inner = block.inner(area);
        block.render(area, buf);

        for (i, (label, _)) in self.items.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            let is_selected = i == self.selected;
            let indicator = if is_selected { " ▸ " } else { "   " };
            let style = if is_selected {
                Style::default().fg(colors::CYAN).bold()
            } else {
                Style::default().fg(colors::WHITE)
            };

            buf.set_string(inner.x, y, indicator, style);
            buf.set_string(inner.x + 3, y, label, style);
        }
    }
}

/// Progress bar widget
pub struct ProgressBar {
    pub progress: f32,
    pub label: String,
    pub status: String,
}

impl ProgressBar {
    pub fn new(label: &str) -> Self {
        Self {
            progress: 0.0,
            label: label.to_string(),
            status: String::new(),
        }
    }

    #[allow(dead_code)]
    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 100.0);
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Label
        buf.set_string(
            area.x,
            area.y,
            &self.label,
            Style::default().fg(colors::WHITE).bold(),
        );

        // Progress bar
        let bar_y = area.y + 1;
        let bar_width = area.width.saturating_sub(10);
        let filled = ((self.progress / 100.0) * bar_width as f32) as u16;

        // Bar background
        let bar_bg = "─".repeat(bar_width as usize);
        buf.set_string(area.x, bar_y, &bar_bg, Style::default().fg(colors::GRAY));

        // Bar filled
        let bar_filled = "█".repeat(filled as usize);
        buf.set_string(area.x, bar_y, &bar_filled, Style::default().fg(colors::CYAN));

        // Percentage
        let percent_str = format!(" {:>5.1}%", self.progress);
        buf.set_string(
            area.x + bar_width + 1,
            bar_y,
            &percent_str,
            Style::default().fg(colors::GREEN),
        );

        // Status
        if !self.status.is_empty() {
            buf.set_string(
                area.x,
                bar_y + 1,
                &self.status,
                Style::default().fg(colors::GRAY),
            );
        }
    }
}

/// Spinner widget for loading states
pub struct Spinner {
    pub frames: Vec<&'static str>,
    pub current_frame: usize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames.len();
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, message: &str) {
        let frame = self.frames[self.current_frame];
        let text = format!("{} {}", frame, message);
        buf.set_string(
            area.x,
            area.y,
            &text,
            Style::default().fg(colors::PINK).bold(),
        );
    }
}

/// Message popup
#[allow(dead_code)]
pub fn render_message(area: Rect, buf: &mut Buffer, title: &str, message: &str, is_error: bool) {
    let color = if is_error { colors::RED } else { colors::GREEN };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(Span::styled(
            format!(" {} ", title),
            Style::default().fg(color).bold(),
        ));

    let inner = block.inner(area);
    block.render(area, buf);

    let paragraph = Paragraph::new(message)
        .style(Style::default().fg(colors::WHITE))
        .wrap(Wrap { trim: true });

    paragraph.render(inner, buf);
}
