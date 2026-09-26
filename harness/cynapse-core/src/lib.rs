use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use regex::Regex;
pub mod bots;
pub mod compressor;
pub mod doctor;
pub mod downloader;
pub mod offline_agent;
pub mod persona;
pub mod receipts;
pub mod session;
pub mod subagent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub concurrency_safe: bool,
}

pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file",
            description: "Read text contents of a file within permitted workspace",
            concurrency_safe: true,
        },
        ToolDefinition {
            name: "write_file",
            description: "Write content to a file within permitted workspace",
            concurrency_safe: false,
        },
        ToolDefinition {
            name: "grep",
            description: "Search regex pattern recursively across workspace files",
            concurrency_safe: true,
        },
        ToolDefinition {
            name: "execute_command",
            description: "Execute a safe bash shell command with timeout",
            concurrency_safe: false,
        },
        ToolDefinition {
            name: "spawn_subagent",
            description: "Spawn a delegated autonomous subagent task with optional bot profile and wait mode",
            concurrency_safe: false,
        },
    ]
}

/// Returns true if a tool is safe for concurrent read/inspection execution.
pub fn is_tool_concurrency_safe(name: &str) -> bool {
    list_tools()
        .into_iter()
        .find(|t| t.name == name)
        .map(|t| t.concurrency_safe)
        .unwrap_or(false)
}

/// Partitions a slice of tool calls into execution batches:
/// contiguous runs of concurrency-safe tools are grouped together,
/// while exclusive (state-mutating) tools are placed into individual singleton batches.
pub fn partition_tool_calls(calls: &[offline_agent::ToolCall]) -> Vec<Vec<offline_agent::ToolCall>> {
    let mut partitions = Vec::new();
    let mut current_safe_batch = Vec::new();

    for call in calls {
        if is_tool_concurrency_safe(&call.name) {
            current_safe_batch.push(call.clone());
        } else {
            if !current_safe_batch.is_empty() {
                partitions.push(std::mem::take(&mut current_safe_batch));
            }
            partitions.push(vec![call.clone()]);
        }
    }

    if !current_safe_batch.is_empty() {
        partitions.push(current_safe_batch);
    }

    partitions
}

/// Validate path safety: prevents path traversal and symlink bypass to sensitive system files.
fn validate_safe_path(p_str: &str, for_write: bool) -> Result<PathBuf> {
    let raw_path = if p_str.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(p_str.trim_start_matches('~').trim_start_matches('/'))
        } else {
            PathBuf::from(p_str)
        }
    } else {
        PathBuf::from(p_str)
    };
    let normalized = if raw_path.is_absolute() {
        raw_path
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(raw_path)
    };

    let resolved = if normalized.exists() {
        normalized.canonicalize().unwrap_or_else(|_| normalized.clone())
    } else if let Some(parent) = normalized.parent() {
        if parent.exists() {
            parent.canonicalize().map(|p| p.join(normalized.file_name().unwrap_or_default())).unwrap_or_else(|_| normalized.clone())
        } else {
            normalized.clone()
        }
    } else {
        normalized.clone()
    };

    let p_str_clean = normalized.to_string_lossy();
    let resolved_str = resolved.to_string_lossy();

    let sensitive_patterns = [
        "/etc/shadow", "/etc/passwd", "/etc/sudoers", "/etc/master.passwd",
        "/.ssh", "/root", "/proc/kcore", "/dev/mem", "/dev/kmem",
    ];

    for sensitive in &sensitive_patterns {
        if p_str_clean.contains(sensitive) || resolved_str.contains(sensitive) {
            bail!("Access denied: path '{}' accesses sensitive system resources.", p_str);
        }
    }

    if for_write {
        if resolved_str == "/" || resolved_str == "/etc" || resolved_str == "/usr" || resolved_str == "/bin" || resolved_str == "/sbin" {
            bail!("Access denied: cannot write to system directory '{}'.", p_str);
        }
    }

    Ok(resolved)
}

/// Shared safe file reading helper returning the canonical path and full content.
pub(crate) fn read_file_at(path: &str) -> Result<(PathBuf, String)> {
    let safe_path = validate_safe_path(path, false)?;
    if !safe_path.exists() {
        bail!("File not found: {}", path);
    }
    if safe_path.is_dir() {
        bail!("Path is a directory, not a file: {}", path);
    }
    let metadata = fs::metadata(&safe_path)?;
    if metadata.len() > 10 * 1024 * 1024 {
        bail!("File too large to read into context ({} bytes, limit is 10 MB)", metadata.len());
    }
    let content = fs::read_to_string(&safe_path)
        .with_context(|| format!("Failed to read file {}", safe_path.display()))?;
    Ok((safe_path, content))
}

