use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Widget},
};
use crate::app::state::WorkerState;
use crate::ui::theme::ThemeColors;

pub fn render_worker_streams(
    buf:         &mut Buffer,
    area:        Rect,
    workers:     &[WorkerState],
    colors:      &ThemeColors,
    label:       &str,
    gauge_color: Color,
) {
    let block = Block::default()
        .title(format!(" {} Workers ", label))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.border));
    let inner = block.inner(area);
    block.render(area, buf);

    if workers.is_empty() { return; }

    let row_h = ((inner.height as usize) / workers.len()).max(1) as u16;
    let max_speed = workers.iter().map(|w| w.speed_mbps).fold(1.0f64, f64::max);

    for (i, worker) in workers.iter().enumerate() {
        let y = inner.y + i as u16 * row_h;
        if y >= inner.y + inner.height { break; }

        let ratio = (worker.speed_mbps / max_speed.max(1.0)).clamp(0.0, 1.0);
        let icon  = if worker.complete { "✓" } else if worker.active { "▶" } else { "○" };
        let lbl   = format!("{} W{} {:.1} Mbps", icon, worker.id + 1, worker.speed_mbps);

        Gauge::default()
            .gauge_style(Style::default().fg(gauge_color).bg(colors.gauge_empty))
            .ratio(ratio)
            .label(Span::styled(lbl, Style::default().fg(colors.text)))
            .render(Rect { x: inner.x, y, width: inner.width, height: 1 }, buf);
    }
}
