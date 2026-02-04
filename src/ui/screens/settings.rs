use ratatui::prelude::*;
use crossterm::event::KeyCode;

use crate::config::Config;
use crate::ui::colors;
use super::InputField;

/// Settings screen options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsOption {
    DownloadPath,
    ParallelDownloads,
    DebugMode,
    Back,
}

impl SettingsOption {
    pub fn all() -> Vec<SettingsOption> {
        vec![
            SettingsOption::DownloadPath,
            SettingsOption::ParallelDownloads,
            SettingsOption::DebugMode,
            SettingsOption::Back,
        ]
    }

    pub fn label(&self) -> &str {
        match self {
            SettingsOption::DownloadPath => "📁 Download Path",
            SettingsOption::ParallelDownloads => "⚡ Parallel Downloads",
            SettingsOption::DebugMode => "🔧 Debug Mode",
            SettingsOption::Back => "← Back to Menu",
        }
    }
}

/// Settings screen state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsState {
    Menu,
    EditingPath,
    EditingParallel,
}

/// Settings screen
pub struct SettingsScreen {
    pub state: SettingsState,
    pub options: Vec<SettingsOption>,
    pub selected: usize,
    pub config: Config,
    pub path_input: InputField,
    pub parallel_value: u8,
    pub status_message: Option<(String, bool)>,
}

impl SettingsScreen {
    pub fn new(config: Config) -> Self {
        let mut path_input = InputField::new("Enter download path:");
        path_input.value = config.download_path.clone();
        path_input.cursor_position = path_input.value.len();

        Self {
            state: SettingsState::Menu,
            options: SettingsOption::all(),
            selected: 0,
            config,
            path_input,
            parallel_value: 3,
            status_message: None,
        }
    }

    pub fn reset(&mut self) {
        self.state = SettingsState::Menu;
        self.selected = 0;
        self.status_message = None;
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
        self.path_input.value = self.config.download_path.clone();
        self.path_input.cursor_position = self.path_input.value.len();
        self.parallel_value = self.config.parallel_downloads;
    }

