use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};
use crate::app::state::TestResult;
use crate::ui::theme::ThemeColors;

pub fn render_history(
    buf:     &mut Buffer,
    area:    Rect,
    history: &[TestResult],
    scroll:  usize,
    colors:  &ThemeColors,
) {
    let visible = area.height.saturating_sub(4) as usize;

    let items: Vec<ListItem> = history
        .iter()
        .rev()
        .skip(scroll)
        .take(visible)
        .map(|r| {
            let ts = r.timestamp
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".into());

            ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(format!("{:<17}", ts),                   Style::default().fg(colors.text_muted)),
                Span::raw("  "),
                Span::styled("▼ ", Style::default().fg(colors.accent_orange)),
                Span::styled(format!("{:>7.1} Mbps", r.download_mbps), Style::default().fg(colors.text)),
                Span::raw("   "),
                Span::styled("▲ ", Style::default().fg(colors.upload_color())),
                Span::styled(format!("{:>7.1} Mbps", r.upload_mbps),   Style::default().fg(colors.text)),
                Span::raw("   "),
                Span::styled("ping ", Style::default().fg(colors.text_muted)),
                Span::styled(format!("{:>5.0} ms", r.ping_ms),         Style::default().fg(colors.text)),
                Span::raw("   "),
                Span::styled(format!("[{}]", r.quality_grade),
                    Style::default().fg(colors.accent_yellow)),
            ]))
        })
        .collect();

    List::new(items)
        .block(Block::default()
            .title(Span::styled(
                " TEST HISTORY  [↑↓] scroll  [q] back ",
                Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.outer_border)))
        .render(area, buf);
}
