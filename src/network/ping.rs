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
    let url = format!("https://{}/favicon.ico", host);
    let mut successes = 0;

    for _ in 0..count {
        let start = Instant::now();
        let result = client
            .head(&url)
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

    let _packet_loss = ((count - successes) as f64 / count as f64) * 100.0;
    let _ = tx.send(NetworkMsg::PingComplete).await;
    Ok(())
}
