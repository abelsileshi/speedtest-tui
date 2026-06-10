use anyhow::Result;
use reqwest::Client;
use std::time::Instant;
use tokio::sync::mpsc;
use crate::app::events::NetworkMsg;

pub async fn run_ping_test(
    client: Client,
    host: String,
    count: usize,
    tx: mpsc::Sender<NetworkMsg>,
) -> Result<()> {
    let url = format!("https://{}/cdn-cgi/trace", host);
    let mut successes = 0;

    for _ in 0..count {
        let start = Instant::now();
        let result = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        if result.is_ok() {
            successes += 1;
            let _ = tx.send(NetworkMsg::PingSample(elapsed)).await;
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    let packet_loss_pct = if count == 0 {
        0.0
    } else {
        ((count - successes) as f64 / count as f64) * 100.0
    };
    let _ = tx.send(NetworkMsg::PingComplete { packet_loss_pct }).await;
    Ok(())
}
