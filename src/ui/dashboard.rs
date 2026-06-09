use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
    Frame,
};
use crate::app::state::{AppState, Phase};
use crate::ui::{graphs, history, theme::ThemeColors, widgets};

pub fn render(f: &mut Frame, state: &AppState) {
    let colors = ThemeColors::for_theme(&state.theme);
    let area = f.area();
    f.render_widget(Clear, area);

    // Full outer border — blue/purple like the screenshot
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.outer_border));
    let inner = outer_block.inner(area);
    outer_block.render(area, f.buffer_mut());

    match &state.phase {
        Phase::History => {
            history::render_history(f.buffer_mut(), inner, &state.history, state.history_scroll, &colors);
            return;
        }
        Phase::Help => {
            render_help(f.buffer_mut(), inner, &colors);
            return;
        }
        _ => {}
    }

    // ── Root vertical slices ────────────────────────────────────────
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),   // nav bar
            Constraint::Length(2),   // animated status indicator
            Constraint::Length(10),  // throughput sparkline
            Constraint::Length(1),   // spacer
            Constraint::Length(12),  // download + upload bars + big numbers
            Constraint::Length(1),   // spacer
            Constraint::Min(8),      // bottom: history mini-graph | server details
        ])
        .split(inner);

    render_nav(f.buffer_mut(), rows[0], state, &colors);
    render_status_bar(f.buffer_mut(), rows[1], state, &colors);
    render_sparkline_section(f.buffer_mut(), rows[2], state, &colors);
    render_speed_section(f.buffer_mut(), rows[4], state, &colors);
    render_bottom_section(f.buffer_mut(), rows[6], state, &colors);
}

