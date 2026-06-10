use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Clear, Dataset, GraphType, Paragraph, Widget},
    Frame,
};
use crate::app::state::{AppState, Phase, WorkerState, DASHBOARD_HISTORY_WINDOW};
use crate::ui::{history, theme::ThemeColors};

pub fn render(f: &mut Frame, state: &AppState) {
    let c = ThemeColors::for_theme(&state.theme);
    let area = f.area();
    f.render_widget(Clear, area);

    draw_outer_border(f.buffer_mut(), area, &c);
    let inner = shrink(area, 1);

    match &state.phase {
        Phase::History => {
            history::render_history(f.buffer_mut(), inner, &state.history, state.history_scroll, &c);
            return;
        }
        Phase::Help => {
            render_help(f.buffer_mut(), inner, &c);
            return;
        }
        _ => {}
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(14),
            Constraint::Length(1),
            Constraint::Min(8),
        ])
        .split(inner);

    render_nav(f.buffer_mut(), rows[0], state, &c);
    render_stage_pills(f.buffer_mut(), rows[1], state, &c);
    hline(f.buffer_mut(), rows[2].x, rows[2].y, rows[2].width, &c, true);
    render_hero_panel(f.buffer_mut(), rows[3], state, &c);
    hline(f.buffer_mut(), rows[4].x, rows[4].y, rows[4].width, &c, true);
    render_bottom(f.buffer_mut(), rows[5], state, &c);
}

fn draw_outer_border(buf: &mut Buffer, area: Rect, c: &ThemeColors) {
    let col = Style::default().fg(c.panel_border);

    buf[(area.x, area.y)].set_char('╭').set_style(col);
    buf[(area.x + area.width - 1, area.y)].set_char('╮').set_style(col);
    buf[(area.x, area.y + area.height - 1)].set_char('╰').set_style(col);
    buf[(area.x + area.width - 1, area.y + area.height - 1)].set_char('╯').set_style(col);

    for x in (area.x + 1)..(area.x + area.width - 1) {
        buf[(x, area.y)].set_char('─').set_style(col);
        buf[(x, area.y + area.height - 1)].set_char('─').set_style(col);
    }

    for y in (area.y + 1)..(area.y + area.height - 1) {
        buf[(area.x, y)].set_char('│').set_style(col);
        buf[(area.x + area.width - 1, y)].set_char('│').set_style(col);
    }
}

fn hline(buf: &mut Buffer, x: u16, y: u16, w: u16, c: &ThemeColors, tee: bool) {
    let col = Style::default().fg(c.divider);
    let brd = Style::default().fg(c.panel_border);

    if tee && x > 0 {
        buf[(x - 1, y)].set_char('├').set_style(brd);
    }
    for cx in x..x + w {
        buf[(cx, y)].set_char('─').set_style(col);
    }
    if tee && x + w < buf.area.width {
        buf[(x + w, y)].set_char('┤').set_style(brd);
    }
}

fn shrink(r: Rect, n: u16) -> Rect {
    Rect {
        x: r.x + n,
        y: r.y + n,
        width: r.width.saturating_sub(n * 2),
        height: r.height.saturating_sub(n * 2),
    }
}

