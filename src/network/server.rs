use crate::app::state::ServerInfo;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IpApiResponse {
    #[serde(default)]
    query: String,
    #[serde(default)]
    isp: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    status: String,
}

pub async fn fetch_ip_info(client: &Client) -> Result<(String, String, String)> {
    let resp = client
        .get("http://ip-api.com/json/?fields=status,country,regionName,city,isp,query")
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await?
        .json::<IpApiResponse>()
        .await?;

    if resp.status != "success" {
        anyhow::bail!("ip-api returned non-success");
    }

    let ip       = if resp.query.is_empty()       { "Unknown".into() } else { resp.query };
    let isp      = if resp.isp.is_empty()          { "Unknown ISP".into() } else { resp.isp };
    let location = format!("{}, {}", resp.city, resp.country);

    Ok((ip, isp, location))
}

pub fn get_server_list() -> Vec<ServerInfo> {
    vec![
        ServerInfo {
            id: "1".into(),
            name: "Cloudflare".into(),
            host: "speed.cloudflare.com".into(),
            location: "Global Anycast".into(),
            latency_ms: 0.0,
        },
        ServerInfo {
            id: "2".into(),
            name: "Cloudflare EU".into(),
            host: "speed.cloudflare.com".into(),
            location: "Global".into(),
            latency_ms: 0.0,
        },
    ]
}

pub async fn measure_server_latency(client: &Client, host: &str) -> f64 {
    let url   = format!("https://{}/cdn-cgi/trace", host);
    let start = std::time::Instant::now();
    if client
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .is_ok()
    {
        start.elapsed().as_secs_f64() * 1000.0
    } else {
        9999.0
    }
}

pub async fn select_best_server(
    client:       &Client,
    servers:      &mut Vec<ServerInfo>,
    preferred_id: &str,
) -> usize {
    if !preferred_id.is_empty() {
        if let Some(idx) = servers.iter().position(|s| s.id == preferred_id) {
            return idx;
        }
    }
    for server in servers.iter_mut() {
        server.latency_ms = measure_server_latency(client, &server.host).await;
    }
    servers
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.latency_ms.partial_cmp(&b.latency_ms).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0)
}