// ─────────────────────────────────────────────────────────────────────────────
// NAV BAR — centered, icon + label with underlined shortcut key
// ─────────────────────────────────────────────────────────────────────────────
fn render_nav(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let items: &[(&str, &str, &str)] = &[
        ("↺", "R", "ETEST"),
        ("⊟", "H", "ISTORY"),
        ("◑", "T", "HEME"),
        ("↑", "E", "XPORT"),
        ("?", "", "HELP"),
    ];

    let item_widths: usize = items.iter().map(|(ic,k,r)| ic.len() + k.len() + r.len() + 4).sum();
    let pad = (area.width as usize).saturating_sub(item_widths) / 2;

    let mut spans: Vec<Span> = vec![Span::raw(" ".repeat(pad))];

    for (icon, key, rest) in items {
        spans.push(Span::styled(*icon, Style::default().fg(colors.accent_orange)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            *key,
            Style::default().fg(colors.accent_orange)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
        spans.push(Span::styled(
            *rest,
            Style::default().fg(colors.nav_text).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw("   "));
    }

    Paragraph::new(Line::from(spans))
        .style(Style::default().bg(colors.bg))
        .render(area, buf);
}

// ─────────────────────────────────────────────────────────────────────────────
// ANIMATED STATUS BAR — spinner + phase label + animated progress pulse
// ─────────────────────────────────────────────────────────────────────────────
fn render_status_bar(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let tick = state.animation_tick;

    // Spinner frames for "in-progress" phases
    let spinner_frames = ["⠋","⠙","⠹","⠸","⠼","⠴","⠦","⠧","⠇","⠏"];
    let spinner = spinner_frames[(tick as usize) % spinner_frames.len()];

    let (phase_label, is_active, is_done, is_error) = match &state.phase {
        Phase::Init                => ("Initializing",       true,  false, false),
        Phase::ConnectivityCheck   => ("Checking connectivity", true, false, false),
        Phase::ServerSelection     => ("Selecting best server", true, false, false),
        Phase::LatencyMeasurement  => ("Measuring latency / ping", true, false, false),
        Phase::Download            => ("Download test running", true, false, false),
        Phase::Upload              => ("Upload test running",   true, false, false),
        Phase::Results             => ("Test complete",        false, true,  false),
        Phase::Error(_)            => ("Test failed",          false, false, true),
        _                          => ("",                     false, false, false),
    };

    // ── Row 0: spinner icon + phase text + animated dots ──
    let icon = if is_done { "✓" } else if is_error { "✗" } else { spinner };
    let icon_color = if is_done { colors.accent_green }
                     else if is_error { colors.accent_red }
                     else { colors.accent_orange };

    // Animated trailing dots — only animate while test is running
    let dots = if is_active { match tick % 4 { 0=>"", 1=>".", 2=>"..", _=>"..." } } else { "" };

    // Build phase progress pill indicators
    let phases = [
        ("CONNECT", matches!(state.phase,
            Phase::ConnectivityCheck|Phase::ServerSelection|Phase::LatencyMeasurement|Phase::Download|Phase::Upload|Phase::Results)),
        ("PING",    matches!(state.phase,
            Phase::LatencyMeasurement|Phase::Download|Phase::Upload|Phase::Results)),
        ("DOWNLOAD",matches!(state.phase,
            Phase::Download|Phase::Upload|Phase::Results)),
        ("UPLOAD",  matches!(state.phase,
            Phase::Upload|Phase::Results)),
        ("DONE",    matches!(state.phase, Phase::Results)),
    ];

    // ── Row 0: spinner + label ──
    let row0 = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
    // ── Row 1: progress pills ──
    let row1 = Rect { x: area.x, y: area.y + 1, width: area.width, height: 1 };

    let mut spans0: Vec<Span> = vec![
        Span::raw("  "),
        Span::styled(icon,  Style::default().fg(icon_color).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(phase_label, Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
        Span::styled(dots, Style::default().fg(colors.text_muted)),
    ];

    // Add animated pulse wave on active phases
    if is_active {
        let wave_width: usize = 20;
        let wave_pos = (tick as usize * 2) % (area.width as usize + wave_width);
        let label_len = phase_label.len() + 6;
        let available = area.width as usize;
        if available > label_len + wave_width + 4 {
            let spacer = available.saturating_sub(label_len + wave_width + 4);
            spans0.push(Span::raw(" ".repeat(spacer.min(10))));
            // Render a scrolling pulse bar
            for i in 0..wave_width {
                let abs_x = wave_pos + i;
                let intensity = {
                    let center = wave_width / 2;
                    let dist = if i > center { i - center } else { center - i };
                    (wave_width / 2).saturating_sub(dist)
                };
                let ch = match intensity {
                    0 => '░', 1..=2 => '▒', 3..=5 => '▓', _ => '█'
                };
                let alpha = (intensity as f64 / (wave_width as f64 / 2.0)).clamp(0.0, 1.0);
                let r = (colors.accent_orange.to_string().len() as f64 * alpha) as u8; // rough
                spans0.push(Span::styled(
                    ch.to_string(),
                    Style::default().fg(Color::Rgb(
                        (255.0 * alpha * 0.65) as u8,
                        (140.0 * alpha * 0.35) as u8,
                        0,
                    )),
                ));
            }
        }
    }

    Paragraph::new(Line::from(spans0)).render(row0, buf);

    // ── Row 1: stage pills ──
    let mut spans1: Vec<Span> = vec![Span::raw("  ")];
    for (label, done) in &phases {
        let is_current = match (*label, &state.phase) {
            ("CONNECT", Phase::ConnectivityCheck | Phase::ServerSelection) => true,
            ("PING",    Phase::LatencyMeasurement) => true,
            ("DOWNLOAD",Phase::Download) => true,
            ("UPLOAD",  Phase::Upload) => true,
            _ => false,
        };

        let (fg, bg, prefix) = if is_current {
            let pulse = if (tick / 3) % 2 == 0 { colors.accent_orange } else { Color::Rgb(200,100,0) };
            (colors.bg, pulse, "►")
        } else if *done {
            (colors.bg, colors.accent_green, "✓")
        } else {
            (colors.text_muted, colors.surface, "○")
        };

        spans1.push(Span::styled(
            format!(" {} {} ", prefix, label),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        ));
        spans1.push(Span::raw("  "));
    }

    Paragraph::new(Line::from(spans1)).render(row1, buf);
}

// ─────────────────────────────────────────────────────────────────────────────
// THROUGHPUT SPARKLINE — full-width box, DL orange + UL grey, both bottom-up
// ─────────────────────────────────────────────────────────────────────────────
fn render_sparkline_section(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let block = Block::default()
        .title(Span::styled(
            "── THROUGHPUT SPARKLINE ──",
            Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.accent_orange));
    let inner = block.inner(area);
    block.render(area, buf);

    // Layout: peak label top-right | chart body fills the rest
    // Split inner into: [chart area | right info column]
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),      // main sparkline
            Constraint::Length(2),    // gap
            Constraint::Length(14),   // peak / labels right column
        ])
        .split(inner);

    let chart_area = split[0];
    let info_area  = split[2];

    // ── Right info column: peak + current values ──
    let info_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // DL peak
            Constraint::Length(1), // DL current
            Constraint::Length(1), // spacer
            Constraint::Length(1), // UL peak
            Constraint::Length(1), // UL current
            Constraint::Min(0),
        ])
        .split(info_area);

    Paragraph::new(Line::from(Span::styled(
        format!("Peak {:.1}", state.download.peak_mbps),
        Style::default().fg(colors.text_muted),
    ))).render(info_rows[0], buf);
    Paragraph::new(Line::from(Span::styled(
        format!("{:.2} Mbps", state.download.current_mbps),
        Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD),
    ))).render(info_rows[1], buf);
    Paragraph::new(Line::from(Span::styled(
        format!("Peak {:.1}", state.upload.peak_mbps),
        Style::default().fg(colors.text_muted),
    ))).render(info_rows[3], buf);
    Paragraph::new(Line::from(Span::styled(
        format!("{:.2} Mbps", state.upload.current_mbps),
        Style::default().fg(Color::Rgb(200, 200, 210)).add_modifier(Modifier::BOLD),
    ))).render(info_rows[4], buf);

    // ── Sparkline chart area: split vertically for DL (top) and UL (bottom) ──
    let spark_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // download — more prominent
            Constraint::Percentage(40), // upload
        ])
        .split(chart_area);

    render_block_sparkline(buf, spark_rows[0], &state.download.history, colors.accent_orange, colors.bg);
    render_block_sparkline(buf, spark_rows[1], &state.upload.history,   Color::Rgb(160, 160, 175), colors.bg);
}