    pub fn selected_option(&self) -> SettingsOption {
        self.options[self.selected]
    }

    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % self.options.len();
    }

    pub fn previous(&mut self) {
        self.selected = if self.selected == 0 {
            self.options.len() - 1
        } else {
            self.selected - 1
        };
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.state {
            SettingsState::Menu => match key {
                KeyCode::Up => self.previous(),
                KeyCode::Down => self.next(),
                _ => {}
            },
            SettingsState::EditingPath => {
                if key == KeyCode::Tab {
                    self.autocomplete_path();
                } else {
                    self.path_input.handle_key(key);
                }
            }
            SettingsState::EditingParallel => match key {
                KeyCode::Up | KeyCode::Right => {
                    if self.parallel_value < 10 {
                        self.parallel_value += 1;
                    }
                }
                KeyCode::Down | KeyCode::Left => {
                    if self.parallel_value > 1 {
                        self.parallel_value -= 1;
                    }
                }
                _ => {}
            },
        }
    }

    fn autocomplete_path(&mut self) {
        let current_input = &self.path_input.value;
        let path = std::path::Path::new(current_input);
        
        // Determine directory to search and prefix to match
        let (dir, prefix) = if current_input.ends_with(std::path::MAIN_SEPARATOR) {
            (path, "")
        } else {
            match path.parent() {
                Some(parent) => (parent, path.file_name().and_then(|s| s.to_str()).unwrap_or("")),
                None => (std::path::Path::new("."), current_input.as_str()),
            }
        };

        // If directory is empty (relative path start), assume "."
        let dir = if dir.as_os_str().is_empty() {
            std::path::Path::new(".")
        } else {
            dir
        };

        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut matches: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    // Include all dirs for cycling if we already have a match, 
                    // otherwise filter by prefix
                    Some(name)
                })
                .collect();
            
            matches.sort();

            if matches.is_empty() {
                return;
            }

            // Find current match index to cycle
            let current_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            
            // Filter matches if we are starting fresh (not cycling existing)
            let valid_matches: Vec<&String> = if matches.contains(&current_name.to_string()) {
                matches.iter().collect()
            } else {
                matches.iter().filter(|m| m.starts_with(prefix)).collect()
            };

            if valid_matches.is_empty() {
                return;
            }

            let next_match = if let Some(idx) = valid_matches.iter().position(|m| *m == current_name) {
                valid_matches[(idx + 1) % valid_matches.len()]
            } else {
                valid_matches[0]
            };

            // Construct new path
            let new_path = if dir == std::path::Path::new(".") {
                if current_input.starts_with("./") {
                     format!("./{}", next_match)
                } else {
                     next_match.clone()
                }
            } else {
                dir.join(next_match).to_string_lossy().to_string()
            };

            self.path_input.value = new_path;
            self.path_input.cursor_position = self.path_input.value.len();
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Title
        buf.set_string(
            area.x,
            area.y,
            "⚙️  Settings",
            Style::default().fg(colors::CYAN).bold(),
        );

        match self.state {
            SettingsState::Menu => self.render_menu(area, buf),
            SettingsState::EditingPath => self.render_edit_path(area, buf),
            SettingsState::EditingParallel => self.render_edit_parallel(area, buf),
        }
    }

    fn render_menu(&self, area: Rect, buf: &mut Buffer) {
        let start_y = area.y + 2;

        for (i, option) in self.options.iter().enumerate() {
            let y = start_y + (i as u16 * 2);
            if y >= area.y + area.height - 2 {
                break;
            }

            let is_selected = i == self.selected;
            let indicator = if is_selected { " ▸ " } else { "   " };

            let style = if is_selected {
                Style::default().fg(colors::CYAN).bold()
            } else {
                Style::default().fg(colors::WHITE)
            };

            buf.set_string(area.x, y, indicator, style);
            buf.set_string(area.x + 3, y, option.label(), style);

            // Show current value
            let value = match option {
                SettingsOption::DownloadPath => {
                    let path = &self.config.download_path;
                    if path.len() > 40 {
                        format!("...{}", &path[path.len() - 37..])
                    } else {
                        path.clone()
                    }
                }
                SettingsOption::ParallelDownloads => {
                    format!("{}", self.config.parallel_downloads)
                }
                SettingsOption::DebugMode => {
                    if self.config.debug_mode {
                        "ON".to_string()
                    } else {
                        "OFF".to_string()
                    }
                }
                SettingsOption::Back => String::new(),
            };

            if !value.is_empty() {
                let value_x = area.x + 30;
                buf.set_string(value_x, y, &value, Style::default().fg(colors::GRAY));
            }
        }

        // Status message
        if let Some((ref msg, is_error)) = self.status_message {
            let color = if is_error { colors::RED } else { colors::GREEN };
            buf.set_string(area.x, area.y + area.height - 3, msg, Style::default().fg(color));
        }

        // Instructions
        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "↑↓ navigate • Enter to edit • Esc to go back",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_edit_path(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y + 3,
            "Edit Download Path:",
            Style::default().fg(colors::WHITE).bold(),
        );

        let input_area = Rect {
            x: area.x,
            y: area.y + 5,
            width: area.width,
            height: 4,
        };
        self.path_input.render(input_area, buf);

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "Enter to save • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }

    fn render_edit_parallel(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(
            area.x,
            area.y + 3,
            "Parallel Downloads (1-10):",
            Style::default().fg(colors::WHITE).bold(),
        );

        // Visual slider
        let slider_y = area.y + 5;
        let slider_width = 20u16;

        // Background
        let bg = "─".repeat(slider_width as usize);
        buf.set_string(area.x, slider_y, &bg, Style::default().fg(colors::GRAY));

        // Fill
        let fill_width = ((self.parallel_value as f32 / 10.0) * slider_width as f32) as u16;
        let fill = "█".repeat(fill_width as usize);
        buf.set_string(area.x, slider_y, &fill, Style::default().fg(colors::CYAN));

        // Value
        buf.set_string(
            area.x + slider_width + 2,
            slider_y,
            &format!("{}", self.parallel_value),
            Style::default().fg(colors::WHITE).bold(),
        );

        let instr_y = area.y + area.height - 1;
        buf.set_string(
            area.x,
            instr_y,
            "←→ or ↑↓ to adjust • Enter to save • Esc to cancel",
            Style::default().fg(colors::GRAY),
        );
    }
}
