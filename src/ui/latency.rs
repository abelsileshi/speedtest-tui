// Latency is rendered inline in dashboard.rs — kept for module compatibility
use ratatui::{buffer::Buffer, layout::Rect};
use crate::app::state::LatencyStats;
use crate::ui::theme::ThemeColors;

pub fn render_latency_panel(
    _buf:    &mut Buffer,
    _area:   Rect,
    _stats:  &LatencyStats,
    _colors: &ThemeColors,
) {}
