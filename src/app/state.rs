use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    Init,
    ConnectivityCheck,
    ServerSelection,
    LatencyMeasurement,
    Download,
    Upload,
    Results,
    History,
    Help,
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct ServerInfo {
    pub id: String,
    pub name: String,
    pub host: String,
    pub location: String,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Default)]
pub struct LatencyStats {
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss_pct: f64,
    pub samples: Vec<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkerState {
    pub id: usize,
    pub speed_mbps: f64,
    pub bytes_transferred: u64,
    pub active: bool,
    pub complete: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SpeedStats {
    pub current_mbps: f64,
    pub peak_mbps: f64,
    pub avg_mbps: f64,
    pub history: Vec<f64>, // rolling 30s
    pub workers: Vec<WorkerState>,
    pub bytes_total: u64,
    pub elapsed_secs: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestResult {
    pub timestamp: Option<DateTime<Utc>>,
    pub isp: String,
    pub ip: String,
    pub location: String,
    pub server_name: String,
    pub server_host: String,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub packet_loss_pct: f64,
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub quality_score: f64,
    pub quality_grade: String,
}

#[derive(Debug, Clone, Default)]
pub struct IpInfo {
    pub ip: String,
    pub isp: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub phase: Phase,
    pub ip_info: IpInfo,
    pub servers: Vec<ServerInfo>,
    pub selected_server_idx: usize,
    pub latency: LatencyStats,
    pub download: SpeedStats,
    pub upload: SpeedStats,
    pub result: Option<TestResult>,
    pub history: Vec<TestResult>,
    pub history_scroll: usize,
    pub status_message: String,
    pub theme: Theme,
    pub skip_upload: bool,
    pub animation_tick: u64,
    pub ping_progress: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            phase: Phase::Init,
            ip_info: IpInfo::default(),
            servers: Vec::new(),
            selected_server_idx: 0,
            latency: LatencyStats::default(),
            download: SpeedStats::default(),
            upload: SpeedStats::default(),
            result: None,
            history: Vec::new(),
            history_scroll: 0,
            status_message: String::new(),
            theme: Theme::Dark,
            skip_upload: false,
            animation_tick: 0,
            ping_progress: 0.0,
        }
    }
}

impl AppState {
    pub fn compute_quality_score(&self) -> (f64, String) {
        let ping_score = if self.latency.avg_ms < 20.0 { 5.0 }
            else if self.latency.avg_ms < 50.0 { 4.0 }
            else if self.latency.avg_ms < 100.0 { 3.0 }
            else if self.latency.avg_ms < 150.0 { 2.0 }
            else { 1.0 };

        let dl_score = if self.download.avg_mbps > 200.0 { 5.0 }
            else if self.download.avg_mbps > 50.0 { 4.0 }
            else if self.download.avg_mbps > 10.0 { 3.0 }
            else if self.download.avg_mbps > 2.0 { 2.0 }
            else { 1.0 };

        let ul_score = if self.upload.avg_mbps > 50.0 { 5.0 }
            else if self.upload.avg_mbps > 10.0 { 4.0 }
            else if self.upload.avg_mbps > 3.0 { 3.0 }
            else if self.upload.avg_mbps > 1.0 { 2.0 }
            else { 1.0 };

        let loss_score = if self.latency.packet_loss_pct == 0.0 { 5.0 }
            else if self.latency.packet_loss_pct < 1.0 { 4.0 }
            else if self.latency.packet_loss_pct < 3.0 { 3.0 }
            else if self.latency.packet_loss_pct < 5.0 { 2.0 }
            else { 1.0 };

        let score = (ping_score + dl_score + ul_score + loss_score) / 4.0;
        let grade = match score as u32 {
            5 => "A",
            4 => "B",
            3 => "C",
            2 => "D",
            _ => "F",
        };
        (score, grade.to_string())
    }
}

impl Default for Phase {
    fn default() -> Self {
        Phase::Init
    }
}
