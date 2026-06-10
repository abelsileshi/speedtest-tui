mod app;
mod cli;
mod config;
mod network;
mod storage;
mod ui;

use anyhow::Result;
use app::{
    events::NetworkMsg,
    state::{AppState, Phase, Theme, WorkerState, DASHBOARD_HISTORY_STEP, DASHBOARD_HISTORY_WINDOW},
};
use chrono::Utc;
use clap::Parser;
use cli::{Cli, ThemeArg};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::{Duration, Instant}};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli    = Cli::parse();
    let config = config::Config::load().unwrap_or_default();

    if cli.quiet {
        return run_quiet(&cli, &config).await;
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend      = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let result = run_app(&mut terminal, &cli, &config).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cli:      &Cli,
    config:   &config::Config,
) -> Result<()> {
    let mut current_theme = match &cli.theme {
        Some(ThemeArg::Dark)  => Theme::Dark,
        Some(ThemeArg::Light) => Theme::Light,
        None => match config.theme.as_str() {
            "light" => Theme::Light,
            _       => Theme::Dark,
        },
    };

    loop {
        let mut state      = AppState::default();
        state.skip_upload  = cli.no_upload;
        state.theme        = current_theme.clone();
        state.history      = storage::history::load_history();
        state.phase        = Phase::ConnectivityCheck;

        let action = run_single_test(terminal, &mut state, cli, config).await?;

        current_theme = state.theme.clone();

        match action {
            Action::Quit    => break,
            Action::Restart => continue,
        }
    }
    Ok(())
}

#[derive(PartialEq)]
enum Action { Quit, Restart }