// ════════════════════════════════════════════════════════════════════════════
// NAV BAR
// ════════════════════════════════════════════════════════════════════════════
fn render_nav(buf: &mut Buffer, area: Rect, _state: &AppState, c: &ThemeColors) {
    for x in area.x..area.x + area.width {
        buf[(x, area.y)].set_bg(c.nav_bg);
    }

    let title = "⚡ SPEEDTEST-TUI ";
    let title_len = title.len() as u16;
    let cx = area.x + area.width.saturating_sub(title_len) / 2;
    px(buf, cx, area.y, title, c.dl_primary, Modifier::BOLD);

    let actions = [("R", "ETEST"), ("H", "ISTORY"), ("T", "HEME"), ("E", "XPORT"), ("?", "HELP")];
    let right_str: String = actions.iter().map(|(k, r)| format!("  {}{}", k, r)).collect();
    let mut cx = area.x + area.width.saturating_sub(right_str.len() as u16 + 2);

    for (key, rest) in &actions {
        cx += 2;
        for ch in key.chars() {
            if cx < area.x + area.width {
                buf[(cx, area.y)]
                    .set_char(ch)
                    .set_fg(c.nav_key)
                    .set_bg(c.nav_bg)
                    .set_style(Style::default().fg(c.nav_key).bg(c.nav_bg).add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
            }
            cx += 1;
        }
        for ch in rest.chars() {
            if cx < area.x + area.width {
                buf[(cx, area.y)].set_char(ch).set_fg(c.text_muted).set_bg(c.nav_bg);
            }
            cx += 1;
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// STAGE PILLS
// ════════════════════════════════════════════════════════════════════════════
fn render_stage_pills(buf: &mut Buffer, area: Rect, state: &AppState, c: &ThemeColors) {
    let tick = state.animation_tick;
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spin = spinners[(tick as usize) % spinners.len()];

    let (label, active, done) = match &state.phase {
        Phase::Init               => ("Initializing",          true,  false),
        Phase::ConnectivityCheck  => ("Checking connectivity", true,  false),
        Phase::ServerSelection    => ("Selecting server",      true,  false),
        Phase::LatencyMeasurement => ("Measuring latency",     true,  false),
        Phase::Download           => ("↓  Download running",   true,  false),
        Phase::Upload             => ("↑  Upload running",     true,  false),
        Phase::Results            => ("✓  Test complete",      false, true),
        _                         => ("",                      false, false),
    };

    let stages: &[(&str, bool)] = &[
        ("CONNECT",  matches!(state.phase, Phase::ConnectivityCheck | Phase::ServerSelection | Phase::LatencyMeasurement | Phase::Download | Phase::Upload | Phase::Results)),
        ("PING",     matches!(state.phase, Phase::LatencyMeasurement | Phase::Download | Phase::Upload | Phase::Results)),
        ("DOWNLOAD", matches!(state.phase, Phase::Download | Phase::Upload | Phase::Results)),
        ("UPLOAD",   matches!(state.phase, Phase::Upload | Phase::Results)),
        ("DONE",     matches!(state.phase, Phase::Results)),
    ];

    let row0 = Rect { x: area.x, y: area.y,     width: area.width, height: 1 };
    let row1 = Rect { x: area.x, y: area.y + 1, width: area.width, height: 1 };

    let icon     = if done { "✓" } else { spin };
    let icon_col = if done { c.accent_green } else { c.dl_primary };
    let dots     = if active { match tick % 4 { 0 => "", 1 => ".", 2 => "..", _ => "..." } } else { "" };

    let line0 = Line::from(vec![
        Span::raw(" "),
        Span::styled(icon,  Style::default().fg(icon_col).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(label, Style::default().fg(c.text).add_modifier(Modifier::BOLD)),
        Span::styled(dots,  Style::default().fg(c.text_muted)),
    ]);
    Paragraph::new(line0).render(row0, buf);

    let mut spans = vec![Span::raw(" ")];
    for (stage, done_flag) in stages {
        let is_now = matches!(
            (*stage, &state.phase),
            ("CONNECT",  Phase::ConnectivityCheck | Phase::ServerSelection) |
            ("PING",     Phase::LatencyMeasurement) |
            ("DOWNLOAD", Phase::Download) |
            ("UPLOAD",   Phase::Upload)
        );

        let (fg, bg, pre) = if is_now {
            let pulse = if (tick / 3) % 2 == 0 { c.pill_active_bg } else { Color::Rgb(180, 90, 10) };
            (c.bg, pulse, "▶")
        } else if *done_flag {
            (c.pill_done_fg, c.pill_done_bg, "✓")
        } else {
            (c.text_faint, c.surface, "·")
        };

        spans.push(Span::styled(
            format!(" {} {} ", pre, stage),
            Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
    }

    Paragraph::new(Line::from(spans)).render(row1, buf);
}

// ════════════════════════════════════════════════════════════════════════════
// HERO PANEL  (no nested Block border — uses vdiv only)
// ════════════════════════════════════════════════════════════════════════════
fn render_hero_panel(buf: &mut Buffer, area: Rect, state: &AppState, c: &ThemeColors) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Length(1),
            Constraint::Percentage(30),
            Constraint::Length(1),
            Constraint::Percentage(35),
        ])
        .split(area);

    render_worker_panel(buf, cols[0], "▼  DOWNLOAD", &state.download.workers, c.dl_primary, c.dl_dim, c);
    render_center_readout(buf, cols[2], state, c);
    render_worker_panel(buf, cols[4], "▲  UPLOAD",   &state.upload.workers,   c.ul_primary, c.ul_dim, c);
}

fn render_worker_panel(
    buf:     &mut Buffer,
    area:    Rect,
    title:   &str,
    workers: &[WorkerState],
    color:   Color,
    dim:     Color,
    c:       &ThemeColors,
) {
    if area.width < 6 || area.height < 6 { return; }
    fill_rect(buf, area, c.nav_bg);
    draw_soft_box(buf, area, c);

    pxc(buf, area.x, area.y, area.width, title, color, Modifier::BOLD);

    let inner = shrink(area, 1);
    let chart_h = inner.height.saturating_sub(3);
    let chart_area = Rect { x: inner.x, y: inner.y, width: inner.width, height: chart_h };
    let agg_y = inner.y + inner.height.saturating_sub(2);
    let stat_y = inner.y + inner.height.saturating_sub(1);

    let n = if workers.is_empty() { 8 } else { workers.len() };
    draw_worker_bars(buf, chart_area, workers, n, color, dim, c);

    let w_count = workers.iter().filter(|w| w.active).count();
    let agg: f64 = workers.iter().map(|w| w.speed_mbps).sum();
    pxc(buf, area.x, agg_y, area.width, &format!("{} active workers", w_count), c.text_faint, Modifier::empty());
    pxc(buf, area.x, stat_y, area.width, &format!("{:.1} Mbps total", agg), color, Modifier::BOLD);
}

fn draw_worker_bars(
    buf:     &mut Buffer,
    area:    Rect,
    workers: &[WorkerState],
    n:       usize,
    color:   Color,
    dim:     Color,
    c:       &ThemeColors,
) {
    if area.width == 0 || area.height == 0 || n == 0 { return; }

    let h = area.height as usize;

    // Dynamically fit all n bars into the available width.
    // Try bw=2,gap=1 first; fall back to bw=1,gap=1; then bw=1,gap=0.
    let (bw, gap) = {
        let needed_2_1 = n as u16 * 2 + n.saturating_sub(1) as u16;
        let needed_1_1 = n as u16 * 1 + n.saturating_sub(1) as u16;
        if area.width >= needed_2_1 {
            (2u16, 1u16)
        } else if area.width >= needed_1_1 {
            (1u16, 1u16)
        } else {
            (1u16, 0u16)
        }
    };

    let total_w = n as u16 * bw + n.saturating_sub(1) as u16 * gap;
    let x0      = area.x + area.width.saturating_sub(total_w) / 2;
    let max_spd = workers.iter().map(|w| w.speed_mbps).fold(0.1f64, f64::max);

    for i in 0..n {
        let spd      = workers.get(i).map(|w| w.speed_mbps).unwrap_or(0.0);
        let active   = workers.get(i).map(|w| w.active).unwrap_or(false);
        let complete = workers.get(i).map(|w| w.complete).unwrap_or(false);

        let ratio  = (spd / max_spd).clamp(0.0, 1.0);
        let filled = ((ratio * h as f64).round() as usize).max(if spd > 0.0 { 1 } else { 0 });
        let bar_col = if complete { dim } else if active { color } else { c.text_faint };

        let bx = x0 + i as u16 * (bw + gap);
        for row in 0..h {
            let y = area.y + (h - 1 - row) as u16;
            for dx in 0..bw {
                let x = bx + dx;
                if x >= area.x + area.width { continue; }
                if row < filled {
                    let ch = if row == filled.saturating_sub(1) && ratio < 1.0 {
                        frac_block((ratio * h as f64) - (filled as f64 - 1.0))
                    } else { '█' };
                    buf[(x, y)].set_char(ch).set_fg(bar_col);
                } else if (i + row) % 4 == 0 {
                    buf[(x, y)].set_char('·').set_fg(c.text_faint);
                } else {
                    buf[(x, y)].set_char(' ');
                }
            }
        }
    }
}


fn frac_block(f: f64) -> char {
    match (f * 8.0).round() as u8 {
        0 => ' ', 1 => '▁', 2 => '▂', 3 => '▃',
        4 => '▄', 5 => '▅', 6 => '▆', _ => '▇',
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CENTER READOUT
// ─────────────────────────────────────────────────────────────────────────────
fn render_center_readout(buf: &mut Buffer, area: Rect, state: &AppState, c: &ThemeColors) {
    if area.width < 8 || area.height < 6 { return; }
    fill_rect(buf, area, c.nav_bg);
    draw_soft_box(buf, area, c);
    pxc(buf, area.x, area.y, area.width, "LIVE THROUGHPUT", c.text, Modifier::BOLD);

    let inner = shrink(area, 1);
    let half = inner.height / 2;
    let dl_area = Rect { x: inner.x, y: inner.y, width: inner.width, height: half };
    let ul_area = Rect { x: inner.x, y: inner.y + half, width: inner.width, height: inner.height - half };

    if ul_area.y > inner.y {
        draw_hrule(buf, inner.x, ul_area.y, inner.width, c.divider);
    }

    pxr(buf, dl_area.x, dl_area.y, dl_area.width,
        &format!("peak {:.1}", state.download.peak_mbps), c.text_muted, Modifier::empty());
    draw_block_speed(
        buf,
        Rect { x: dl_area.x, y: dl_area.y + 1, width: dl_area.width, height: dl_area.height.saturating_sub(1) },
        state.download.current_mbps, c.dl_primary,
    );

    pxr(buf, ul_area.x, ul_area.y, ul_area.width,
        &format!("peak {:.1}", state.upload.peak_mbps), c.text_muted, Modifier::empty());
    draw_block_speed(
        buf,
        Rect { x: ul_area.x, y: ul_area.y + 1, width: ul_area.width, height: ul_area.height.saturating_sub(1) },
        state.upload.current_mbps, c.ul_primary,
    );
}

fn render_bottom(buf: &mut Buffer, area: Rect, state: &AppState, c: &ThemeColors) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Length(1), Constraint::Percentage(48)])
        .split(area);

    render_history_graph(buf, cols[0], state, c);
    render_server_card(buf, cols[2], state, c);
}

fn render_history_graph(buf: &mut Buffer, area: Rect, state: &AppState, c: &ThemeColors) {
    let title = Line::from(Span::styled(
        "TEST HISTORY",
        Style::default().fg(c.text).add_modifier(Modifier::BOLD),
    ));

    if area.width < 8 || area.height < 6 { return; }
    fill_rect(buf, area, c.nav_bg);
    draw_soft_box(buf, area, c);
    let frame = shrink(area, 1);
    let block = Block::default()
        .title(title.clone())
        .title_alignment(Alignment::Center)
        .borders(Borders::NONE);
    let inner = block.inner(frame);
    block.render(frame, buf);
    if inner.width < 8 || inner.height < 6 { return; }

    let card = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    };
    if card.width < 8 || card.height < 5 { return; }
    fill_rect(buf, card, c.nav_bg);
    draw_soft_box(buf, card, c);

    if state.history.is_empty() {
        let chart_area = shrink(card, 1);
        fill_rect(buf, chart_area, c.nav_bg);
        pxc(buf, chart_area.x, chart_area.y + chart_area.height / 2, chart_area.width,
            "No history yet — run a test first", c.text_faint, Modifier::empty());
        return;
    }

    let n = state.history.len();
    let window_len = n.min(DASHBOARD_HISTORY_WINDOW);
    let max_start = n.saturating_sub(window_len);
    let start = if state.history_graph_follow_newest {
        max_start
    } else {
        state.history_graph_start.min(max_start)
    };
    let end = start + window_len;
    let visible = &state.history[start..end];

    let dl_data: Vec<(f64, f64)> = visible.iter()
        .enumerate()
        .map(|(i, r)| (i as f64, r.download_mbps))
        .collect();
    let ul_data: Vec<(f64, f64)> = visible.iter()
        .enumerate()
        .map(|(i, r)| (i as f64, r.upload_mbps))
        .collect();
    let ping_data: Vec<(f64, f64)> = visible.iter()
        .enumerate()
        .map(|(i, r)| (i as f64, r.ping_ms / 10.0))
        .collect();

    let max_dl = dl_data.iter().map(|d| d.1).fold(1.0f64, f64::max);
    let max_ul = ul_data.iter().map(|d| d.1).fold(1.0f64, f64::max);
    let y_max  = (max_dl.max(max_ul) * 1.25).max(10.0);

    let chart_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(c.nav_bg));
    let chart_area = chart_block.inner(shrink(card, 1));
    fill_rect(buf, chart_area, c.nav_bg);
    let chart_style = Style::default().bg(c.nav_bg);
    if chart_area.height < 3 {
        return;
    }

    let plot_area = Rect {
        x: chart_area.x,
        y: chart_area.y,
        width: chart_area.width,
        height: chart_area.height.saturating_sub(1),
    };
    let left_label = if start == 0 {
        "oldest".to_string()
    } else {
        format!("#{}", start + 1)
    };
    let right_label = if end >= n {
        "newest".to_string()
    } else {
        format!("#{} →", end)
    };

    let chart = Chart::new(vec![
        Dataset::default().name("▼ DL")
            .marker(symbols::Marker::Dot).graph_type(GraphType::Line)
            .style(chart_style.fg(c.dl_primary)).data(&dl_data),
        Dataset::default().name("▲ UL")
            .marker(symbols::Marker::Dot).graph_type(GraphType::Line)
            .style(chart_style.fg(c.ul_primary)).data(&ul_data),
        Dataset::default().name("Ping÷10")
            .marker(symbols::Marker::Dot).graph_type(GraphType::Scatter)
            .style(chart_style.fg(c.accent_green)).data(&ping_data),
    ])
    .block(chart_block)
    .style(chart_style)
    .x_axis(Axis::default().bounds([0.0, window_len.saturating_sub(1).max(1) as f64])
        .style(chart_style.fg(c.text_faint))
        .labels(vec![
            Span::styled(left_label,  chart_style.fg(c.text_faint)),
            Span::styled(right_label, chart_style.fg(c.text_muted)),
        ]))
    .y_axis(Axis::default().bounds([0.0, y_max])
        .style(chart_style.fg(c.text_faint))
        .labels(vec![
            Span::styled("0",                   chart_style.fg(c.text_faint)),
            Span::styled(format!("{:.0}", y_max / 2.0), chart_style.fg(c.text_muted)),
            Span::styled(format!("{:.0}", y_max),       chart_style.fg(c.text_muted)),
        ]));

    chart.render(plot_area, buf);

    let footer_y = chart_area.y + chart_area.height - 1;
    let verbose = format!(
        "{}-{} of {}  [← older] [→ newer] [Home oldest] [End newest]",
        start + 1,
        end,
        n
    );
    let compact = format!("{}-{} of {}", start + 1, end, n);
    let footer = if verbose.len() as u16 <= chart_area.width {
        verbose.as_str()
    } else {
        compact.as_str()
    };
    px(buf, chart_area.x, footer_y, footer, c.text_muted, Modifier::empty());
}

fn render_server_card(buf: &mut Buffer, area: Rect, state: &AppState, c: &ThemeColors) {
    let title = Line::from(Span::styled(
        "SERVER DETAILS",
        Style::default().fg(c.text).add_modifier(Modifier::BOLD),
    ));
    fill_rect(buf, area, c.nav_bg);
    draw_soft_box(buf, area, c);

    let frame = shrink(area, 1);
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::NONE);
    let inner = block.inner(frame);
    block.render(frame, buf);
    if inner.height == 0 || inner.width < 8 { return; }

    let server = state.servers.get(state.selected_server_idx);
    let server_name = server
        .map(|s| s.name.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Cloudflare");
    let server_location = server
        .map(|s| s.location.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if state.ip_info.location.is_empty() {
                "Unknown"
            } else {
                state.ip_info.location.as_str()
            }
        });
    let isp = if state.ip_info.isp.is_empty() {
        "Unavailable".to_string()
    } else {
        state.ip_info.isp.clone()
    };
    let ip = if state.ip_info.ip.is_empty() {
        "Unavailable".to_string()
    } else {
        state.ip_info.ip.clone()
    };

    let lat_col = c.latency_color(state.latency.avg_ms);
    let loss_col = if state.latency.packet_loss_pct == 0.0 {
        c.accent_green
    } else if state.latency.packet_loss_pct < 2.0 {
        c.accent_yellow
    } else {
        c.accent_red
    };

    let (score, grade) = state.compute_quality_score();
    let grade_col = c.quality_color(&grade);

    let card = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    };
    if card.width < 8 || card.height < 6 { return; }

    for y in card.y..card.y + card.height {
        for x in card.x..card.x + card.width {
            buf[(x, y)].set_bg(c.nav_bg);
        }
    }
    draw_soft_box(buf, card, c);

    let table = shrink(card, 1);
    let divider_x = table.x + (table.width.saturating_mul(34) / 100).max(11).min(table.width.saturating_sub(12));

    draw_vrule(buf, divider_x, table.y, table.height, c.divider);
    draw_hrule(buf, table.x, table.y + 1, table.width, c.divider);

    px(buf, table.x + 1, table.y, "ITEM", c.text_faint, Modifier::BOLD);
    px(buf, divider_x + 2, table.y, "VALUE", c.text_muted, Modifier::BOLD);

    let rows: Vec<(&str, String, Color, Modifier)> = vec![
        ("ISP", isp, c.text, Modifier::empty()),
        ("IP", ip, c.accent_teal, Modifier::empty()),
        ("SERVER", server_name.to_string(), c.text, Modifier::BOLD),
        ("LOCATION", server_location.to_string(), c.text_muted, Modifier::empty()),
        ("LATENCY", format!("{:.0} ms", state.latency.avg_ms), lat_col, Modifier::BOLD),
        ("JITTER", format!("{:.1} ms", state.latency.jitter_ms), c.text, Modifier::empty()),
        ("PKT LOSS", format!("{:.1}%", state.latency.packet_loss_pct), loss_col, Modifier::BOLD),
    ];

    let body_top = table.y + 2;
    let mut y = body_top;
    for (label, value, col, modifier) in rows {
        if y >= table.y + table.height.saturating_sub(2) { break; }
        px(buf, table.x + 1, y, label, c.text_faint, Modifier::empty());
        px(buf, divider_x + 2, y, &value, col, modifier);
        y += 1;
    }

    let quality_y = table.y + table.height.saturating_sub(2);
    if quality_y > body_top && quality_y < table.y + table.height {
        draw_hrule(buf, table.x, quality_y - 1, table.width, c.divider);
        let stars = format!(
            "{}{}",
            "★".repeat(score.round() as usize),
            "☆".repeat(5usize.saturating_sub(score.round() as usize))
        );
        px(buf, table.x + 1, quality_y, "QUALITY", c.text_faint, Modifier::BOLD);
        px(buf, divider_x + 2, quality_y, &stars, grade_col, Modifier::BOLD);
        px(
            buf,
            divider_x + 2 + stars.len() as u16 + 1,
            quality_y,
            &format!("Grade {}", grade),
            grade_col,
            Modifier::BOLD,
        );
    }
}

