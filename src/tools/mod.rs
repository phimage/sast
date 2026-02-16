pub mod cppcheck;
pub mod generic;
pub mod scan_build;
pub mod semgrep;

use crate::model::Finding;
use crate::runner::ToolRun;
use anyhow::Result;

pub fn parse_output(tool_name: &str, run: &ToolRun) -> Result<Vec<Finding>> {
    match tool_name {
        "cppcheck" => cppcheck::parse(run),
        "semgrep" => semgrep::parse(run),
        "scan-build" => scan_build::parse(run),
        _ => generic::parse(run),
    }
}
