use crate::config::{OutputStream, ToolConfig};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct ToolRun {
    pub tool_name: String,
    #[allow(dead_code)]
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

fn interpolate_args(args: &[String], project_path: &Path, output_dir: &Path) -> Vec<String> {
    args.iter()
        .map(|arg| {
            arg.replace("{output_dir}", &output_dir.to_string_lossy())
                .replace("{project_path}", &project_path.to_string_lossy())
        })
        .collect()
}

async fn run_single_tool(
    name: &str,
    config: &ToolConfig,
    project_path: &Path,
    output_dir: &Path,
) -> Result<ToolRun> {
    let mut args = interpolate_args(&config.args, project_path, output_dir);

    // Append the project path for tools that take it as a positional arg
    match config.output_stream {
        OutputStream::Stdout | OutputStream::Stderr => {
            args.push(project_path.to_string_lossy().to_string());
        }
        OutputStream::Filesystem => {}
    }

    // If append_sources is set, glob matching files and append them
    if let Some(pattern) = &config.append_sources {
        let full_pattern = format!("{}/{}", project_path.display(), pattern);
        for path in glob::glob(&full_pattern)
            .unwrap_or_else(|_| glob::glob("").unwrap())
            .flatten()
        {
            args.push(path.to_string_lossy().to_string());
        }
    }

    eprintln!("[sast] Running {}...", name);
    tracing::debug!("Executing: {} {}", config.command, args.join(" "));

    // On Windows, use scan-build.bat for scan-build
    let command_name = if cfg!(windows) && config.command == "scan-build" {
        "scan-build.bat".to_string()
    } else {
        config.command.clone()
    };

    let output = tokio::process::Command::new(&command_name)
        .args(&args)
        .output()
        .await
        .with_context(|| format!("Failed to execute '{}'. Is it installed?", command_name))?;

    let exit_code = output.status.code().unwrap_or(-1);

    if exit_code != 0 {
        tracing::warn!(
            "{} exited with code {} (this may be normal for tools reporting findings)",
            name,
            exit_code
        );
    }

    Ok(ToolRun {
        tool_name: name.to_string(),
        exit_code,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub async fn run_tools(
    tools_to_run: &[String],
    tool_configs: &HashMap<String, ToolConfig>,
    project_path: &Path,
    output_dir: &Path,
) -> Vec<Result<ToolRun>> {
    let mut join_set = tokio::task::JoinSet::new();

    for name in tools_to_run {
        let config = tool_configs[name].clone();
        let name = name.clone();
        let project = project_path.to_owned();
        let out = output_dir.to_owned();

        join_set.spawn(async move { run_single_tool(&name, &config, &project, &out).await });
    }

    let mut results = Vec::new();
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(tool_result) => results.push(tool_result),
            Err(e) => results.push(Err(anyhow::anyhow!("Task join error: {e}"))),
        }
    }
    results
}