fn render_block_sparkline(buf: &mut Buffer, area: Rect, data: &[f64], color: Color, bg: Color) {
    if area.width == 0 || area.height == 0 { return; }
    let width  = area.width as usize;
    let height = area.height as usize;

    // Sub-character vertical precision using 8 block levels
    let block_chars = ['▁','▂','▃','▄','▅','▆','▇','█'];
    // Dim dot for empty areas — matches reference screenshot aesthetic
    let empty_dot   = '·';
    let empty_color = Color::Rgb(45, 42, 65);

    // Find global max for scale (use at least 1.0 to avoid div-by-zero)
    let max = data.iter().cloned().fold(1.0f64, f64::max);

    // Pad or trim data to exactly `width` columns (newest data on the right)
    let visible: Vec<f64> = if data.len() >= width {
        data[data.len()-width..].to_vec()
    } else {
        let mut v = vec![0.0f64; width - data.len()];
        v.extend_from_slice(data);
        v
    };

    for (x_off, &val) in visible.iter().enumerate() {
        let x = area.x + x_off as u16;
        let ratio = (val / max).clamp(0.0, 1.0);
        // Total sub-row units this column fills (each row = 8 eighths)
        let total_eighths = (ratio * (height as f64 * 8.0)).round() as usize;
        let full_rows  = total_eighths / 8;
        let partial    = total_eighths % 8;

        for row in 0..height {
            // row 0 = bottom, row (height-1) = top
            let y = area.y + (height - 1 - row) as u16;
            let cell = buf.get_mut(x, y);
            cell.set_bg(bg);

            if row < full_rows {
                // Fully filled row
                cell.set_char('█').set_fg(color);
            } else if row == full_rows && partial > 0 {
                // Partial top row — use fractional block char
                cell.set_char(block_chars[partial - 1]).set_fg(color);
            } else {
                // Above the bar OR no data yet — show subtle dot grid
                if (x_off + row) % 2 == 0 {
                    cell.set_char(empty_dot).set_fg(empty_color);
                } else {
                    cell.set_char(' ');
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SPEED SECTION — bar charts left, big numbers right
// ─────────────────────────────────────────────────────────────────────────────
fn render_speed_section(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.section_border));
    let inner = block.inner(area);
    block.render(area, buf);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55),  // bar charts
            Constraint::Percentage(45),  // big numbers
        ])
        .split(inner);

    render_bar_charts(buf, cols[0], state, colors);
    render_big_numbers(buf, cols[1], state, colors);
}

