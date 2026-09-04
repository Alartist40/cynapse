use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use regex::Regex;
pub mod session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "read_file",
            description: "Read text contents of a file",
        },
        ToolDefinition {
            name: "write_file",
            description: "Write content to a file",
        },
        ToolDefinition {
            name: "grep",
            description: "Search regex pattern across directory files",
        },
        ToolDefinition {
            name: "execute_command",
            description: "Execute a bash shell command",
        },
    ]
}

/// Native implementation of atomic-agent tools (read_file, write_file, grep, execute_command).
pub fn execute_tool(name: &str, arg1: &str, arg2: Option<&str>) -> Result<String> {
    match name {
        "read_file" => {
            let content = fs::read_to_string(arg1).with_context(|| format!("Failed to read file {}", arg1))?;
            Ok(content)
        }
        "write_file" => {
            let content = arg2.unwrap_or_default();
            fs::write(arg1, content).with_context(|| format!("Failed to write file {}", arg1))?;
            Ok(format!("Successfully wrote {} bytes to {}", content.len(), arg1))
        }
        "grep" => {
            let pattern = arg1;
            let dir_path = arg2.unwrap_or(".");
            let re = Regex::new(pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;
            let mut matches = Vec::new();

            let target = Path::new(dir_path);
            if target.is_file() {
                if let Ok(content) = fs::read_to_string(target) {
                    for (line_no, line) in content.lines().enumerate() {
                        if re.is_match(line) {
                            matches.push(format!("{}:{}: {}", target.display(), line_no + 1, line.trim()));
                        }
                    }
                }
            } else if target.is_dir() {
                if let Ok(entries) = fs::read_dir(target) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Ok(content) = fs::read_to_string(&path) {
                                for (line_no, line) in content.lines().enumerate() {
                                    if re.is_match(line) {
                                        matches.push(format!("{}:{}: {}", path.display(), line_no + 1, line.trim()));
                                        if matches.len() >= 50 {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if matches.is_empty() {
                Ok(format!("No matches found for pattern '{}' in {}", pattern, dir_path))
            } else {
                Ok(matches.join("\n"))
            }
        }
        "execute_command" => {
            let output = Command::new("bash")
                .arg("-c")
                .arg(arg1)
                .output()
                .with_context(|| format!("Failed to execute command: {}", arg1))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("{}\n{}", stdout, stderr))
        }
        _ => Ok(format!("Unknown tool: {}", name)),
    }
}

/// Real HuggingFace model downloader streaming target .gguf or .safetensors files into models_dir.
pub async fn pull_huggingface_model(url_or_repo: &str, models_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(models_dir)?;

    let clean_ref = url_or_repo.trim().trim_end_matches('/');
    let target_filename = if clean_ref.ends_with(".gguf") || clean_ref.ends_with(".safetensors") {
        Path::new(clean_ref).file_name().unwrap().to_string_lossy().to_string()
    } else {
        let repo_name = clean_ref.split('/').last().unwrap_or("model");
        format!("{}.gguf", repo_name)
    };

    let target_path = models_dir.join(&target_filename);

    let download_url = if clean_ref.starts_with("http://") || clean_ref.starts_with("https://") {
        if clean_ref.contains("huggingface.co") && !clean_ref.contains("/resolve/") {
            format!("{}/resolve/main/{}", clean_ref, target_filename)
        } else {
            clean_ref.to_string()
        }
    } else {
        format!("https://huggingface.co/{}/resolve/main/{}", clean_ref, target_filename)
    };

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
        anyhow::bail!("HuggingFace server returned HTTP status: {}", res.status());
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
