mod cli;
mod config;
mod model;
mod output;
mod runner;
mod tools;

use anyhow::{ensure, Result};
use clap::Parser;
use output::OutputFormat;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = cli::Cli::parse();

    // Validate project path
    ensure!(
        cli.path.exists(),
        "Project path '{}' does not exist",
        cli.path.display()
    );

    // Load and merge config
    let config = config::load_config(cli.config.as_deref())?;

    // Resolve effective settings (CLI overrides config)
    let tools_to_run = cli
        .tools
        .or(config.defaults.tools.clone())
        .unwrap_or_else(|| vec!["cppcheck".into(), "scan-build".into(), "semgrep".into()]);

    let format_str = cli
        .format
        .or(config.defaults.format.clone())
        .unwrap_or_else(|| "native".to_string());
    let format = OutputFormat::from_str(&format_str)?;

    let output_dir = cli
        .output
        .or(config.defaults.output.clone())
        .unwrap_or_else(|| cli.path.join("sast_report"));

    // Validate requested tools exist in config
    for t in &tools_to_run {
        ensure!(
            config.tools.contains_key(t),
            "Unknown tool '{}'. Available: {:?}",
            t,
            config.tools.keys().collect::<Vec<_>>()
        );
    }

    // Create output directory
    tokio::fs::create_dir_all(&output_dir).await?;

    eprintln!("===== SAST Analysis =====");
    eprintln!("Project:  {}", cli.path.display());
    eprintln!("Tools:    {}", tools_to_run.join(", "));
    eprintln!("Format:   {}", format_str);
    eprintln!("Output:   {}", output_dir.display());
    eprintln!();

    // Run tools in parallel
    let results = runner::run_tools(&tools_to_run, &config.tools, &cli.path, &output_dir).await;

    // Collect successful runs
    let mut successful_runs = Vec::new();
    let mut failures = 0;
    for result in results {
        match result {
            Ok(run) => successful_runs.push(run),
            Err(e) => {
                eprintln!("[sast] Tool failed: {e}");
                failures += 1;
            }
        }
    }

    if successful_runs.is_empty() {
        anyhow::bail!("All tools failed to execute");
    }

    // Parse findings if needed (skip for native mode)
    let findings = if format != OutputFormat::Native {
        let mut all_findings = Vec::new();
        for run in &successful_runs {
            match tools::parse_output(&run.tool_name, run) {
                Ok(f) => all_findings.extend(f),
                Err(e) => {
                    tracing::warn!("Failed to parse {} output: {e}", run.tool_name);
                }
            }
        }
        all_findings
    } else {
        Vec::new()
    };

    // Write output
    let project_name = cli
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".to_string());

    output::write_output(
        format,
        &successful_runs,
        &findings,
        &output_dir,
        &project_name,
        &config.tools,
    )
    .await?;

    eprintln!();
    eprintln!("===== SAST Complete =====");
    eprintln!("Reports:  {}", output_dir.display());
    if failures > 0 {
        eprintln!("Warning:  {} tool(s) failed", failures);
    }
    if format != OutputFormat::Native {
        eprintln!("Findings: {}", findings.len());
    }

    Ok(())
}
