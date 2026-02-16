use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sast", about = "Run SAST tools and aggregate results")]
pub struct Cli {
    /// Path to the project to analyze
    pub path: PathBuf,

    /// Output format: native, sarif, html, json [default: native]
    #[arg(short, long)]
    pub format: Option<String>,

    /// Output folder (default: <path>/sast_report/)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Comma-separated list of tools to run (overrides config)
    #[arg(short, long, value_delimiter = ',')]
    pub tools: Option<Vec<String>>,

    /// Path to config file
    #[arg(short, long)]
    pub config: Option<PathBuf>,
}