/// Native implementation of atomic-agent tools with sandboxing and execution timeout.
pub fn execute_tool(name: &str, arg1: &str, arg2: Option<&str>) -> Result<String> {
    match name {
        "read_file" => {
            let (_path, content) = read_file_at(arg1)?;
            Ok(content)
        }
        "write_file" => {
            let mut safe_path = validate_safe_path(arg1, true)?;
            if safe_path.is_dir() {
                safe_path = safe_path.join("about.md");
            }
            let content = arg2.unwrap_or_default();
            if let Some(parent) = safe_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&safe_path, content)
                .with_context(|| format!("Failed to write file {}", safe_path.display()))?;
            Ok(format!("Successfully wrote {} bytes to {}", content.len(), safe_path.display()))
        }
        "grep" => {
            let pattern = arg1;
            let dir_path = arg2.unwrap_or(".");
            let safe_dir = validate_safe_path(dir_path, false)?;
            let re = Regex::new(pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;
            let mut matches = Vec::new();

            fn walk_and_grep(dir: &Path, re: &Regex, matches: &mut Vec<String>, depth: usize) {
                if depth > 8 || matches.len() >= 50 {
                    return;
                }
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                        if fname.starts_with('.') || fname == "target" || fname == "node_modules" {
                            continue;
                        }
                        if path.is_file() {
                            if let Ok(content) = fs::read_to_string(&path) {
                                for (line_no, line) in content.lines().enumerate() {
                                    if re.is_match(line) {
                                        matches.push(format!("{}:{}: {}", path.display(), line_no + 1, line.trim()));
                                        if matches.len() >= 50 {
                                            return;
                                        }
                                    }
                                }
                            }
                        } else if path.is_dir() {
                            walk_and_grep(&path, re, matches, depth + 1);
                        }
                    }
                }
            }

            if safe_dir.is_file() {
                if let Ok(content) = fs::read_to_string(&safe_dir) {
                    for (line_no, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            matches.push(format!("{}:{}: {}", safe_dir.display(), line_no + 1, line.trim()));
                        }
                    }
                }
            } else {
                walk_and_grep(&safe_dir, &re, &mut matches, 0);
            }

            if matches.is_empty() {
                Ok(format!("No matches found for pattern '{}' in {}", pattern, dir_path))
            } else {
                Ok(matches.join("\n"))
            }
        }
        "execute_command" => {
            let trimmed = arg1.trim();
            // Block dangerous commands
            let dangerous = ["rm -rf /", "rm -rf /*", "mkfs", ":(){ :|:& };:", "> /dev/sda"];
            for d in &dangerous {
                if trimmed.contains(d) {
                    bail!("Security violation: destructive command blocked: '{}'", trimmed);
                }
            }

            let mut child = Command::new("bash")
                .arg("-c")
                .arg(trimmed)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to spawn command: {}", trimmed))?;

            // Timeout enforcement: 10 seconds
            let start = std::time::Instant::now();
            let timeout = Duration::from_secs(10);

            while start.elapsed() < timeout {
                if let Ok(Some(status)) = child.try_wait() {
                    let output = child.wait_with_output()?;
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Ok(format!("Exit Status: {}\n{}{}", status, stdout, stderr));
                }
                std::thread::sleep(Duration::from_millis(50));
            }

            let _ = child.kill();
            bail!("Command timed out after 10 seconds: {}", trimmed);
        }
        _ => bail!("Unknown tool: {}", name),
    }
}

