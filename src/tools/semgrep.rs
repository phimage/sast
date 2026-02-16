use crate::model::{Finding, Location, Severity};
use crate::runner::ToolRun;
use anyhow::Result;
use std::path::PathBuf;

pub fn parse(run: &ToolRun) -> Result<Vec<Finding>> {
    let text = String::from_utf8_lossy(&run.stdout);
    let json: serde_json::Value = serde_json::from_str(&text)?;

    let mut findings = Vec::new();

    if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
        for result in results {
            let path = result
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("unknown");
            let line = result
                .pointer("/start/line")
                .and_then(|l| l.as_u64())
                .map(|l| l as u32);
            let column = result
                .pointer("/start/col")
                .and_then(|c| c.as_u64())
                .map(|c| c as u32);
            let message = result
                .pointer("/extra/message")
                .and_then(|m| m.as_str())
                .unwrap_or("No message");
            let check_id = result
                .get("check_id")
                .and_then(|c| c.as_str())
                .map(String::from);
            let severity_str = result
                .pointer("/extra/severity")
                .and_then(|s| s.as_str())
                .unwrap_or("warning");

            let severity = match severity_str {
                "ERROR" => Severity::Error,
                "WARNING" => Severity::Warning,
                "INFO" => Severity::Info,
                _ => Severity::Warning,
            };

            findings.push(Finding {
                tool: "semgrep".to_string(),
                rule_id: check_id,
                severity,
                message: message.to_string(),
                location: Location {
                    file: PathBuf::from(path),
                    line,
                    column,
                },
            });
        }
    }

    Ok(findings)
}
