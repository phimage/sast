use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub tool: String,
    pub rule_id: Option<String>,
    pub severity: Severity,
    pub message: String,
    pub location: Location,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Style,
    Note,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Style => write!(f, "style"),
            Severity::Note => write!(f, "note"),
        }
    }
}
