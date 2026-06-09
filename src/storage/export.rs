use anyhow::Result;
use crate::app::state::TestResult;
use crate::cli::ExportFormat;

pub fn export_result(result: &TestResult, format: &ExportFormat, path: Option<&str>) -> Result<()> {
    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(result)?;
            let filename = path.unwrap_or("speedtest-result.json");
            std::fs::write(filename, json)?;
            eprintln!("Exported JSON to {}", filename);
        }
        ExportFormat::Csv => {
            let filename = path.unwrap_or("speedtest-results.csv");
            let file_exists = std::path::Path::new(filename).exists();
            let file = std::fs::OpenOptions::new()
                .append(true).create(true).open(filename)?;
            let mut wtr = csv::Writer::from_writer(file);
            if !file_exists {
                wtr.write_record(&[
                    "timestamp","isp","ip","location","server",
                    "ping_ms","jitter_ms","packet_loss_pct",
                    "download_mbps","upload_mbps","grade",
                ])?;
            }
            let ts = result.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default();
            wtr.write_record(&[
                &ts, &result.isp, &result.ip, &result.location, &result.server_name,
                &format!("{:.2}", result.ping_ms),
                &format!("{:.2}", result.jitter_ms),
                &format!("{:.2}", result.packet_loss_pct),
                &format!("{:.2}", result.download_mbps),
                &format!("{:.2}", result.upload_mbps),
                &result.quality_grade,
            ])?;
            wtr.flush()?;
            eprintln!("Exported CSV to {}", filename);
        }
        ExportFormat::Png => {
            eprintln!("PNG export: not yet implemented.");
        }
    }
    Ok(())
}
