use ratatui::style::Color;
use crate::app::state::Theme;

pub struct ThemeColors {
    pub bg:            Color,   // very dark navy/charcoal
    pub surface:       Color,   // slightly lighter panel bg
    pub outer_border:  Color,   // blue-purple outer frame
    pub section_border:Color,   // subtle inner section borders
    pub border:        Color,   // general borders
    pub text:          Color,   // primary text (near-white)
    pub text_muted:    Color,   // secondary labels
    pub nav_text:      Color,   // nav bar labels (white)
    pub accent_orange: Color,   // Cloudflare orange (titles, DL)
    pub accent_blue:   Color,   // keep for compat
    pub accent_yellow: Color,   // stars
    pub accent_green:  Color,   // good latency / success
    pub accent_red:    Color,   // errors
    pub shortcut:      Color,   // keyboard hints
    pub header_bg:     Color,
    pub gauge_filled:  Color,
    pub gauge_empty:   Color,
}

impl ThemeColors {
    pub fn for_theme(theme: &Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                // Match the screenshot: very dark navy, orange accents, blue-purple border
                bg:             Color::Rgb(15,  14,  26),   // #0f0e1a deep navy
                surface:        Color::Rgb(24,  22,  40),   // slightly lighter
                outer_border:   Color::Rgb(80,  80,  200),  // blue-purple border
                section_border: Color::Rgb(55,  52,  80),   // muted purple-grey
                border:         Color::Rgb(60,  58,  90),
                text:           Color::Rgb(220, 220, 230),  // near-white
                text_muted:     Color::Rgb(120, 118, 145),
                nav_text:       Color::Rgb(230, 228, 240),
                accent_orange:  Color::Rgb(230, 130, 50),   // #e68232 — screenshot orange
                accent_blue:    Color::Rgb(80,  150, 230),
                accent_yellow:  Color::Rgb(240, 195, 60),
                accent_green:   Color::Rgb(70,  210, 120),
                accent_red:     Color::Rgb(220, 70,  70),
                shortcut:       Color::Rgb(100, 140, 200),
                header_bg:      Color::Rgb(18,  16,  30),
                gauge_filled:   Color::Rgb(230, 130, 50),
                gauge_empty:    Color::Rgb(40,  38,  60),
            },
            Theme::Light => Self {
                bg:             Color::Rgb(250, 248, 255),
                surface:        Color::Rgb(240, 238, 250),
                outer_border:   Color::Rgb(80,  80,  200),
                section_border: Color::Rgb(180, 178, 210),
                border:         Color::Rgb(190, 188, 215),
                text:           Color::Rgb(20,  18,  40),
                text_muted:     Color::Rgb(110, 108, 140),
                nav_text:       Color::Rgb(30,  28,  55),
                accent_orange:  Color::Rgb(200, 100, 20),
                accent_blue:    Color::Rgb(30,  100, 200),
                accent_yellow:  Color::Rgb(190, 130, 0),
                accent_green:   Color::Rgb(30,  160, 80),
                accent_red:     Color::Rgb(190, 40,  40),
                shortcut:       Color::Rgb(60,  100, 180),
                header_bg:      Color::Rgb(238, 236, 248),
                gauge_filled:   Color::Rgb(200, 100, 20),
                gauge_empty:    Color::Rgb(215, 212, 235),
            },
        }
    }

    pub fn download_color(&self) -> Color { self.accent_orange }
    pub fn upload_color(&self)   -> Color { Color::Rgb(200, 200, 210) }

    pub fn latency_color(&self, ms: f64) -> Color {
        if ms < 50.0        { Color::Rgb(80, 200, 120) }
        else if ms < 100.0  { self.accent_yellow }
        else if ms < 200.0  { Color::Rgb(255, 155, 40) }
        else                { self.accent_red }
    }
}
