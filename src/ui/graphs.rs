use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Widget},
};
use crate::ui::theme::ThemeColors;

pub fn render_single_chart(
    buf:     &mut Buffer,
    area:    Rect,
    title:   &str,
    history: &[f64],
    color:   Color,
    colors:  &ThemeColors,
) {
    let max_val = history.iter().cloned().fold(0.0f64, f64::max);
    let y_max   = (max_val * 1.3).max(10.0);

    let data: Vec<(f64, f64)> = history.iter()
        .enumerate()
        .map(|(i, &v)| (i as f64, v))
        .collect();

    let x_max = history.len().max(2) as f64;
    let p90   = percentile_90(history);
    let legend = if p90 > 0.0 { format!("90th ≈ {:.1} Mbps", p90) } else { "waiting…".into() };

    let datasets = vec![
        Dataset::default()
            .name(legend.as_str())
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(&data),
    ];

    let y_labels = vec![
        Span::styled("  0", Style::default().fg(colors.text_muted)),
        Span::styled(format!("{:>3.0}", y_max / 2.0), Style::default().fg(colors.text_muted)),
        Span::styled(format!("{:>3.0}", y_max),        Style::default().fg(colors.text_muted)),
    ];

    Widget::render(
        Chart::new(datasets)
            .block(Block::default()
                .title(Span::styled(format!(" {} ", title),
                    Style::default().fg(colors.text_muted)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.section_border)))
            .x_axis(Axis::default().bounds([0.0, x_max])
                .style(Style::default().fg(colors.section_border)))
            .y_axis(Axis::default().bounds([0.0, y_max]).labels(y_labels)
                .style(Style::default().fg(colors.section_border))),
        area, buf,
    );
}

pub fn render_throughput_chart(
    buf:              &mut Buffer,
    area:             Rect,
    download_history: &[f64],
    upload_history:   &[f64],
    colors:           &ThemeColors,
) {
    let max_val = download_history.iter().chain(upload_history.iter())
        .cloned().fold(1.0f64, f64::max);
    let y_max = (max_val * 1.2).max(10.0);
    let dl: Vec<(f64, f64)> = download_history.iter().enumerate().map(|(i,&v)|(i as f64,v)).collect();
    let ul: Vec<(f64, f64)> = upload_history.iter().enumerate().map(|(i,&v)|(i as f64,v)).collect();
    let x_max = download_history.len().max(upload_history.len()).max(1) as f64;

    Widget::render(
        Chart::new(vec![
            Dataset::default().name("▼ DL").marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line).style(Style::default().fg(colors.download_color())).data(&dl),
            Dataset::default().name("▲ UL").marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line).style(Style::default().fg(colors.upload_color())).data(&ul),
        ])
        .block(Block::default().title(" Throughput ").borders(Borders::ALL)
            .border_style(Style::default().fg(colors.section_border)))
        .x_axis(Axis::default().bounds([0.0, x_max]).style(Style::default().fg(colors.section_border)))
        .y_axis(Axis::default().bounds([0.0, y_max])
            .labels(vec![Span::raw("0"), Span::raw(format!("{:.0}", y_max))])
            .style(Style::default().fg(colors.section_border))),
        area, buf,
    );
}

fn percentile_90(data: &[f64]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut s = data.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((s.len() as f64 * 0.9) as usize).min(s.len() - 1)]
}
