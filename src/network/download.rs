use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::app::events::NetworkMsg;

const CHUNK_SIZES: &[usize] = &[262_144, 524_288, 1_048_576, 4_194_304]; // 256KB..4MB
const TEST_URL: &str = "https://speed.cloudflare.com/__down?bytes=";

pub async fn run_download_test(
    client: Client,
    num_workers: usize,
    duration_secs: u64,
    tx: mpsc::Sender<NetworkMsg>,
) -> Result<f64> {
    let start = Arc::new(Instant::now());
    let total_bytes = Arc::new(Mutex::new(0u64));
    let worker_bytes: Vec<Arc<Mutex<u64>>> = (0..num_workers)
        .map(|_| Arc::new(Mutex::new(0u64)))
        .collect();

    let mut handles = Vec::new();
    let deadline = Duration::from_secs(duration_secs);

    for worker_id in 0..num_workers {
        let client = client.clone();
        let tx = tx.clone();
        let total_bytes = Arc::clone(&total_bytes);
        let worker_bytes = Arc::clone(&worker_bytes[worker_id]);
        let start = Arc::clone(&start);

        let handle = tokio::spawn(async move {
            let _ = tx.send(NetworkMsg::DownloadSample {
                worker_id,
                bytes: 0,
                mbps: 0.0,
                aggregate_mbps: 0.0,
            }).await;

            let mut chunk_idx = 0;
            while start.elapsed() < deadline {
                let chunk_size = CHUNK_SIZES[chunk_idx.min(CHUNK_SIZES.len() - 1)];
                let url = format!("{}{}", TEST_URL, chunk_size);

                if let Ok(resp) = client
                    .get(&url)
                    .timeout(Duration::from_secs(15))
                    .send()
                    .await
                {
                    let mut stream = resp.bytes_stream();
                    while let Some(Ok(chunk)) = stream.next().await {
                        if start.elapsed() >= deadline {
                            break;
                        }
                        let len = chunk.len() as u64;
                        {
                            let mut wb = worker_bytes.lock().unwrap();
                            *wb += len;
                        }
                        {
                            let mut tb = total_bytes.lock().unwrap();
                            *tb += len;
                        }

                        let elapsed = start.elapsed().as_secs_f64();
                        let total = *total_bytes.lock().unwrap();
                        let worker_total = *worker_bytes.lock().unwrap();
                        let aggregate_mbps = if elapsed > 0.0 {
                            (total as f64 * 8.0) / (elapsed * 1_000_000.0)
                        } else {
                            0.0
                        };
                        let worker_mbps = if elapsed > 0.0 {
                            (worker_total as f64 * 8.0) / (elapsed * 1_000_000.0)
                        } else {
                            0.0
                        };

                        let _ = tx.send(NetworkMsg::DownloadSample {
                            worker_id,
                            bytes: worker_total,
                            mbps: worker_mbps,
                            aggregate_mbps,
                        }).await;
                    }
                }

                if chunk_idx < CHUNK_SIZES.len() - 1 {
                    chunk_idx += 1;
                }
            }
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed().as_secs_f64();
    let total = *total_bytes.lock().unwrap();
    let final_mbps = if elapsed > 0.0 {
        (total as f64 * 8.0) / (elapsed * 1_000_000.0)
    } else {
        0.0
    };

    let _ = tx.send(NetworkMsg::DownloadComplete(final_mbps)).await;
    Ok(final_mbps)
}
