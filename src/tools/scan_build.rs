use crate::model::{Finding, Location, Severity};
use crate::runner::ToolRun;
use anyhow::Result;
use regex::Regex;
use std::path::PathBuf;

pub fn parse(run: &ToolRun) -> Result<Vec<Finding>> {
    // scan-build outputs diagnostics to stderr with lines like:
    // /path/file.cpp:12:5: warning: ...
    let text = String::from_utf8_lossy(&run.stderr);
    let re = Regex::new(r"([^:\s]+\.\w+):(\d+):(\d+):\s*(warning|error|note):\s*(.+)")?;

    let mut findings = Vec::new();
    for cap in re.captures_iter(&text) {
        let file = cap[1].to_string();
        let line: u32 = cap[2].parse().unwrap_or(0);
        let column: u32 = cap[3].parse().unwrap_or(0);
        let severity = match &cap[4] {
            "error" => Severity::Error,
            "warning" => Severity::Warning,
            _ => Severity::Note,
        };
        let message = cap[5].to_string();

        findings.push(Finding {
            tool: "scan-build".to_string(),
            rule_id: None,
            severity,
            message,
            location: Location {
                file: PathBuf::from(file),
                line: Some(line),
                column: Some(column),
            },
        });
    }

    Ok(findings)
}