/// Executes a tool enforcing a BotProfile's allow/deny rules and workspace confinement.
pub fn execute_tool_with_profile(
    name: &str,
    arg1: &str,
    arg2: Option<&str>,
    profile: &bots::BotProfile,
) -> Result<String> {
    if !profile.is_tool_allowed(name) {
        bail!("Permission denied: tool '{}' is not permitted by bot profile '@{}'", name, profile.slug);
    }

    if profile.requires_approval(name) {
        bail!("Approval required: tool '{}' requires interactive confirmation (in tools_ask for '@{}')", name, profile.slug);
    }

    // Path confinement check for file tools
    match name {
        "read_file" | "write_file" => {
            if !profile.is_path_in_workspace(arg1) {
                bail!(
                    "Workspace violation: path '{}' is outside the restricted workspace '{:?}' of bot profile '@{}'",
                    arg1,
                    profile.workspace_restrict,
                    profile.slug
                );
            }
        }
        "grep" => {
            let dir = arg2.unwrap_or(".");
            if !profile.is_path_in_workspace(dir) {
                bail!(
                    "Workspace violation: directory '{}' is outside the restricted workspace '{:?}' of bot profile '@{}'",
                    dir,
                    profile.workspace_restrict,
                    profile.slug
                );
            }
        }
        _ => {}
    }

    execute_tool(name, arg1, arg2)
}

/// Executes a tool enforcing a BotProfile's allow/deny rules and workspace confinement,
/// bypassing the `requires_approval` check because explicit user approval was already granted.
pub fn execute_tool_with_profile_approved(
    name: &str,
    arg1: &str,
    arg2: Option<&str>,
    profile: &bots::BotProfile,
) -> Result<String> {
    if !profile.is_tool_allowed(name) {
        bail!("Permission denied: tool '{}' is not permitted by bot profile '@{}'", name, profile.slug);
    }

    // Path confinement check for file tools
    match name {
        "read_file" | "write_file" => {
            if !profile.is_path_in_workspace(arg1) {
                bail!(
                    "Workspace violation: path '{}' is outside the restricted workspace '{:?}' of bot profile '@{}'",
                    arg1,
                    profile.workspace_restrict,
                    profile.slug
                );
            }
        }
        "grep" => {
            let dir = arg2.unwrap_or(".");
            if !profile.is_path_in_workspace(dir) {
                bail!(
                    "Workspace violation: directory '{}' is outside the restricted workspace '{:?}' of bot profile '@{}'",
                    dir,
                    profile.workspace_restrict,
                    profile.slug
                );
            }
        }
        _ => {}
    }

    execute_tool(name, arg1, arg2)
}