fn render_bar_charts(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // DOWNLOAD label
            Constraint::Length(4),  // download bars
            Constraint::Length(1),  // UPLOAD label
            Constraint::Length(4),  // upload bars
            Constraint::Min(0),
        ])
        .split(area);

    // DOWNLOAD label
    Paragraph::new(Line::from(
        Span::styled("  DOWNLOAD",
            Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD))
    )).render(rows[0], buf);

    // Download bars (orange)
    render_bar_chart(buf, rows[1], &state.download.history, colors.accent_orange, colors.bg);

    // UPLOAD label
    Paragraph::new(Line::from(
        Span::styled("  UPLOAD",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
    )).render(rows[2], buf);

    // Upload bars (white/light)
    render_bar_chart(buf, rows[3], &state.upload.history, Color::Rgb(200, 200, 210), colors.bg);
}

fn render_bar_chart(buf: &mut Buffer, area: Rect, data: &[f64], color: Color, bg: Color) {
    if area.width == 0 || area.height == 0 { return; }
    let max = data.iter().cloned().fold(0.1f64, f64::max);
    let width = (area.width as usize).saturating_sub(2); // side padding
    let height = area.height as usize;

    // Each bar is 2 chars wide with 1 gap — like the screenshot
    let bar_count = (width + 1) / 3;
    let visible: Vec<f64> = if data.len() >= bar_count {
        data[data.len()-bar_count..].to_vec()
    } else {
        let mut v = vec![0.0f64; bar_count - data.len()];
        v.extend_from_slice(data);
        v
    };

    for (i, &val) in visible.iter().enumerate() {
        let x_base = area.x + 1 + (i * 3) as u16;
        let ratio = (val / max).clamp(0.0, 1.0);
        let filled_rows = (ratio * height as f64).round() as usize;

        for row in 0..height {
            let y = area.y + (height - 1 - row) as u16;
            for dx in 0..2u16 {
                let x = x_base + dx;
                if x >= area.x + area.width { break; }
                let cell = buf.get_mut(x, y);
                cell.set_bg(bg);
                if row < filled_rows {
                    cell.set_char('█').set_fg(color);
                } else {
                    cell.set_char(' ');
                }
            }
        }
    }
}

fn render_big_numbers(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Peak DL label
            Constraint::Length(4),  // big DL number
            Constraint::Length(1),  // Peak UL label
            Constraint::Length(4),  // big UL number
            Constraint::Min(0),
        ])
        .split(area);

    // Peak DL
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  Peak {:.1}", state.download.peak_mbps),
            Style::default().fg(colors.accent_orange),
        ),
    ])).render(rows[0], buf);

    // Big DL number — simulate large text with bold + extra chars
    render_huge_number(buf, rows[1], state.download.current_mbps, colors.accent_orange);

    // Peak UL
    Paragraph::new(Line::from(vec![
        Span::styled(
            format!("  Peak {:.1}", state.upload.peak_mbps),
            Style::default().fg(Color::Rgb(200,200,210)),
        ),
    ])).render(rows[2], buf);

    // Big UL number
    render_huge_number(buf, rows[3], state.upload.current_mbps, Color::White);
}

