pub mod banner;
pub mod menu;
pub mod screens;



/// Common colors used throughout the app
pub mod colors {
    use ratatui::style::Color;

    pub const PINK: Color = Color::Rgb(255, 121, 198);
    pub const ORANGE: Color = Color::Rgb(255, 184, 108);
    pub const YELLOW: Color = Color::Rgb(241, 250, 140);
    pub const GREEN: Color = Color::Rgb(80, 250, 123);
    pub const CYAN: Color = Color::Rgb(139, 233, 253);
    pub const BLUE: Color = Color::Rgb(98, 114, 164);
    #[allow(dead_code)]
    pub const PURPLE: Color = Color::Rgb(189, 147, 249);
    pub const RED: Color = Color::Rgb(255, 85, 85);
    pub const WHITE: Color = Color::Rgb(248, 248, 242);
    pub const GRAY: Color = Color::Rgb(98, 114, 164);
    #[allow(dead_code)]
    pub const DARK: Color = Color::Rgb(40, 42, 54);
    pub const DARKER: Color = Color::Rgb(30, 31, 41);
}
