use crate::model::Finding;
use anyhow::Result;
use std::path::Path;

pub async fn write(findings: &[Finding], output_dir: &Path) -> Result<()> {
    let path = output_dir.join("report.json");
    let json = serde_json::to_string_pretty(findings)?;
    tokio::fs::write(&path, json).await?;
    eprintln!("[sast] JSON report saved to {}", path.display());
    Ok(())
}