// ════════════════════════════════════════════════════════════════════════════
// HELP OVERLAY
// ════════════════════════════════════════════════════════════════════════════
fn render_help(buf: &mut Buffer, area: Rect, c: &ThemeColors) {
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_bg(c.bg);
        }
    }

    let pw = area.width.min(58);
    let ph = area.height.min(16);
    let pop = Rect {
        x: area.x + area.width.saturating_sub(pw) / 2,
        y: area.y + area.height.saturating_sub(ph) / 2,
        width: pw, height: ph,
    };

    draw_outer_border(buf, pop, c);
    let inner = shrink(pop, 1);

    px(buf, inner.x + 1, inner.y, " ⚡ Keyboard Shortcuts", c.text, Modifier::BOLD);

    let sc: &[(&str, &str)] = &[
        ("r",     "Restart test"),
        ("h",     "Open history browser"),
        ("e",     "Export result as JSON"),
        ("t",     "Toggle dark / light theme"),
        ("↑ ↓",   "Scroll history"),
        ("← →",   "Pan dashboard TEST HISTORY"),
        ("Home End", "Jump to oldest / newest graph"),
        ("?",     "Show this help"),
        ("q/Esc", "Quit / close overlay"),
    ];

    for (i, (key, desc)) in sc.iter().enumerate() {
        let y = inner.y + 2 + i as u16;
        if y >= inner.y + inner.height { break; }
        px(buf, inner.x + 2, y, &format!("[{}]", key), c.dl_primary, Modifier::BOLD);
        px(buf, inner.x + 12, y, desc, c.text, Modifier::empty());
    }

    px(buf, inner.x + 2, inner.y + inner.height.saturating_sub(2),
        "Press any key to close", c.text_faint, Modifier::empty());
}

