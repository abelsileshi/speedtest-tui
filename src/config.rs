use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub parallel_workers: usize,
    pub ping_count: usize,
    pub test_duration_secs: u64,
    pub preferred_server: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "auto".into(),
            parallel_workers: 8,
            ping_count: 100,
            test_duration_secs: 10,
            preferred_server: String::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        if let Some(path) = config_path() {
            if path.exists() {
                let text = std::fs::read_to_string(&path)?;
                let cfg: Config = toml::from_str(&text)?;
                return Ok(cfg);
            }
        }
        Ok(Config::default())
    }
}

pub fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "speedtest-tui")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}

pub fn history_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "speedtest-tui")
        .map(|dirs| dirs.data_local_dir().join("history.json"))
}
