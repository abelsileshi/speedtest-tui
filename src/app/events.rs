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
        mbps: f64,
        aggregate_mbps: f64,
    },
    DownloadComplete(f64),
    UploadSample {
        worker_id: usize,
        mbps: f64,
        aggregate_mbps: f64,
    },
    UploadComplete(f64),
}