fn render_huge_number(buf: &mut Buffer, area: Rect, value: f64, color: Color) {
    // Use double-height effect via stacked lines with modifier
    let num_str = format!("{:.2}", value);
    let line1 = Line::from(vec![
        Span::styled(
            format!("  {} Mbps", num_str),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    // Render on the vertically-centered row for a "large" feel
    let y_mid = area.y + area.height / 2;
    let render_area = Rect { y: y_mid, height: 1, ..area };
    Paragraph::new(line1).render(render_area, buf);

    // Duplicate one row above for double-height feel
    if y_mid > area.y {
        let line2 = Line::from(vec![
            Span::styled(
                format!("  {} Mbps", num_str),
                Style::default().fg(color).add_modifier(Modifier::BOLD | Modifier::DIM),
            ),
        ]);
        let above = Rect { y: y_mid - 1, height: 1, ..area };
        Paragraph::new(line2).render(above, buf);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// BOTTOM SECTION — TEST HISTORY left | SERVER DETAILS right
// ─────────────────────────────────────────────────────────────────────────────
fn render_bottom_section(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(52),
            Constraint::Length(2),
            Constraint::Percentage(48),
        ])
        .split(area);

    render_history_graph(buf, cols[0], state, colors);
    render_server_details(buf, cols[2], state, colors);
}

fn render_history_graph(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    use ratatui::widgets::{Axis, Chart, Dataset, GraphType};
    use ratatui::symbols::Marker;

    let block = Block::default()
        .title(Span::styled(
            "─── TEST HISTORY  [H] for table ───",
            Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.section_border));
    let inner = block.inner(area);
    block.render(area, buf);

    if state.history.is_empty() {
        Paragraph::new(Line::from(Span::styled(
            "  No history yet — run a test first",
            Style::default().fg(colors.text_muted),
        ))).render(inner, buf);
        return;
    }

    // Split: legend top, chart fills rest
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(4),
        ])
        .split(inner);

    // Legend row
    Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled("── DL", Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("── UL", Style::default().fg(Color::Rgb(160, 160, 220)).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("── Ping", Style::default().fg(Color::Rgb(80, 210, 140)).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("(newest →)", Style::default().fg(colors.text_muted)),
    ])).render(rows[0], buf);

    let chart_area = rows[1];

    // Build data points: x = index (0..n), y = value
    let n = state.history.len();
    let dl_data: Vec<(f64, f64)> = state.history.iter().enumerate()
        .map(|(i, e)| (i as f64, e.download_mbps)).collect();
    let ul_data: Vec<(f64, f64)> = state.history.iter().enumerate()
        .map(|(i, e)| (i as f64, e.upload_mbps)).collect();
    let ping_data: Vec<(f64, f64)> = state.history.iter().enumerate()
        .map(|(i, e)| (i as f64, e.ping_ms)).collect();

    // Scale axes
    let max_speed = dl_data.iter().chain(ul_data.iter())
        .map(|(_,v)| *v).fold(1.0f64, f64::max);
    let y_speed_max = (max_speed * 1.2).max(10.0);

    let max_ping = ping_data.iter().map(|(_,v)| *v).fold(1.0f64, f64::max);
    // Normalize ping to same y-axis scale (show as proportion of max_speed)
    // We overlay ping as a line scaled to 0..y_speed_max using its own proportion
    let ping_scaled: Vec<(f64, f64)> = ping_data.iter()
        .map(|(x, v)| (*x, (v / max_ping.max(1.0)) * y_speed_max * 0.4))
        .collect();

    let x_max = (n as f64 - 1.0).max(1.0);

    // Y-axis labels
    let y_mid  = format!("{:.0}", y_speed_max / 2.0);
    let y_top  = format!("{:.0}", y_speed_max);
    let y_zero = "0".to_string();

    let datasets = vec![
        Dataset::default()
            .name("DL")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(colors.accent_orange))
            .data(&dl_data),
        Dataset::default()
            .name("UL")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Rgb(160, 160, 220)))
            .data(&ul_data),
        Dataset::default()
            .name("Ping")
            .marker(Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Rgb(80, 210, 140)))
            .data(&ping_scaled),
    ];

    let chart = Chart::new(datasets)
        .block(Block::default())
        .x_axis(
            Axis::default()
                .bounds([0.0, x_max])
                .style(Style::default().fg(colors.section_border)),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, y_speed_max])
                .labels(vec![
                    Span::styled(y_zero, Style::default().fg(colors.text_muted)),
                    Span::styled(y_mid,  Style::default().fg(colors.text_muted)),
                    Span::styled(y_top,  Style::default().fg(colors.text_muted)),
                ])
                .style(Style::default().fg(colors.section_border)),
        );

    Widget::render(chart, chart_area, buf);
}


