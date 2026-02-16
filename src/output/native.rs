use crate::config::{OutputStream, ToolConfig};
use crate::runner::ToolRun;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub async fn write(
    runs: &[ToolRun],
    output_dir: &Path,
    tool_configs: &HashMap<String, ToolConfig>,
) -> Result<()> {
    for run in runs {
        let ext = tool_configs
            .get(&run.tool_name)
            .and_then(|c| c.native_extension.as_deref())
            .unwrap_or("txt");

        let config = tool_configs.get(&run.tool_name);
        let is_filesystem = config
            .map(|c| matches!(c.output_stream, OutputStream::Filesystem))
            .unwrap_or(false);

        if is_filesystem {
            // Filesystem tools (like scan-build) write their own output to output_dir
            // Nothing extra to save here
            eprintln!(
                "[sast] {} output written to {}/",
                run.tool_name,
                output_dir.display()
            );
            continue;
        }

        let filename = format!("{}.{}", run.tool_name, ext);
        let path = output_dir.join(&filename);

        let data = match config.map(|c| &c.output_stream) {
            Some(OutputStream::Stderr) => {
                // Some tools (e.g. cppcheck) may write to stdout on certain platforms
                if run.stderr.is_empty() {
                    &run.stdout
                } else {
                    &run.stderr
                }
            }
            _ => &run.stdout,
        };

        tokio::fs::write(&path, data).await?;
        eprintln!(
            "[sast] {} output saved to {}",
            run.tool_name,
            path.display()
        );
    }

    Ok(())
}
