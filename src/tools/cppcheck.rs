use crate::model::{Finding, Location, Severity};
use crate::runner::ToolRun;
use anyhow::Result;
use regex::Regex;
use std::path::PathBuf;

pub fn parse(run: &ToolRun) -> Result<Vec<Finding>> {
    // cppcheck typically writes to stderr, but some versions/platforms (e.g. Windows)
    // may use stdout. Parse both and use whichever yields findings.
    let stderr_findings = parse_text(&String::from_utf8_lossy(&run.stderr))?;
    if !stderr_findings.is_empty() {
        return Ok(stderr_findings);
    }
    parse_text(&String::from_utf8_lossy(&run.stdout))
}

fn parse_text(text: &str) -> Result<Vec<Finding>> {
    // cppcheck format: /path/file.cpp:12:5: severity: message [ruleId]
    let re = Regex::new(r"^(.+?):(\d+):\d+:\s*(\w+):\s*(.+?)(?:\s*\[(\w+)\])?\s*$")?;

    let mut findings = Vec::new();
    for line in text.lines() {
        if let Some(cap) = re.captures(line) {
            let file = cap[1].to_string();
            let line_num: u32 = cap[2].parse().unwrap_or(0);
            let severity_str = &cap[3];
            let message = cap[4].to_string();
            let rule_id = cap.get(5).map(|m| m.as_str().to_string());

            let severity = match severity_str {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                "style" => Severity::Style,
                "information" => Severity::Info,
                "performance" => Severity::Warning,
                "portability" => Severity::Warning,
                "note" => Severity::Note,
                _ => Severity::Note,
            };

            findings.push(Finding {
                tool: "cppcheck".to_string(),
                rule_id,
                severity,
                message,
                location: Location {
                    file: PathBuf::from(file),
                    line: Some(line_num),
                    column: None,
                },
            });
        }
    }

    Ok(findings)
}
