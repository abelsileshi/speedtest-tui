use crate::app::state::ServerInfo;
use anyhow::Result;
use reqwest::{header::RETRY_AFTER, Client, StatusCode};
use serde::Deserialize;
use std::{
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};

const IPWHOIS_URL: &str = "https://ipwho.is/";
const IP_LOOKUP_TIMEOUT: Duration = Duration::from_secs(8);
const RATE_LIMIT_FALLBACK: Duration = Duration::from_secs(300);

#[derive(Debug, Default, Deserialize)]
struct IpWhoIsConnection {
    #[serde(default)]
    isp: String,
}

#[derive(Debug, Default, Deserialize)]
struct IpWhoIsResponse {
    #[serde(default)]
    ip: String,
    #[serde(default)]
    success: bool,
    #[serde(default)]
    message: String,
    #[serde(default)]
    city: String,
    #[serde(default)]
    country: String,
    #[serde(default)]
    connection: IpWhoIsConnection,
}

fn ip_lookup_cooldown() -> &'static Mutex<Option<Instant>> {
    static COOLDOWN: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    COOLDOWN.get_or_init(|| Mutex::new(None))
}

fn parse_retry_after_seconds(value: Option<&reqwest::header::HeaderValue>) -> Option<u64> {
    value
        .and_then(|header| header.to_str().ok())
        .and_then(|text| text.trim().parse::<u64>().ok())
}

fn format_location(city: &str, country: &str) -> String {
    match (city.trim(), country.trim()) {
        ("", "") => "Unknown location".into(),
        ("", country) => country.to_string(),
        (city, "") => city.to_string(),
        (city, country) => format!("{}, {}", city, country),
    }
}

pub async fn fetch_ip_info(client: &Client) -> Result<(String, String, String)> {
    {
        let now = Instant::now();
        let mut cooldown = ip_lookup_cooldown()
            .lock()
            .expect("ip lookup cooldown mutex poisoned");
        if let Some(until) = *cooldown {
            if now < until {
                let wait = until.saturating_duration_since(now).as_secs().max(1);
                anyhow::bail!("ipwho.is cooldown active for {} more seconds", wait);
            }
            *cooldown = None;
        }
    }

    let response = client
        .get(IPWHOIS_URL)
        .timeout(IP_LOOKUP_TIMEOUT)
        .send()
        .await?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = parse_retry_after_seconds(response.headers().get(RETRY_AFTER))
            .map(Duration::from_secs)
            .unwrap_or(RATE_LIMIT_FALLBACK);
        let wait_secs = retry_after.as_secs().max(1);
        let mut cooldown = ip_lookup_cooldown()
            .lock()
            .expect("ip lookup cooldown mutex poisoned");
        *cooldown = Some(Instant::now() + retry_after);
        anyhow::bail!("ipwho.is rate limit reached; retry after {} seconds", wait_secs);
    }

    let response = response.error_for_status()?;
    let resp = response.json::<IpWhoIsResponse>().await?;

    if !resp.success {
        let message = if resp.message.trim().is_empty() {
            "ipwho.is returned an unsuccessful response"
        } else {
            resp.message.trim()
        };
        anyhow::bail!(message.to_string());
    }

    let ip = if resp.ip.trim().is_empty() {
        "Unknown".into()
    } else {
        resp.ip
    };
    let isp = if resp.connection.isp.trim().is_empty() {
        "Unknown ISP".into()
    } else {
        resp.connection.isp
    };
    let location = format_location(&resp.city, &resp.country);

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
