use ratatui::prelude::*;
use crossterm::event::KeyCode;

use crate::ui::colors;
use crate::ytdlp::search::SearchResult;
use super::{InputField, SelectionList, Spinner};

/// Search screen state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchState {
    Input,
    Results,
    QualitySelect,
}

/// Search screen
pub struct SearchScreen {
    pub state: SearchState,
    pub input: InputField,
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
    pub quality_options: SelectionList<String>,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub is_loading: bool,
    pub spinner: Spinner,
}

impl Default for SearchScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchScreen {
    pub fn new() -> Self {
        Self {
            state: SearchState::Input,
            input: InputField::new("🔍 Search YouTube:"),
            results: Vec::new(),
            selected_result: 0,
            quality_options: SelectionList::new("Select Quality"),
            status_message: None,
            is_loading: false,
            spinner: Spinner::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = SearchState::Input;
        self.input.clear();
        self.results.clear();
        self.selected_result = 0;
        self.quality_options.items.clear();
        self.status_message = None;
        self.is_loading = false;
    }

    pub fn set_results(&mut self, results: Vec<SearchResult>) {
        self.results = results;
        self.selected_result = 0;
        self.state = SearchState::Results;
        self.is_loading = false;
    }

    pub fn set_quality_options(&mut self, options: Vec<(String, String)>) {
        self.quality_options.set_items(options);
        self.state = SearchState::QualitySelect;
    }

    pub fn next_result(&mut self) {
        if !self.results.is_empty() {
            self.selected_result = (self.selected_result + 1) % self.results.len();
        }
    }

    pub fn previous_result(&mut self) {
        if !self.results.is_empty() {
            self.selected_result = if self.selected_result == 0 {
                self.results.len() - 1
            } else {
                self.selected_result - 1
            };
        }
    }

    pub fn selected_video(&self) -> Option<&SearchResult> {
        self.results.get(self.selected_result)
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            SearchState::Input => {
                self.input.handle_key(key);
            }
            SearchState::Results => match key {
                KeyCode::Up => self.previous_result(),
                KeyCode::Down => self.next_result(),
                _ => {}
            },
            SearchState::QualitySelect => match key {
                KeyCode::Up => self.quality_options.previous(),
                KeyCode::Down => self.quality_options.next(),
                _ => {}
            },
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.state {
            SearchState::Input => self.render_input(area, buf),
            SearchState::Results => self.render_results(area, buf),
            SearchState::QualitySelect => self.render_quality(area, buf),
        }
    }

    fn render_input(&self, area: Rect, buf: &mut Buffer) {
        self.input.render(area, buf);

        if self.is_loading {
            let loading_y = area.y + 5;
            self.spinner.render(
                Rect {
                    x: area.x,
                    y: loading_y,
                    width: area.width,
                    height: 1,
                },
                buf,
                "Searching...",
            );
        }

        // Instructions
        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "Enter to search • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_results(&self, area: Rect, buf: &mut Buffer) {
        let title = format!("📋 Search Results ({} found)", self.results.len());
        buf.set_string(
            area.x,
            area.y,
            &title,
            Style::default().fg(colors::CYAN).bold(),
        );

        let list_area = Rect {
            x: area.x,
            y: area.y + 2,
            width: area.width,
            height: area.height.saturating_sub(4),
        };

        for (i, result) in self.results.iter().enumerate() {
            let y = list_area.y + (i as u16 * 2);
            if y >= list_area.y + list_area.height {
                break;
            }

            let is_selected = i == self.selected_result;
            let indicator = if is_selected { " ▸ " } else { "   " };

            // Selection indicator
            let ind_style = if is_selected {
                Style::default().fg(colors::CYAN).bold()
            } else {
                Style::default().fg(colors::GRAY)
            };
            buf.set_string(list_area.x, y, indicator, ind_style);

            // Title (truncated)
            let max_title_len = (list_area.width - 20) as usize;
            let title = if result.title.len() > max_title_len {
                format!("{}...", &result.title[..max_title_len])
            } else {
                result.title.clone()
            };

            let title_style = if is_selected {
                Style::default().fg(colors::WHITE).bold()
            } else {
                Style::default().fg(colors::WHITE)
            };
            buf.set_string(list_area.x + 3, y, &title, title_style);

            // Duration and channel
            let meta = format!(
                "   {} • {}",
                result.get_duration(),
                result.get_channel()
            );
            buf.set_string(
                list_area.x + 3,
                y + 1,
                &meta,
                Style::default().fg(colors::GRAY),
            );
        }

        // Instructions
        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to select • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_quality(&self, area: Rect, buf: &mut Buffer) {
        if let Some(video) = self.selected_video() {
            // Video info
            let title = format!("🎬 {}", video.title);
            buf.set_string(
                area.x,
                area.y,
                &title,
                Style::default().fg(colors::WHITE).bold(),
            );

            let meta = format!("{} • {}", video.get_duration(), video.get_channel());
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
            width: area.width,
            height: area.height.saturating_sub(5),
        };
        self.quality_options.render(list_area, buf);

        // Instructions
        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to download • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }
}
