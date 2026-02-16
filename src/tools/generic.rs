use crate::model::{Finding, Location, Severity};
use crate::runner::ToolRun;
use anyhow::Result;
use std::path::PathBuf;

pub fn parse(run: &ToolRun) -> Result<Vec<Finding>> {
    let text = if !run.stdout.is_empty() {
        String::from_utf8_lossy(&run.stdout)
    } else {
        String::from_utf8_lossy(&run.stderr)
    };

    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Return the entire output as a single finding
    Ok(vec![Finding {
        tool: run.tool_name.clone(),
        rule_id: None,
        severity: Severity::Info,
        message: text.trim().to_string(),
        location: Location {
            file: PathBuf::from("."),
            line: None,
            column: None,
        },
    }])
}
