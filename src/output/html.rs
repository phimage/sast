use crate::model::Finding;
use anyhow::Result;
use std::path::Path;

pub async fn write(findings: &[Finding], output_dir: &Path, project_name: &str) -> Result<()> {
    let mut rows = String::new();
    for f in findings {
        let severity_class = format!("{}", f.severity);
        let rule = f.rule_id.as_deref().unwrap_or("-");
        let file = f.location.file.to_string_lossy();
        let line = f
            .location
            .line
            .map(|l| l.to_string())
            .unwrap_or_else(|| "-".to_string());

        rows.push_str(&format!(
            r#"<tr class="severity-{sev}">
  <td>{tool}</td>
  <td>{sev}</td>
  <td>{file}</td>
  <td>{line}</td>
  <td>{rule}</td>
  <td>{msg}</td>
</tr>
"#,
            sev = severity_class,
            tool = html_escape(&f.tool),
            file = html_escape(&file),
            line = line,
            rule = html_escape(rule),
            msg = html_escape(&f.message),
        ));
    }

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>SAST Report - {project}</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 2rem; background: #f5f5f5; }}
  h1 {{ color: #333; }}
  .summary {{ margin: 1rem 0; padding: 1rem; background: #fff; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
  table {{ border-collapse: collapse; width: 100%; background: #fff; border-radius: 8px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
  th {{ background: #2c3e50; color: #fff; padding: 12px; text-align: left; }}
  td {{ padding: 10px 12px; border-bottom: 1px solid #eee; }}
  tr:hover {{ background: #f8f9fa; }}
  .severity-error td:nth-child(2) {{ color: #e74c3c; font-weight: bold; }}
  .severity-warning td:nth-child(2) {{ color: #f39c12; font-weight: bold; }}
  .severity-info td:nth-child(2) {{ color: #3498db; }}
  .severity-style td:nth-child(2) {{ color: #9b59b6; }}
  .severity-note td:nth-child(2) {{ color: #7f8c8d; }}
  .count {{ display: inline-block; padding: 4px 12px; border-radius: 12px; margin: 0 4px; font-weight: bold; }}
  .count-error {{ background: #fde8e8; color: #e74c3c; }}
  .count-warning {{ background: #fef3e2; color: #f39c12; }}
  .count-total {{ background: #e8f4fd; color: #2980b9; }}
</style>
</head>
<body>
<h1>SAST Report: {project}</h1>
<div class="summary">
  <span class="count count-total">{total} findings</span>
  <span class="count count-error">{errors} errors</span>
  <span class="count count-warning">{warnings} warnings</span>
</div>
<table>
<thead>
<tr><th>Tool</th><th>Severity</th><th>File</th><th>Line</th><th>Rule</th><th>Message</th></tr>
</thead>
<tbody>
{rows}
</tbody>
</table>
</body>
</html>"#,
        project = html_escape(project_name),
        total = findings.len(),
        errors = findings
            .iter()
            .filter(|f| matches!(f.severity, crate::model::Severity::Error))
            .count(),
        warnings = findings
            .iter()
            .filter(|f| matches!(f.severity, crate::model::Severity::Warning))
            .count(),
        rows = rows,
    );

    let path = output_dir.join("report.html");
    tokio::fs::write(&path, html).await?;
    eprintln!("[sast] HTML report saved to {}", path.display());
    Ok(())
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
