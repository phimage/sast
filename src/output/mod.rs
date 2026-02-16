pub mod html;
pub mod json;
pub mod native;
pub mod sarif;

use crate::config::ToolConfig;
use crate::model::Finding;
use crate::runner::ToolRun;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Native,
    Sarif,
    Html,
    Json,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "native" => Ok(Self::Native),
            "sarif" => Ok(Self::Sarif),
            "html" => Ok(Self::Html),
            "json" => Ok(Self::Json),
            _ => anyhow::bail!("Unknown format '{}'. Use: native, sarif, html, json", s),
        }
    }
}

pub async fn write_output(
    format: OutputFormat,
    runs: &[ToolRun],
    findings: &[Finding],
    output_dir: &Path,
    project_name: &str,
    tool_configs: &HashMap<String, ToolConfig>,
) -> Result<()> {
    match format {
        OutputFormat::Native => native::write(runs, output_dir, tool_configs).await,
        OutputFormat::Sarif => sarif::write(findings, output_dir).await,
        OutputFormat::Html => html::write(findings, output_dir, project_name).await,
        OutputFormat::Json => json::write(findings, output_dir).await,
    }
}
