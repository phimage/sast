# sast

A command-line tool that orchestrates multiple SAST (Static Analysis Security Testing) tools and aggregates their results into unified reports.

## Features

- Run multiple SAST tools in parallel
- Unified output in multiple formats: native, JSON, SARIF, HTML
- Configurable via YAML — add custom tools without changing code
- Built-in support for cppcheck, scan-build (Clang), and semgrep
- Works out of the box with zero configuration

## Installation

```bash
cargo build --release
cp target/release/sast /usr/local/bin/
```

### Prerequisites

Install the SAST tools you want to use:

```bash
# macOS
brew install cppcheck llvm semgrep

# Ubuntu/Debian
apt install cppcheck clang-tools semgrep
```

## Usage

```bash
# Run all default tools with native output
sast /path/to/project

# Generate an HTML report
sast /path/to/project -f html

# Generate a SARIF report (for IDE/CI integration)
sast /path/to/project -f sarif

# Run specific tools only
sast /path/to/project -t cppcheck,semgrep

# Custom output directory
sast /path/to/project -o ./reports

# Use a custom config file
sast /path/to/project -c my-config.yaml
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `<PATH>` | Path to the project to analyze | required |
| `-f, --format` | Output format: `native`, `json`, `sarif`, `html` | `native` |
| `-o, --output` | Output directory | `<PATH>/sast_report/` |
| `-t, --tools` | Comma-separated list of tools to run | config default |
| `-c, --config` | Path to YAML config file | auto-detected |

## Configuration

Configuration is optional. Files are loaded in order (later values override earlier ones):

1. Built-in defaults
2. `~/.config/sast/config.yaml`
3. `./sast.yaml` (project-local)
4. `--config <path>`
5. CLI flags

### Example `sast.yaml`

```yaml
defaults:
  tools: [cppcheck, semgrep]
  format: html

tools:
  cppcheck:
    command: cppcheck
    args: ["--enable=all", "--inconclusive", "--std=c++17", "--force"]
    output_stream: stderr
    native_extension: txt

  scan-build:
    command: scan-build
    args: ["-o", "{output_dir}/scan_build", "clang++", "-std=c++17", "-Wall", "-Wextra", "-c"]
    append_sources: "*.cpp"
    output_stream: filesystem
    native_extension: html

  semgrep:
    command: semgrep
    args: ["--config=auto", "--json"]
    output_stream: stdout
    native_extension: json
```

### Adding a custom tool

Add any tool via YAML — no code changes needed:

```yaml
tools:
  flawfinder:
    command: flawfinder
    args: ["--columns", "--context"]
    output_stream: stdout
    native_extension: txt
```

### Tool config fields

| Field | Description |
|-------|-------------|
| `command` | Executable name or path |
| `args` | List of arguments (`{output_dir}` and `{project_path}` are interpolated) |
| `output_stream` | Where the tool writes results: `stdout`, `stderr`, or `filesystem` |
| `native_extension` | File extension for native output (e.g. `txt`, `json`) |
| `append_sources` | Glob pattern of source files to append to args (e.g. `*.cpp`) |

## Output Formats

- **native** — each tool's raw output saved as-is
- **json** — all findings normalized into a single JSON file
- **sarif** — [SARIF 2.1.0](https://sarifweb.azurewebsites.net/) for CI/CD and IDE integration
- **html** — styled HTML report with severity summary and findings table