/// Real HuggingFace model downloader streaming target .gguf or .safetensors files into models_dir.
pub async fn pull_huggingface_model(url_or_repo: &str, models_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(models_dir)?;
    let (download_url, target_filename) = downloader::resolve_hf_download_url_async(url_or_repo, "Q4_K_M").await;
    let target_path = models_dir.join(&target_filename);

    println!("📥 Downloading HuggingFace Model...");
    println!("   URL: {}", download_url);
    println!("   Target: {}", target_path.display());

    let client = reqwest::Client::new();
    let res = client
        .get(&download_url)
        .header("User-Agent", "Cynapse-Agent-Downloader/1.0")
        .send()
        .await
        .with_context(|| format!("Failed to connect to HuggingFace URL: {}", download_url))?;

    if !res.status().is_success() {
        bail!("HuggingFace server returned HTTP status: {}", res.status());
    }

    let mut file = File::create(&target_path)
        .with_context(|| format!("Failed to create destination file: {}", target_path.display()))?;

    let mut stream = res.bytes_stream();
    let mut downloaded_bytes = 0u64;

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.context("Error downloading chunk from stream")?;
        file.write_all(&bytes)?;
        downloaded_bytes += bytes.len() as u64;
    }

    println!("✓ Download complete: {:.2} MB saved to {}", downloaded_bytes as f64 / 1_048_576.0, target_path.display());
    Ok(target_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_sanitization() {
        assert!(validate_safe_path("/etc/shadow", false).is_err());
        assert!(validate_safe_path("/root/.ssh/id_rsa", false).is_err());
        assert!(validate_safe_path("src/lib.rs", false).is_ok());
    }

    #[test]
    fn test_symlink_safety() {
        let temp_dir = std::env::temp_dir();
        let symlink_path = temp_dir.join("cynapse_test_symlink_passwd");
        let _ = fs::remove_file(&symlink_path);
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            if symlink("/etc/passwd", &symlink_path).is_ok() {
                let res = validate_safe_path(&symlink_path.to_string_lossy(), false);
                let _ = fs::remove_file(&symlink_path);
                assert!(res.is_err());
            }
        }
    }

    #[test]
    fn test_tool_concurrency_classification() {
        assert!(is_tool_concurrency_safe("read_file"));
        assert!(is_tool_concurrency_safe("grep"));
        assert!(!is_tool_concurrency_safe("write_file"));
        assert!(!is_tool_concurrency_safe("execute_command"));
        assert!(!is_tool_concurrency_safe("spawn_subagent"));
        assert!(!is_tool_concurrency_safe("unknown_tool"));
    }

    #[test]
    fn test_partition_tool_calls_mixed() {
        use serde_json::json;
        let calls = vec![
            offline_agent::ToolCall {
                name: "read_file".into(),
                arguments: json!({"path": "a.txt"}),
            },
            offline_agent::ToolCall {
                name: "grep".into(),
                arguments: json!({"pattern": "foo"}),
            },
            offline_agent::ToolCall {
                name: "write_file".into(),
                arguments: json!({"path": "b.txt", "content": "hi"}),
            },
            offline_agent::ToolCall {
                name: "execute_command".into(),
                arguments: json!({"command": "ls"}),
            },
            offline_agent::ToolCall {
                name: "read_file".into(),
                arguments: json!({"path": "c.txt"}),
            },
        ];

        let partitions = partition_tool_calls(&calls);
        assert_eq!(partitions.len(), 4);
        // Batch 1: contiguous safe [read_file, grep]
        assert_eq!(partitions[0].len(), 2);
        assert_eq!(partitions[0][0].name, "read_file");
        assert_eq!(partitions[0][1].name, "grep");
        // Batch 2: singleton exclusive [write_file]
        assert_eq!(partitions[1].len(), 1);
        assert_eq!(partitions[1][0].name, "write_file");
        // Batch 3: singleton exclusive [execute_command]
        assert_eq!(partitions[2].len(), 1);
        assert_eq!(partitions[2][0].name, "execute_command");
        // Batch 4: singleton safe [read_file]
        assert_eq!(partitions[3].len(), 1);
        assert_eq!(partitions[3][0].name, "read_file");
    }

    #[test]
    fn test_execute_command_timeout_and_block() {
        assert!(execute_tool("execute_command", "rm -rf /", None).is_err());
        let res = execute_tool("execute_command", "echo 'hello cynapse'", None);
        assert!(res.is_ok());
        assert!(res.unwrap().contains("hello cynapse"));
    }

    #[test]
    fn test_execute_tool_with_profile_permissions_and_confinement() {
        let temp = tempfile::tempdir().unwrap();
        let ws = temp.path().join("sub_ws");
        fs::create_dir_all(&ws).unwrap();
        let safe_file = ws.join("test.txt");
        fs::write(&safe_file, "hello from safe ws").unwrap();

        let restricted_profile = bots::BotProfile {
            slug: "isolated".into(),
            display_name: "Isolated Bot".into(),
            description: "".into(),
            persona_file: None,
            model_preset: None,
            tools_allow: vec!["read_file".into()],
            tools_deny: vec!["write_file".into()],
            tools_ask: vec![],
            workspace_restrict: Some(ws.to_string_lossy().to_string()),
            max_concurrent: None,
        };

        // Allowed inside workspace
        let res = execute_tool_with_profile("read_file", &safe_file.to_string_lossy(), None, &restricted_profile);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "hello from safe ws");

        // Denied tool (write_file)
        let res_write = execute_tool_with_profile("write_file", &safe_file.to_string_lossy(), Some("new"), &restricted_profile);
        assert!(res_write.is_err());
        assert!(res_write.unwrap_err().to_string().contains("Permission denied"));

        // Tool outside workspace (read outside)
        let outside_file = temp.path().join("outside.txt");
        fs::write(&outside_file, "secret").unwrap();
        let res_outside = execute_tool_with_profile("read_file", &outside_file.to_string_lossy(), None, &restricted_profile);
        assert!(res_outside.is_err());
        assert!(res_outside.unwrap_err().to_string().contains("Workspace violation"));

        // Tool requiring interactive approval (tools_ask)
        let ask_profile = bots::BotProfile {
            slug: "cautious".into(),
            display_name: "Cautious Bot".into(),
            description: "".into(),
            persona_file: None,
            model_preset: None,
            tools_allow: vec!["*".into()],
            tools_deny: vec![],
            tools_ask: vec!["execute_command".into()],
            workspace_restrict: None,
            max_concurrent: None,
        };
        let res_ask = execute_tool_with_profile("execute_command", "ls", None, &ask_profile);
        assert!(res_ask.is_err());
        assert!(res_ask.unwrap_err().to_string().contains("Approval required"));
    }
}