// ════════════════════════════════════════════════════════════════════════════
// BLOCK-DIGIT SPEED GLYPHS
// ════════════════════════════════════════════════════════════════════════════
fn draw_block_speed(buf: &mut Buffer, area: Rect, value: f64, color: Color) {
    #[rustfmt::skip]
    const DIGITS: [[[bool; 3]; 5]; 10] = [
        [[true,true,true],[true,false,true],[true,false,true],[true,false,true],[true,true,true]],
        [[false,true,false],[false,true,false],[false,true,false],[false,true,false],[false,true,false]],
        [[true,true,true],[false,false,true],[true,true,true],[true,false,false],[true,true,true]],
        [[true,true,true],[false,false,true],[true,true,true],[false,false,true],[true,true,true]],
        [[true,false,true],[true,false,true],[true,true,true],[false,false,true],[false,false,true]],
        [[true,true,true],[true,false,false],[true,true,true],[false,false,true],[true,true,true]],
        [[true,true,true],[true,false,false],[true,true,true],[true,false,true],[true,true,true]],
        [[true,true,true],[false,false,true],[false,false,true],[false,false,true],[false,false,true]],
        [[true,true,true],[true,false,true],[true,true,true],[true,false,true],[true,true,true]],
        [[true,true,true],[true,false,true],[true,true,true],[false,false,true],[true,true,true]],
    ];
    const DOT: [[bool; 1]; 5] = [[false],[false],[false],[false],[true]];

    struct Seg { w: u16, rows: Vec<Vec<bool>> }
    let mut segs: Vec<Seg> = Vec::new();
    for ch in format!("{:.2}", value).chars() {
        if let Some(d) = ch.to_digit(10) {
            segs.push(Seg { w: 3, rows: DIGITS[d as usize].iter().map(|r| r.to_vec()).collect() });
        } else if ch == '.' {
            segs.push(Seg { w: 1, rows: DOT.iter().map(|r| r.to_vec()).collect() });
        }
        segs.push(Seg { w: 1, rows: vec![vec![false]; 5] });
    }
    if !segs.is_empty() { segs.pop(); }

    let total_w: u16 = segs.iter().map(|s| s.w).sum();
    let suffix_w = 6u16;

    if area.height >= 5 && total_w + suffix_w <= area.width {
        let y0  = area.y + area.height.saturating_sub(5) / 2;
        let x0  = area.x + area.width.saturating_sub(total_w + suffix_w) / 2;
        let mut cur_x = x0;

        for seg in &segs {
            for (row, cells) in seg.rows.iter().enumerate() {
                let y = y0 + row as u16;
                if y >= area.y + area.height { break; }
                for (col, &filled) in cells.iter().enumerate() {
                    let x = cur_x + col as u16;
                    if x >= area.x + area.width { break; }
                    if filled { buf[(x, y)].set_char('█').set_fg(color); }
                    else      { buf[(x, y)].set_char(' '); }
                }
            }
            cur_x += seg.w;
        }

        let sy = y0 + 4;
        for (i, ch) in " Mbps".chars().enumerate() {
            let x = cur_x + 1 + i as u16;
            if x >= area.x + area.width { break; }
            buf[(x, sy)].set_char(ch).set_fg(color)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
    } else {
        let txt = format!("{:.2} Mbps", value);
        let y   = area.y + area.height / 2;
        let x0  = area.x + area.width.saturating_sub(txt.len() as u16) / 2;
        for (i, ch) in txt.chars().enumerate() {
            let x = x0 + i as u16;
            if x >= area.x + area.width { break; }
            buf[(x, y)].set_char(ch).set_fg(color)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// LOW-LEVEL HELPERS
// ════════════════════════════════════════════════════════════════════════════
fn px(buf: &mut Buffer, x: u16, y: u16, s: &str, col: Color, m: Modifier) {
    for (i, ch) in s.chars().enumerate() {
        buf[(x + i as u16, y)].set_char(ch)
            .set_fg(col)
            .set_style(Style::default().add_modifier(m));
    }
}

fn pxc(buf: &mut Buffer, ax: u16, y: u16, aw: u16, s: &str, col: Color, m: Modifier) {
    let x0 = ax + aw.saturating_sub(s.len() as u16) / 2;
    for (i, ch) in s.chars().enumerate() {
        let x = x0 + i as u16;
        if x >= ax + aw { break; }
        buf[(x, y)].set_char(ch).set_fg(col).set_style(Style::default().add_modifier(m));
    }
}

fn pxr(buf: &mut Buffer, ax: u16, y: u16, aw: u16, s: &str, col: Color, m: Modifier) {
    let x0 = ax + aw.saturating_sub(s.len() as u16 + 1);
    for (i, ch) in s.chars().enumerate() {
        let x = x0 + i as u16;
        if x >= ax + aw { break; }
        buf[(x, y)].set_char(ch).set_fg(col).set_style(Style::default().add_modifier(m));
    }
}

fn fill_rect(buf: &mut Buffer, area: Rect, color: Color) {
    for y in area.y..area.y + area.height {
        for x in area.x..area.x + area.width {
            buf[(x, y)].set_bg(color);
        }
    }
}

fn draw_soft_box(buf: &mut Buffer, area: Rect, c: &ThemeColors) {
    if area.width < 2 || area.height < 2 { return; }
    let style = Style::default().fg(c.divider);

    buf[(area.x, area.y)].set_char('╭').set_style(style);
    buf[(area.x + area.width - 1, area.y)].set_char('╮').set_style(style);
    buf[(area.x, area.y + area.height - 1)].set_char('╰').set_style(style);
    buf[(area.x + area.width - 1, area.y + area.height - 1)].set_char('╯').set_style(style);

    for x in area.x + 1..area.x + area.width - 1 {
        buf[(x, area.y)].set_char('─').set_style(style);
        buf[(x, area.y + area.height - 1)].set_char('─').set_style(style);
    }
    for y in area.y + 1..area.y + area.height - 1 {
        buf[(area.x, y)].set_char('│').set_style(style);
        buf[(area.x + area.width - 1, y)].set_char('│').set_style(style);
    }
}

fn draw_hrule(buf: &mut Buffer, x: u16, y: u16, width: u16, color: Color) {
    for dx in 0..width {
        buf[(x + dx, y)].set_char('─').set_fg(color);
    }
}

fn draw_vrule(buf: &mut Buffer, x: u16, y: u16, height: u16, color: Color) {
    for dy in 0..height {
        buf[(x, y + dy)].set_char('│').set_fg(color);
    }
}
