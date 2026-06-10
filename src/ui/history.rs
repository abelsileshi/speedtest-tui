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
    buf:    &mut Buffer,
    area:   Rect,
    history: &[TestResult],
    scroll:  usize,
    c:       &ThemeColors,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled("── ", Style::default().fg(c.dl_primary)),
            Span::styled(
                "TEST HISTORY",
                Style::default().fg(c.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  [↑↓] scroll  [q] back ──",
                Style::default().fg(c.text_muted),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(c.panel_border));
    let inner = block.inner(area);
    block.render(area, buf);

    if history.is_empty() {
        let msg = "No test history yet.";
        let x = inner.x + inner.width.saturating_sub(msg.len() as u16) / 2;
        let y = inner.y + inner.height / 2;
        for (i, ch) in msg.chars().enumerate() {
            buf[(x + i as u16, y)].set_char(ch).set_fg(c.text_faint);
        }
        return;
    }

    let header = ListItem::new(Line::from(vec![
        Span::styled(
            format!("{:<17} ", "TIMESTAMP"),
            Style::default().fg(c.text_faint).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<12}", "▼ DL Mbps"),
            Style::default().fg(c.dl_primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<12}", "▲ UL Mbps"),
            Style::default().fg(c.ul_primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{:<10}", "PING ms"),
            Style::default().fg(c.accent_green).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "SERVER",
            Style::default().fg(c.text_muted).add_modifier(Modifier::BOLD),
        ),
    ]));

    let rows: Vec<ListItem> = history
        .iter()
        .skip(scroll)
        .take(inner.height.saturating_sub(3) as usize)
        .map(|r| {
            let ts = r
                .timestamp
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "—".into());

            let dl_col = if r.download_mbps > 50.0 {
                c.accent_green
            } else if r.download_mbps > 10.0 {
                c.dl_primary
            } else {
                c.text_muted
            };

            let ul_col = if r.upload_mbps > 20.0 {
                c.accent_green
            } else if r.upload_mbps > 5.0 {
                c.ul_primary
            } else {
                c.text_muted
            };

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<17} ", ts),
                    Style::default().fg(c.text_muted),
                ),
                Span::styled(
                    format!("{:<12.2}", r.download_mbps),
                    Style::default().fg(dl_col).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<12.2}", r.upload_mbps),
                    Style::default().fg(ul_col).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:<10.0}", r.ping_ms),
                    Style::default().fg(c.text_muted),
                ),
                Span::styled(
                    r.server_name.as_str(),
                    Style::default().fg(c.text_faint),
                ),
            ]))
        })
        .collect();

    let mut items = vec![header];
    items.extend(rows);

    List::new(items).render(inner, buf);
}
