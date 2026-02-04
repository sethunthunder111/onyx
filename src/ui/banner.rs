use ratatui::prelude::*;
use ratatui::widgets::*;

use super::colors;

/// ASCII art for ONYX logo with gradient colors
const ONYX_BANNER: &[&str] = &[
    r"  ██████╗ ███╗   ██╗██╗   ██╗██╗  ██╗",
    r" ██╔═══██╗████╗  ██║╚██╗ ██╔╝╚██╗██╔╝",
    r" ██║   ██║██╔██╗ ██║ ╚████╔╝  ╚███╔╝ ",
    r" ██║   ██║██║╚██╗██║  ╚██╔╝   ██╔██╗ ",
    r" ╚██████╔╝██║ ╚████║   ██║   ██╔╝ ██╗",
    r"  ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝",
];

/// Get gradient colors for the banner
fn get_gradient_colors() -> Vec<Color> {
    vec![
        colors::PINK,
        Color::Rgb(255, 150, 150),
        colors::ORANGE,
        colors::YELLOW,
        colors::GREEN,
        colors::CYAN,
    ]
}

/// Render the ONYX banner with rainbow gradient
pub fn render_banner(area: Rect, buf: &mut Buffer) {
    let gradient = get_gradient_colors();
    let lines = ONYX_BANNER.len();

    // Calculate starting position to center the banner
    let banner_width = ONYX_BANNER[0].chars().count() as u16;
    let x_offset = area.x + (area.width.saturating_sub(banner_width)) / 2;

    for (i, line) in ONYX_BANNER.iter().enumerate() {
        let color_idx = (i * gradient.len()) / lines;
        let color = gradient[color_idx.min(gradient.len() - 1)];

        let y = area.y + i as u16;
        if y < area.y + area.height {
            buf.set_string(x_offset, y, *line, Style::default().fg(color).bold());
        }
    }

    // Render subtitle below the banner
    let subtitle = "The Ultimate YouTube Downloader";
    let subtitle_x = area.x + (area.width.saturating_sub(subtitle.len() as u16)) / 2;
    let subtitle_y = area.y + lines as u16 + 1;

    if subtitle_y < area.y + area.height {
        buf.set_string(
            subtitle_x,
            subtitle_y,
            subtitle,
            Style::default().fg(colors::GRAY).italic(),
        );
    }
}

/// Get the height needed for the banner
#[allow(dead_code)]
pub fn banner_height() -> u16 {
    ONYX_BANNER.len() as u16 + 3 // banner + subtitle + padding
}

/// Render a decorative box around the banner area
pub fn render_banner_box(area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors::BLUE));

    block.render(area, buf);
}

/// Widget wrapper for the banner
pub struct BannerWidget;

impl Widget for BannerWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render box first
        render_banner_box(area, buf);

        // Render banner inside the box (with padding)
        let inner = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };
        render_banner(inner, buf);
    }
}