async fn run_single_test(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state:    &mut AppState,
    cli:      &Cli,
    config:   &config::Config,
) -> Result<Action> {
    let (tx, mut rx) = mpsc::channel::<NetworkMsg>(512);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let preferred   = cli.server.clone().unwrap_or_else(|| config.preferred_server.clone());
    let num_workers = config.parallel_workers;
    let duration    = config.test_duration_secs;
    let ping_count  = config.ping_count;
    let skip_upload = cli.no_upload;

    let net_client = client.clone();
    let net_tx     = tx.clone();
    tokio::spawn(async move {
        match network::server::fetch_ip_info(&net_client).await {
            Ok((ip, isp, loc)) => {
                let _ = net_tx.send(NetworkMsg::IpInfoReceived {
                    ip, isp, location: loc,
                }).await;
            }
            Err(_) => {
                let _ = net_tx.send(NetworkMsg::IpInfoReceived {
                    ip:       "Unavailable".into(),
                    isp:      "ISP lookup unavailable".into(),
                    location: "Unknown location".into(),
                }).await;
            }
        }

        let mut servers  = network::server::get_server_list();
        let best_idx     = network::server::select_best_server(
            &net_client, &mut servers, &preferred,
        ).await;
        let best_host    = servers.get(best_idx)
            .map(|s| s.host.clone())
            .unwrap_or_else(|| "speed.cloudflare.com".into());
        let _ = net_tx.send(NetworkMsg::ServersReceived(servers)).await;

        let _ = network::ping::run_ping_test(
            net_client.clone(), best_host.clone(), ping_count, net_tx.clone(),
        ).await;

        let _ = network::download::run_download_test(
            net_client.clone(), num_workers, duration, net_tx.clone(),
        ).await;

        if !skip_upload {
            let _ = network::upload::run_upload_test(
                net_client.clone(), num_workers, duration, net_tx.clone(),
            ).await;
        }
    });

    let tick      = Duration::from_millis(33); // ~30 fps
    let mut last  = Instant::now();

    loop {
        while let Ok(msg) = rx.try_recv() {
            handle_network_msg(state, msg);
        }

        terminal.draw(|f| ui::dashboard::render(f, state))?;

        let timeout = tick.saturating_sub(last.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if matches!(state.phase, Phase::History | Phase::Help) {
                                state.phase = if state.result.is_some() {
                                    Phase::Results
                                } else {
                                    Phase::Download
                                };
                            } else {
                                return Ok(Action::Quit);
                            }
                        }
                        KeyCode::Char('c')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            return Ok(Action::Quit);
                        }

                        KeyCode::Char('r') => {
                            return Ok(Action::Restart);
                        }
                        KeyCode::Char('t') => {
                            state.theme = match state.theme {
                                Theme::Dark  => Theme::Light,
                                Theme::Light => Theme::Dark,
                            };
                        }
                        KeyCode::Char('h') => {
                            state.phase = Phase::History;
                        }
                        KeyCode::Char('?') => {
                            state.phase = Phase::Help;
                        }
                        KeyCode::Char('e') => {
                            if let Some(ref r) = state.result.clone() {
                                let _ = storage::export::export_result(
                                    r, &cli::ExportFormat::Json, None,
                                );
                            }
                        }
                        KeyCode::Up => {
                            if matches!(state.phase, Phase::History) {
                                state.history_scroll =
                                    state.history_scroll.saturating_sub(1);
                            }
                        }
                        KeyCode::Down => {
                            if matches!(state.phase, Phase::History) {
                                state.history_scroll = (state.history_scroll + 1)
                                    .min(state.history.len().saturating_sub(1));
                            }
                        }
                        KeyCode::Left => {
                            if !matches!(state.phase, Phase::History | Phase::Help) {
                                let newest_start = state.history.len().saturating_sub(DASHBOARD_HISTORY_WINDOW);
                                let current_start = if state.history_graph_follow_newest {
                                    newest_start
                                } else {
                                    state.history_graph_start.min(newest_start)
                                };
                                state.history_graph_follow_newest = false;
                                state.history_graph_start =
                                    current_start.saturating_sub(DASHBOARD_HISTORY_STEP);
                            }
                        }
                        KeyCode::Right => {
                            if !matches!(state.phase, Phase::History | Phase::Help) {
                                let newest_start = state.history.len().saturating_sub(DASHBOARD_HISTORY_WINDOW);
                                let current_start = if state.history_graph_follow_newest {
                                    newest_start
                                } else {
                                    state.history_graph_start.min(newest_start)
                                };
                                let next_start = (current_start + DASHBOARD_HISTORY_STEP).min(newest_start);
                                state.history_graph_start = next_start;
                                state.history_graph_follow_newest = next_start >= newest_start;
                            }
                        }
                        KeyCode::Home => {
                            if !matches!(state.phase, Phase::History | Phase::Help) {
                                state.history_graph_start = 0;
                                state.history_graph_follow_newest = false;
                            }
                        }
                        KeyCode::End => {
                            if !matches!(state.phase, Phase::History | Phase::Help) {
                                state.history_graph_follow_newest = true;
                            }
                        }
                        _ => {
                            if matches!(state.phase, Phase::Help) {
                                state.phase = Phase::Results;
                            }
                        }
                    }
                }
            }
        }

        if last.elapsed() >= tick {
            state.animation_tick = state.animation_tick.wrapping_add(1);
            last = Instant::now();
        }
    }
}

fn handle_network_msg(state: &mut AppState, msg: NetworkMsg) {
    match msg {
        NetworkMsg::IpInfoReceived { ip, isp, location } => {
            state.ip_info.ip       = ip;
            state.ip_info.isp      = isp;
            state.ip_info.location = location;
            state.phase            = Phase::ServerSelection;
        }
        NetworkMsg::ServersReceived(servers) => {
            state.servers = servers;
            state.phase   = Phase::LatencyMeasurement;
        }
        NetworkMsg::PingSample(ms) => {
            app::metrics::update_latency_stats(&mut state.latency, ms);
        }
        NetworkMsg::PingComplete => {
            state.phase = Phase::Download;
            state.download.workers = std::iter::repeat_with(|| WorkerState {
                active: true,
                ..Default::default()
            })
                .take(8)
                .collect();
        }
        NetworkMsg::DownloadSample { worker_id, mbps, aggregate_mbps } => {
            state.download.current_mbps = aggregate_mbps;
            if aggregate_mbps > state.download.peak_mbps {
                state.download.peak_mbps = aggregate_mbps;
            }
            app::metrics::update_speed_history(&mut state.download.history, aggregate_mbps);
            if let Some(w) = state.download.workers.get_mut(worker_id) {
                w.speed_mbps = mbps;
                w.active = true;
            }
        }
        NetworkMsg::DownloadComplete(avg) => {
            state.download.avg_mbps = avg;
            for w in &mut state.download.workers {
                w.active = false; w.complete = true;
            }
            if !state.skip_upload {
                state.phase = Phase::Upload;
                state.upload.workers = std::iter::repeat_with(|| WorkerState {
                    active: true,
                    ..Default::default()
                })
                    .take(8)
                    .collect();
            } else {
                finalize_result(state);
            }
        }
        NetworkMsg::UploadSample { worker_id, mbps, aggregate_mbps } => {
            state.upload.current_mbps = aggregate_mbps;
            if aggregate_mbps > state.upload.peak_mbps {
                state.upload.peak_mbps = aggregate_mbps;
            }
            app::metrics::update_speed_history(&mut state.upload.history, aggregate_mbps);
            if let Some(w) = state.upload.workers.get_mut(worker_id) {
                w.speed_mbps = mbps;
                w.active = true;
            }
        }
        NetworkMsg::UploadComplete(avg) => {
            state.upload.avg_mbps = avg;
            for w in &mut state.upload.workers {
                w.active = false; w.complete = true;
            }
            finalize_result(state);
        }
    }
}

