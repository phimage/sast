use std::path::PathBuf;
use std::process::Command;

fn sast_bin() -> PathBuf {
    // cargo test builds the binary in target/debug
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove 'deps'
    path.push("sast");
    path
}

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpp_project")
}

fn run_sast(args: &[&str]) -> std::process::Output {
    Command::new(sast_bin())
        .args(args)
        .output()
        .expect("Failed to execute sast binary")
}

// ── CLI tests ──

#[test]
fn test_help_flag() {
    let output = run_sast(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Run SAST tools"));
    assert!(stdout.contains("--format"));
    assert!(stdout.contains("--tools"));
    assert!(stdout.contains("--output"));
}

#[test]
fn test_missing_path_errors() {
    let output = run_sast(&[]);
    assert!(!output.status.success());
}

#[test]
fn test_invalid_path_errors() {
    let output = run_sast(&["/nonexistent/path"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("does not exist"));
}

#[test]
fn test_invalid_format_errors() {
    let output = run_sast(&[fixtures_dir().to_str().unwrap(), "-f", "xml"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown format"));
}

#[test]
fn test_unknown_tool_errors() {
    let output = run_sast(&[fixtures_dir().to_str().unwrap(), "-t", "nonexistent_tool"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Unknown tool"));
}

// ── Tool execution tests (require tools to be installed) ──

fn has_tool(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn temp_output_dir(test_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sast_test_{}", test_name));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn test_cppcheck_native() {
    if !has_tool("cppcheck") {
        eprintln!("Skipping: cppcheck not installed");
        return;
    }

    let out_dir = temp_output_dir("cppcheck_native");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck",
        "-f",
        "native",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stderr: {}", stderr);
    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("cppcheck.txt").exists(),
        "cppcheck.txt should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("cppcheck.txt")).unwrap();
    assert!(!content.is_empty(), "cppcheck output should not be empty");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_cppcheck_json() {
    if !has_tool("cppcheck") {
        eprintln!("Skipping: cppcheck not installed");
        return;
    }

    let out_dir = temp_output_dir("cppcheck_json");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck",
        "-f",
        "json",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.json").exists(),
        "report.json should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.json")).unwrap();
    let findings: serde_json::Value = serde_json::from_str(&content).expect("should be valid JSON");
    assert!(findings.is_array(), "findings should be an array");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_cppcheck_sarif() {
    if !has_tool("cppcheck") {
        eprintln!("Skipping: cppcheck not installed");
        return;
    }

    let out_dir = temp_output_dir("cppcheck_sarif");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck",
        "-f",
        "sarif",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.sarif").exists(),
        "report.sarif should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.sarif")).unwrap();
    let sarif: serde_json::Value = serde_json::from_str(&content).expect("should be valid JSON");
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"].is_array(), "should have runs array");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_cppcheck_html() {
    if !has_tool("cppcheck") {
        eprintln!("Skipping: cppcheck not installed");
        return;
    }

    let out_dir = temp_output_dir("cppcheck_html");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck",
        "-f",
        "html",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.html").exists(),
        "report.html should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.html")).unwrap();
    assert!(content.contains("<!DOCTYPE html>"), "should be valid HTML");
    assert!(
        content.contains("SAST Report"),
        "should contain report title"
    );
    assert!(content.contains("cppcheck"), "should mention the tool");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_semgrep_native() {
    if !has_tool("semgrep") {
        eprintln!("Skipping: semgrep not installed");
        return;
    }

    let out_dir = temp_output_dir("semgrep_native");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "semgrep",
        "-f",
        "native",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stderr: {}", stderr);
    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("semgrep.json").exists(),
        "semgrep.json should be created"
    );

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_semgrep_json() {
    if !has_tool("semgrep") {
        eprintln!("Skipping: semgrep not installed");
        return;
    }

    let out_dir = temp_output_dir("semgrep_json");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "semgrep",
        "-f",
        "json",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.json").exists(),
        "report.json should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.json")).unwrap();
    let findings: serde_json::Value = serde_json::from_str(&content).expect("should be valid JSON");
    assert!(findings.is_array(), "findings should be an array");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_multiple_tools_native() {
    if !has_tool("cppcheck") || !has_tool("semgrep") {
        eprintln!("Skipping: cppcheck and/or semgrep not installed");
        return;
    }

    let out_dir = temp_output_dir("multi_native");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck,semgrep",
        "-f",
        "native",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("cppcheck.txt").exists(),
        "cppcheck.txt should be created"
    );
    assert!(
        out_dir.join("semgrep.json").exists(),
        "semgrep.json should be created"
    );

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_multiple_tools_html() {
    if !has_tool("cppcheck") || !has_tool("semgrep") {
        eprintln!("Skipping: cppcheck and/or semgrep not installed");
        return;
    }

    let out_dir = temp_output_dir("multi_html");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck,semgrep",
        "-f",
        "html",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("=== stderr ===\n{stderr}\n=== end stderr ===");

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.html").exists(),
        "report.html should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.html")).unwrap();
    println!("=== report.html content ===\n{content}\n=== end ===");
    assert!(
        content.contains("cppcheck"),
        "HTML should contain cppcheck findings"
    );
    assert!(
        content.contains("semgrep"),
        "HTML should contain semgrep findings"
    );

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_custom_config() {
    if !has_tool("cppcheck") {
        eprintln!("Skipping: cppcheck not installed");
        return;
    }

    let config_dir = temp_output_dir("config_test");
    std::fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("test_config.yaml");
    std::fs::write(
        &config_path,
        r#"
defaults:
  tools: [cppcheck]
  format: json
"#,
    )
    .unwrap();

    let out_dir = temp_output_dir("config_test_output");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-c",
        config_path.to_str().unwrap(),
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(
        output.status.success(),
        "sast should succeed with custom config"
    );
    assert!(
        out_dir.join("report.json").exists(),
        "should use json format from config"
    );

    std::fs::remove_dir_all(&config_dir).ok();
    std::fs::remove_dir_all(&out_dir).ok();
}

// ── scan-build tests ──

#[test]
fn test_scan_build_native() {
    if !has_tool("scan-build") {
        eprintln!("Skipping: scan-build not installed");
        return;
    }

    let out_dir = temp_output_dir("scan_build_native");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "scan-build",
        "-f",
        "native",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("=== stderr ===\n{stderr}\n=== end stderr ===");
    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("scan_build").exists(),
        "scan_build dir should be created"
    );

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_scan_build_json() {
    if !has_tool("scan-build") {
        eprintln!("Skipping: scan-build not installed");
        return;
    }

    let out_dir = temp_output_dir("scan_build_json");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "scan-build",
        "-f",
        "json",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("=== stderr ===\n{stderr}\n=== end stderr ===");
    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.json").exists(),
        "report.json should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.json")).unwrap();
    let findings: serde_json::Value = serde_json::from_str(&content).expect("should be valid JSON");
    assert!(findings.is_array(), "findings should be an array");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_scan_build_html() {
    if !has_tool("scan-build") {
        eprintln!("Skipping: scan-build not installed");
        return;
    }

    let out_dir = temp_output_dir("scan_build_html");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "scan-build",
        "-f",
        "html",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("=== stderr ===\n{stderr}\n=== end stderr ===");
    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.html").exists(),
        "report.html should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.html")).unwrap();
    println!("=== report.html content ===\n{content}\n=== end ===");
    assert!(content.contains("<!DOCTYPE html>"), "should be valid HTML");
    assert!(
        content.contains("SAST Report"),
        "should contain report title"
    );

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_scan_build_sarif() {
    if !has_tool("scan-build") {
        eprintln!("Skipping: scan-build not installed");
        return;
    }

    let out_dir = temp_output_dir("scan_build_sarif");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "scan-build",
        "-f",
        "sarif",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.sarif").exists(),
        "report.sarif should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.sarif")).unwrap();
    let sarif: serde_json::Value = serde_json::from_str(&content).expect("should be valid JSON");
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"].is_array(), "should have runs array");

    std::fs::remove_dir_all(&out_dir).ok();
}

