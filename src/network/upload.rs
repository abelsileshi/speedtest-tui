use anyhow::Result;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::app::events::NetworkMsg;

const UPLOAD_SIZE: usize = 4_194_304; // 4MB per request

pub async fn run_upload_test(
    client: Client,
    num_workers: usize,
    duration_secs: u64,
    tx: mpsc::Sender<NetworkMsg>,
) -> Result<f64> {
    let start = Arc::new(Instant::now());
    let total_bytes = Arc::new(Mutex::new(0u64));
    let deadline = Duration::from_secs(duration_secs);
    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let client = client.clone();
        let tx = tx.clone();
        let total_bytes = Arc::clone(&total_bytes);
        let start = Arc::clone(&start);

        let handle = tokio::spawn(async move {
            let worker_bytes = Arc::new(Mutex::new(0u64));
            while start.elapsed() < deadline {
                let payload = vec![0u8; UPLOAD_SIZE];
                if client
                    .post("https://speed.cloudflare.com/__up")
                    .body(payload)
                    .timeout(Duration::from_secs(15))
                    .send()
                    .await
                    .is_ok()
                {
                    let len = UPLOAD_SIZE as u64;
                    { *total_bytes.lock().unwrap() += len; }
                    { *worker_bytes.lock().unwrap() += len; }

                    let elapsed = start.elapsed().as_secs_f64();
                    let total = *total_bytes.lock().unwrap();
                    let worker_total = *worker_bytes.lock().unwrap();
                    let agg = if elapsed > 0.0 { (total as f64 * 8.0) / (elapsed * 1_000_000.0) } else { 0.0 };
                    let wmbps = if elapsed > 0.0 { (worker_total as f64 * 8.0) / (elapsed * 1_000_000.0) } else { 0.0 };

                    let _ = tx.send(NetworkMsg::UploadSample {
                        worker_id,
                        bytes: worker_total,
                        mbps: wmbps,
                        aggregate_mbps: agg,
                    }).await;
                }
            }
        });
        handles.push(handle);
    }

    for h in handles { let _ = h.await; }

    let elapsed = start.elapsed().as_secs_f64();
    let total = *total_bytes.lock().unwrap();
    let final_mbps = if elapsed > 0.0 { (total as f64 * 8.0) / (elapsed * 1_000_000.0) } else { 0.0 };
    let _ = tx.send(NetworkMsg::UploadComplete(final_mbps)).await;
    Ok(final_mbps)
}