fn finalize_result(state: &mut AppState) {
    let (score, grade) = state.compute_quality_score();
    let server = state.servers.get(state.selected_server_idx);
    let result = app::state::TestResult {
        timestamp:       Some(Utc::now()),
        isp:             state.ip_info.isp.clone(),
        ip:              state.ip_info.ip.clone(),
        location:        state.ip_info.location.clone(),
        server_name:     server.map(|s| s.name.clone()).unwrap_or_default(),
        server_host:     server.map(|s| s.host.clone()).unwrap_or_default(),
        ping_ms:         state.latency.avg_ms,
        jitter_ms:       state.latency.jitter_ms,
        packet_loss_pct: state.latency.packet_loss_pct,
        download_mbps:   state.download.avg_mbps,
        upload_mbps:     state.upload.avg_mbps,
        quality_score:   score,
        quality_grade:   grade,
    };
    let _ = storage::history::append_result(&result);
    state.history.push(result.clone());
    state.history_graph_follow_newest = true;
    state.result = Some(result);
    state.phase  = Phase::Results;
}

async fn run_quiet(cli: &Cli, config: &config::Config) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let (ip, isp, location) = network::server::fetch_ip_info(&client)
        .await
        .unwrap_or_else(|_| ("Unknown".into(), "Unknown".into(), "Unknown".into()));

    let mut servers = network::server::get_server_list();
    let best_idx    = network::server::select_best_server(
        &client, &mut servers, &config.preferred_server,
    ).await;
    let server = servers.get(best_idx).cloned().unwrap_or_default();

    let (tx, mut rx)   = mpsc::channel::<NetworkMsg>(256);
    let (dl_c, ul_c)   = (client.clone(), client.clone());
    let (dl_tx, ul_tx) = (tx.clone(), tx.clone());
    let (dur, wrk)     = (config.test_duration_secs, config.parallel_workers);

    tokio::spawn(async move {
        let _ = network::download::run_download_test(dl_c, wrk, dur, dl_tx).await;
    });
    tokio::spawn(async move {
        let _ = network::upload::run_upload_test(ul_c, wrk, dur, ul_tx).await;
    });

    let (mut dl_mbps, mut ul_mbps) = (0.0f64, 0.0f64);
    while let Some(msg) = rx.recv().await {
        match msg {
            NetworkMsg::DownloadComplete(v) => dl_mbps = v,
            NetworkMsg::UploadComplete(v)   => { ul_mbps = v; break; }
            _ => {}
        }
    }

    let result = app::state::TestResult {
        timestamp:       Some(Utc::now()),
        isp, ip, location,
        server_name:     server.name,
        server_host:     server.host,
        ping_ms:         0.0,
        jitter_ms:       0.0,
        packet_loss_pct: 0.0,
        download_mbps:   dl_mbps,
        upload_mbps:     ul_mbps,
        quality_score:   0.0,
        quality_grade:   "N/A".into(),
    };

    println!("{}", serde_json::to_string_pretty(&result)?);

    if let Some(ref fmt) = cli.export {
        storage::export::export_result(&result, fmt, None)?;
    }

    Ok(())
}
