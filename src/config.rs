use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub tools: HashMap<String, ToolConfig>,
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
pub struct Defaults {
    pub tools: Option<Vec<String>>,
    pub format: Option<String>,
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ToolConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub output_stream: OutputStream,
    pub native_extension: Option<String>,
    pub append_sources: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputStream {
    #[default]
    Stdout,
    Stderr,
    Filesystem,
}

impl Config {
    pub fn builtin_defaults() -> Self {
        let mut tools = HashMap::new();

        tools.insert(
            "cppcheck".to_string(),
            ToolConfig {
                command: "cppcheck".to_string(),
                args: vec![
                    "--enable=all".into(),
                    "--inconclusive".into(),
                    "--std=c++17".into(),
                    "--force".into(),
                ],
                output_stream: OutputStream::Stderr,
                native_extension: Some("txt".into()),
                append_sources: None,
            },
        );

        tools.insert(
            "scan-build".to_string(),
            ToolConfig {
                command: "scan-build".to_string(),
                args: vec![
                    "-o".into(),
                    "{output_dir}/scan_build".into(),
                    "clang++".into(),
                    "-std=c++17".into(),
                    "-Wall".into(),
                    "-Wextra".into(),
                    "-c".into(),
                ],
                output_stream: OutputStream::Filesystem,
                native_extension: Some("html".into()),
                append_sources: Some("*.cpp".into()),
            },
        );

        tools.insert(
            "semgrep".to_string(),
            ToolConfig {
                command: "semgrep".to_string(),
                args: vec![
                    "--config=auto".into(),
                    "--config=rules/semgrep/".into(),
                    "--json".into(),
                ],
                output_stream: OutputStream::Stdout,
                native_extension: Some("json".into()),
                append_sources: None,
            },
        );

        Config {
            defaults: Defaults {
                tools: Some(vec![
                    "cppcheck".into(),
                    "scan-build".into(),
                    "semgrep".into(),
                ]),
                format: Some("native".into()),
                output: None,
            },
            tools,
        }
    }

    pub fn merge(&mut self, other: Config) {
        if other.defaults.tools.is_some() {
            self.defaults.tools = other.defaults.tools;
        }
        if other.defaults.format.is_some() {
            self.defaults.format = other.defaults.format;
        }
        if other.defaults.output.is_some() {
            self.defaults.output = other.defaults.output;
        }
        for (name, tool) in other.tools {
            self.tools.insert(name, tool);
        }
    }
}

pub fn load_config(cli_config_path: Option<&Path>) -> Result<Config> {
    let mut config = Config::builtin_defaults();

    let candidates: Vec<Option<PathBuf>> = vec![
        dirs::config_dir().map(|d| d.join("sast/config.yaml")),
        Some(PathBuf::from("sast.yaml")),
        cli_config_path.map(PathBuf::from),
    ];

    for path in candidates.into_iter().flatten() {
        if path.exists() {
            let text = std::fs::read_to_string(&path)?;
            let layer: Config = serde_yaml::from_str(&text)?;
            tracing::info!("Loaded config from {}", path.display());
            config.merge(layer);
        }
    }

    Ok(config)
}
