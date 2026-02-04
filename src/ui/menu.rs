use ratatui::prelude::*;

use super::colors;

/// Menu item with icon and label
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub icon: &'static str,
    pub label: &'static str,
    pub id: MenuAction,
    pub is_separator: bool,
}

/// Menu actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    SearchDownload,
    DownloadVideo,
    DownloadAudio,
    DownloadPlaylist,
    DownloadThumbnail,
    WebInterface,
    Separator,
    Settings,
    Exit,
}

impl MenuAction {
    pub fn all() -> Vec<MenuItem> {
        vec![
            MenuItem {
                icon: "🔍",
                label: "Search & Download",
                id: MenuAction::SearchDownload,
                is_separator: false,
            },
            MenuItem {
                icon: "🎬",
                label: "Download Video (URL)",
                id: MenuAction::DownloadVideo,
                is_separator: false,
            },
            MenuItem {
                icon: "🎵",
                label: "Download Audio (URL)",
                id: MenuAction::DownloadAudio,
                is_separator: false,
            },
            MenuItem {
                icon: "📦",
                label: "Download Playlist (URL)",
                id: MenuAction::DownloadPlaylist,
                is_separator: false,
            },
            MenuItem {
                icon: "🖼️",
                label: "Download Thumbnail (URL)",
                id: MenuAction::DownloadThumbnail,
                is_separator: false,
            },
            MenuItem {
                icon: "🌐",
                label: "Launch Web Interface",
                id: MenuAction::WebInterface,
                is_separator: false,
            },
            MenuItem {
                icon: " ",
                label: "────────────────────────",
                id: MenuAction::Separator,
                is_separator: true,
            },
            MenuItem {
                icon: "⚙️",
                label: "Settings",
                id: MenuAction::Settings,
                is_separator: false,
            },
            MenuItem {
                icon: "🚪",
                label: "Exit",
                id: MenuAction::Exit,
                is_separator: false,
            },
        ]
    }
}

/// Main menu state
pub struct MainMenu {
    items: Vec<MenuItem>,
    selected: usize,
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            items: MenuAction::all(),
            selected: 0,
        }
    }

    pub fn next(&mut self) {
        let len = self.items.len();
        let mut new_idx = (self.selected + 1) % len;
        // Skip separator
        while self.items[new_idx].is_separator {
            new_idx = (new_idx + 1) % len;
        }
        self.selected = new_idx;
    }

    pub fn previous(&mut self) {
        let len = self.items.len();
        let mut new_idx = if self.selected == 0 {
            len - 1
        } else {
            self.selected - 1
        };
        // Skip separator
        while self.items[new_idx].is_separator {
            new_idx = if new_idx == 0 { len - 1 } else { new_idx - 1 };
        }
        self.selected = new_idx;
    }

    pub fn selected_action(&self) -> MenuAction {
        self.items[self.selected].id
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Title
        let title = Span::styled(
            " ? What would you like to do?",
            Style::default().fg(colors::CYAN).bold(),
        );
        buf.set_span(area.x, area.y, &title, area.width);

        // Menu items
        let items_start_y = area.y + 1;

        for (i, item) in self.items.iter().enumerate() {
            let y = items_start_y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            if item.is_separator {
                // Render separator line
                buf.set_string(
                    area.x + 3,
                    y,
                    item.label,
                    Style::default().fg(colors::GRAY).dim(),
                );
                continue;
            }

            let is_selected = i == self.selected;

            // Selection indicator
            let indicator = if is_selected { " >" } else { "  " };
            let indicator_style = if is_selected {
                Style::default().fg(colors::CYAN).bold()
            } else {
                Style::default().fg(colors::GRAY)
            };
            buf.set_string(area.x, y, indicator, indicator_style);

            // Icon
            buf.set_string(area.x + 3, y, item.icon, Style::default());

            // Label
            let label_style = if is_selected {
                Style::default().fg(colors::WHITE).bold()
            } else {
                Style::default().fg(colors::GRAY)
            };
            buf.set_string(area.x + 6, y, item.label, label_style);
        }
    }
}

impl Widget for &MainMenu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render(area, buf);
    }
}
