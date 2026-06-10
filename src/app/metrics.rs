use crate::app::state::LatencyStats;

const ROLLING_WINDOW: usize = 60;

pub fn update_latency_stats(stats: &mut LatencyStats, sample_ms: f64) {
    stats.samples.push(sample_ms);
    let n = stats.samples.len() as f64;
    let sum: f64 = stats.samples.iter().sum();
    stats.avg_ms = sum / n;
    stats.min_ms = stats.samples.iter().cloned().fold(f64::INFINITY, f64::min);
    stats.max_ms = stats.samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    if stats.samples.len() > 1 {
        let mean = stats.avg_ms;
        let deviations: f64 = stats.samples.iter().map(|s| (s - mean).abs()).sum();
        stats.jitter_ms = deviations / (n - 1.0);
    }
}

pub fn update_speed_history(history: &mut Vec<f64>, sample: f64) {
    history.push(sample);
    if history.len() > ROLLING_WINDOW {
        history.remove(0);
    }
}
