use ratatui::style::Color;
use crate::app::state::Theme;

pub struct ThemeColors {
    pub bg:             Color,
    pub surface:        Color,
    pub nav_bg:         Color,
    pub panel_border:   Color,
    pub divider:        Color,
    pub text:           Color,
    pub text_muted:     Color,
    pub text_faint:     Color,
    pub nav_key:        Color,
    pub dl_primary:     Color,
    pub dl_dim:         Color,
    pub ul_primary:     Color,
    pub ul_dim:         Color,
    pub accent_green:   Color,
    pub accent_yellow:  Color,
    pub accent_red:     Color,
    pub accent_teal:    Color,
    pub pill_active_bg: Color,
    pub pill_done_fg:   Color,
    pub pill_done_bg:   Color,
}

impl ThemeColors {
    pub fn for_theme(theme: &Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                bg:             Color::Rgb(13,  12,  22),
                surface:        Color::Rgb(30,  28,  48),
                nav_bg:         Color::Rgb(22,  20,  39),

                panel_border:   Color::Rgb(118, 108, 226),
                divider:        Color::Rgb(92,  88,  128),

                text:           Color::Rgb(235, 235, 245),
                text_muted:     Color::Rgb(184, 180, 205),
                text_faint:     Color::Rgb(146, 143, 174),
                nav_key:        Color::Rgb(255, 210, 80),

                dl_primary:     Color::Rgb(244, 152, 68),
                dl_dim:         Color::Rgb(132, 82,  38),
                ul_primary:     Color::Rgb(102, 184, 248),
                ul_dim:         Color::Rgb(58,  107, 156),

                accent_green:   Color::Rgb(80,  220, 130),
                accent_yellow:  Color::Rgb(245, 205, 70),
                accent_red:     Color::Rgb(235, 85,  80),
                accent_teal:    Color::Rgb(65,  215, 195),

                pill_active_bg: Color::Rgb(210, 110, 25),
                pill_done_fg:   Color::Rgb(13,  12,  22),
                pill_done_bg:   Color::Rgb(60,  195, 100),
            },

            Theme::Light => Self {
                bg:             Color::Rgb(248, 247, 255),
                surface:        Color::Rgb(232, 230, 248),
                nav_bg:         Color::Rgb(236, 234, 247),

                panel_border:   Color::Rgb(74,  68,  186),
                divider:        Color::Rgb(132, 126, 174),

                text:           Color::Rgb(18,  15,  38),
                text_muted:     Color::Rgb(70,  66,  102),
                text_faint:     Color::Rgb(92,  88,  128),
                nav_key:        Color::Rgb(155, 80,  0),

                dl_primary:     Color::Rgb(182, 86,  12),
                dl_dim:         Color::Rgb(168, 129, 93),
                ul_primary:     Color::Rgb(22,  86,  180),
                ul_dim:         Color::Rgb(116, 150, 198),

                accent_green:   Color::Rgb(25,  150, 70),
                accent_yellow:  Color::Rgb(175, 120, 0),
                accent_red:     Color::Rgb(185, 35,  35),
                accent_teal:    Color::Rgb(0,   140, 125),

                pill_active_bg: Color::Rgb(195, 95,  15),
                pill_done_fg:   Color::Rgb(248, 247, 255),
                pill_done_bg:   Color::Rgb(25,  150, 70),
            },
        }
    }
}
