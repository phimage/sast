use crate::model::{Finding, Severity};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub async fn write(findings: &[Finding], output_dir: &Path) -> Result<()> {
    // Group findings by tool
    let mut by_tool: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for f in findings {
        by_tool.entry(f.tool.as_str()).or_default().push(f);
    }

    let mut runs = Vec::new();
    for (tool_name, tool_findings) in &by_tool {
        let results: Vec<serde_json::Value> = tool_findings
            .iter()
            .map(|f| {
                let level = match f.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info | Severity::Style => "note",
                    Severity::Note => "note",
                };

                let mut location = serde_json::json!({
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": f.location.file.to_string_lossy()
                        }
                    }
                });

                if let Some(line) = f.location.line {
                    location["physicalLocation"]["region"] =
                        serde_json::json!({ "startLine": line });
                    if let Some(col) = f.location.column {
                        location["physicalLocation"]["region"]["startColumn"] = col.into();
                    }
                }

                let mut result = serde_json::json!({
                    "level": level,
                    "message": { "text": f.message },
                    "locations": [location]
                });

                if let Some(rule_id) = &f.rule_id {
                    result["ruleId"] = serde_json::Value::String(rule_id.clone());
                }

                result
            })
            .collect();

        runs.push(serde_json::json!({
            "tool": {
                "driver": {
                    "name": tool_name,
                    "informationUri": format!("https://github.com/search?q={}", tool_name)
                }
            },
            "results": results
        }));
    }

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": runs
    });

    let path = output_dir.join("report.sarif");
    let json = serde_json::to_string_pretty(&sarif)?;
    tokio::fs::write(&path, json).await?;
    eprintln!("[sast] SARIF report saved to {}", path.display());
    Ok(())
}
