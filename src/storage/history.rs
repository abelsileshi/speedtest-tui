use anyhow::Result;
use crate::app::state::TestResult;
use crate::config::history_path;

pub fn load_history() -> Vec<TestResult> {
    history_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn append_result(result: &TestResult) -> Result<()> {
    let mut history = load_history();
    history.push(result.clone());
    if history.len() > 1000 {
        history = history[history.len() - 1000..].to_vec();
    }
    if let Some(path) = history_path() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(&history)?)?;
    }
    Ok(())
}