fn render_server_details(buf: &mut Buffer, area: Rect, state: &AppState, colors: &ThemeColors) {
    let block = Block::default()
        .title(Span::styled(
            "─── SERVER DETAILS ───",
            Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.section_border));
    let inner = block.inner(area);
    block.render(area, buf);

    let server = state.servers.get(state.selected_server_idx);
    let server_name = server.map(|s| s.name.as_str()).unwrap_or("Cloudflare");
    let server_loc  = server.map(|s| s.location.as_str()).unwrap_or("—");

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(1), // divider
            Constraint::Min(0),    // rows
        ])
        .split(inner);

    // Column header
    let header_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(rows[0]);

    Paragraph::new(Line::from(Span::styled("ITEM",
        Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD))))
        .render(header_cols[0], buf);
    Paragraph::new(Line::from(Span::styled("VALUE",
        Style::default().fg(colors.text_muted).add_modifier(Modifier::BOLD))))
        .render(header_cols[1], buf);

    for x in inner.x..inner.x + inner.width {
        buf[(x, rows[1].y)].set_char('─').set_fg(colors.section_border);
    }

    // Compute quality score for display
    let (quality_score, quality_grade) = state.compute_quality_score();
    let stars = {
        let filled = quality_score.round() as usize;
        let empty  = 5usize.saturating_sub(filled);
        format!("{}{} ({})", "★".repeat(filled), "☆".repeat(empty), quality_grade)
    };
    let grade_color = match quality_grade.as_str() {
        "A" => Color::Rgb(70, 210, 120),   // green
        "B" => Color::Rgb(140, 210, 80),   // yellow-green
        "C" => Color::Rgb(230, 190, 50),   // yellow
        "D" => Color::Rgb(230, 130, 50),   // orange
        _   => Color::Rgb(220, 70,  70),   // red
    };

    // Data rows
    let fields: &[(&str, String)] = &[
        ("ISP",      state.ip_info.isp.clone()),
        ("IP",       state.ip_info.ip.clone()),
        ("SERVER",   server_name.to_string()),
        ("LOCATION", state.ip_info.location.clone()),
        ("LATENCY",  format!("{:.0}ms", state.latency.avg_ms)),
        ("JITTER",   format!("{:.1}ms", state.latency.jitter_ms)),
        ("PKT LOSS", format!("{:.1}%",  state.latency.packet_loss_pct)),
    ];

    for (i, (key, val)) in fields.iter().enumerate() {
        let y = rows[2].y + i as u16;
        if y >= rows[2].y + rows[2].height { break; }

        let row_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(Rect { y, height: 1, ..rows[2] });

        Paragraph::new(Line::from(Span::styled(
            *key, Style::default().fg(colors.accent_orange)
        ))).render(row_cols[0], buf);

        Paragraph::new(Line::from(Span::styled(
            val.as_str(), Style::default().fg(colors.text)
        ))).render(row_cols[1], buf);
    }

    // Quality score row — shown after the regular fields
    let quality_y = rows[2].y + fields.len() as u16;
    if quality_y < rows[2].y + rows[2].height {
        // Divider before quality
        let div_y = quality_y;
        for x in inner.x..inner.x + inner.width {
            if div_y < rows[2].y + rows[2].height {
                buf[(x, div_y)].set_char('─').set_fg(colors.section_border);
            }
        }
        let stars_y = quality_y + 1;
        if stars_y < rows[2].y + rows[2].height {
            let star_cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(Rect { y: stars_y, height: 1, ..rows[2] });

            Paragraph::new(Line::from(Span::styled(
                "QUALITY",
                Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD),
            ))).render(star_cols[0], buf);

            Paragraph::new(Line::from(Span::styled(
                stars,
                Style::default().fg(grade_color).add_modifier(Modifier::BOLD),
            ))).render(star_cols[1], buf);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HELP overlay
// ─────────────────────────────────────────────────────────────────────────────
fn render_help(buf: &mut Buffer, area: Rect, colors: &ThemeColors) {
    let w = 42u16.min(area.width);
    let h = 12u16.min(area.height);
    let popup = Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w, height: h,
    };
    for y in popup.y..popup.y + popup.height {
        for x in popup.x..popup.x + popup.width {
            buf[(x, y)].set_bg(colors.surface);
        }
    }
    let rows = vec![
        Line::raw(""),
        kv("  r  / ↺    ", "Retest",                   colors),
        kv("  h  / ⊟    ", "History",                  colors),
        kv("  e  / ↑    ", "Export result (JSON)",      colors),
        kv("  t  / ◑    ", "Toggle theme",              colors),
        kv("  ↑ ↓       ", "Scroll history",            colors),
        kv("  q / Esc   ", "Quit / close",              colors),
        Line::raw(""),
        Line::from(Span::styled("  Press any key to close",
            Style::default().fg(colors.text_muted))),
    ];
    Paragraph::new(rows)
        .block(Block::default()
            .title(Span::styled(" ? HELP ", Style::default()
                .fg(colors.accent_orange).add_modifier(Modifier::BOLD)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.outer_border))
            .style(Style::default().bg(colors.surface)))
        .render(popup, buf);
}

fn kv<'a>(key: &'a str, val: &'a str, colors: &ThemeColors) -> Line<'a> {
    Line::from(vec![
        Span::styled(key, Style::default().fg(colors.accent_orange).add_modifier(Modifier::BOLD)),
        Span::styled(val, Style::default().fg(colors.text)),
    ])
}
