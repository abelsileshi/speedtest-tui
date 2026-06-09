use ratatui::{buffer::Buffer, layout::Rect, style::Color};

pub fn quality_stars(score: f64) -> String {
    let filled = score.round() as usize;
    let empty  = 5usize.saturating_sub(filled);
    format!("{}{}", "★".repeat(filled), "☆".repeat(empty))
}

pub fn render_speed_bar(
    buf: &mut Buffer,
    area: Rect,
    value: f64,
    max: f64,
    filled_color: Color,
    empty_color: Color,
) {
    let ratio  = (value / max.max(1.0)).clamp(0.0, 1.0);
    let filled = (ratio * area.width as f64) as u16;
    for x in area.x..area.x + area.width {
        let offset = x - area.x;
        let (ch, fg) = if offset < filled {
            ('█', filled_color)
        } else {
            ('░', empty_color)
        };
        buf[(x, area.y)].set_char(ch).set_fg(fg);
    }
}

/// ▁▂▃▄▅▆▇█ sparkline
pub struct Sparkline<'a> {
    pub data:  &'a [f64],
    pub color: Color,
    pub max:   f64,
}

impl<'a> ratatui::widgets::Widget for Sparkline<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.data.is_empty() || area.width == 0 { return; }
        let max = if self.max > 0.0 { self.max } else { 1.0 };
        let bars: Vec<char> = "▁▂▃▄▅▆▇█".chars().collect();
        let n = bars.len() as f64;
        for (i, &val) in self.data.iter().rev().take(area.width as usize).enumerate() {
            let x   = area.x + area.width - 1 - i as u16;
            let idx = ((val / max).clamp(0.0, 1.0) * (n - 1.0)) as usize;
            buf[(x, area.y)].set_char(bars[idx]).set_fg(self.color);
        }
    }
}