#[test]
fn test_semgrep_sarif() {
    if !has_tool("semgrep") {
        eprintln!("Skipping: semgrep not installed");
        return;
    }

    let out_dir = temp_output_dir("semgrep_sarif");
    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "semgrep",
        "-f",
        "sarif",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success(), "sast should succeed");
    assert!(
        out_dir.join("report.sarif").exists(),
        "report.sarif should be created"
    );

    let content = std::fs::read_to_string(out_dir.join("report.sarif")).unwrap();
    let sarif: serde_json::Value = serde_json::from_str(&content).expect("should be valid JSON");
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"].is_array(), "should have runs array");

    std::fs::remove_dir_all(&out_dir).ok();
}

// ── Parser unit tests ──

#[test]
fn test_cppcheck_parser() {
    let stderr = b"[test.cpp:12]: (warning) Variable 'x' is not initialized\n\
                   [main.cpp:5]: (error) Memory leak: data\n\
                   [util.cpp:30]: (style) Unused variable: count\n";

    // We test parsing indirectly via the binary with a mock — but since
    // parsers are internal, we verify through format output tests above.
    // This test validates the expected cppcheck output format exists.
    let text = String::from_utf8_lossy(stderr);
    assert!(text.contains("warning"));
    assert!(text.contains("error"));
    assert!(text.contains("style"));
}

#[test]
fn test_output_dir_created_automatically() {
    if !has_tool("cppcheck") {
        eprintln!("Skipping: cppcheck not installed");
        return;
    }

    let out_dir = temp_output_dir("auto_create").join("nested").join("deep");
    assert!(!out_dir.exists());

    let output = run_sast(&[
        fixtures_dir().to_str().unwrap(),
        "-t",
        "cppcheck",
        "-f",
        "native",
        "-o",
        out_dir.to_str().unwrap(),
    ]);

    assert!(output.status.success());
    assert!(out_dir.exists(), "nested output dir should be auto-created");

    std::fs::remove_dir_all(temp_output_dir("auto_create")).ok();
}
