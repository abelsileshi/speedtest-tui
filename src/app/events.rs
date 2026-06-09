use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    NetworkUpdate(NetworkMsg),
    Quit,
}

#[derive(Debug, Clone)]
pub enum NetworkMsg {
    IpInfoReceived {
        ip: String,
        isp: String,
        location: String,
    },
    ServersReceived(Vec<crate::app::state::ServerInfo>),
    PingSample(f64),
    PingComplete,
    DownloadSample {
        worker_id: usize,
        bytes: u64,
        mbps: f64,
        aggregate_mbps: f64,
    },
    DownloadComplete(f64),
    UploadSample {
        worker_id: usize,
        bytes: u64,
        mbps: f64,
        aggregate_mbps: f64,
    },
    UploadComplete(f64),
    Error(String),
}

pub fn handle_key(key: KeyEvent, phase: &crate::app::state::Phase) -> Option<AppEvent> {
    use crate::app::state::Phase;
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if matches!(phase, Phase::History | Phase::Help) {
                None // handled by caller to go back
            } else {
                Some(AppEvent::Quit)
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppEvent::Quit)
        }
        _ => Some(AppEvent::Key(key)),
    }
}
