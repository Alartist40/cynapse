use std::fs;
use std::path::Path;
use std::time::Instant;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

/// Headroom reserve: 1.5 GiB working space for KV cache + activations
const RESERVE_BYTES: u64 = 1536 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineTier {
    Tier1Fast,
    Tier2LargeGguf,
    Tier3LargeSafetensor,
}

impl EngineTier {
    pub fn label(self) -> &'static str {
        match self {
            EngineTier::Tier1Fast => "Tier 1 (Fast llama.cpp/Ollama API)",
            EngineTier::Tier2LargeGguf => "Tier 2 (Leafcutter Rust GGUF Layer Streaming)",
            EngineTier::Tier3LargeSafetensor => "Tier 3 (Leafcutter Rust Safetensor Streaming)",
        }
    }
}

pub struct RouteDecision {
    pub tier: EngineTier,
    pub model_size_mb: f64,
    pub ram_available_mb: u64,
    pub ram_needed_mb: f64,
    pub is_safetensors: bool,
}

pub fn available_ram_mb() -> u64 {
    if let Ok(text) = fs::read_to_string("/proc/meminfo") {
        for line in text.lines() {
            if line.starts_with("MemAvailable:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(val) = kb.parse::<u64>() {
                        return val / 1024;
                    }
                }
            }
        }
    }
    4096
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHardwareInfo {
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub ram_total_mb: u64,
    pub ram_used_mb: u64,
    pub ram_avail_mb: u64,
    pub ram_used_pct: f32,
    pub gpu_info: String,
}

pub fn probe_hardware_info() -> SystemHardwareInfo {
    let mut cpu_brand = "x86_64 Processor".to_string();
    let mut cpu_cores = 0usize;
    let mut ram_total_mb = 16384u64;
    let mut ram_avail_mb = 8192u64;

    if let Ok(text) = fs::read_to_string("/proc/cpuinfo") {
        for line in text.lines() {
            if line.starts_with("model name") {
                if let Some(pos) = line.find(':') {
                    cpu_brand = line[pos + 1..].trim().to_string();
                }
            }
            if line.starts_with("processor") {
                cpu_cores += 1;
            }
        }
    }
    if cpu_cores == 0 {
        cpu_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    }

    if let Ok(text) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0u64;
        let mut avail_kb = 0u64;
        for line in text.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    total_kb = kb.parse::<u64>().unwrap_or(0);
                }
            }
            if line.starts_with("MemAvailable:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    avail_kb = kb.parse::<u64>().unwrap_or(0);
                }
            }
        }
        if total_kb > 0 {
            ram_total_mb = total_kb / 1024;
            ram_avail_mb = avail_kb / 1024;
        }
    }

    let ram_used_mb = ram_total_mb.saturating_sub(ram_avail_mb);
    let ram_used_pct = if ram_total_mb > 0 {
        (ram_used_mb as f32 / ram_total_mb as f32) * 100.0
    } else {
        0.0
    };

    let mut gpu_info = "CPU Tier (Host RAM)".to_string();
    if Path::new("/proc/driver/nvidia/gpus").exists() || Path::new("/sys/class/drm/card0").exists() {
        gpu_info = "GPU / Hardware Accel".to_string();
    }

    SystemHardwareInfo {
        cpu_brand,
        cpu_cores,
        ram_total_mb,
        ram_used_mb,
        ram_avail_mb,
        ram_used_pct,
        gpu_info,
    }
}

#[derive(Deserialize)]
struct OllamaTagsResp {
    models: Option<Vec<OllamaModelItem>>,
}

#[derive(Deserialize)]
struct OllamaModelItem {
    name: String,
}

/// Fetch list of available models from Ollama endpoint /api/tags
pub async fn fetch_ollama_models(endpoint: &str) -> Vec<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok();
    if let Some(c) = client {
        let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
        if let Ok(res) = c.get(&url).send().await {
            if let Ok(parsed) = res.json::<OllamaTagsResp>().await {
                if let Some(models) = parsed.models {
                    return models.into_iter().map(|m| m.name).collect();
                }
            }
        }
    }
    Vec::new()
}

pub fn route_model(model_path: &Path, prefer_gpu: bool) -> RouteDecision {
    let ram_mb = available_ram_mb();
    let ram_bytes = ram_mb * 1024 * 1024;

    let is_dir = model_path.is_dir();
    let is_safetensors = if is_dir {
        model_path.join("config.json").exists()
    } else {
        model_path.extension().and_then(|s| s.to_str()) == Some("safetensors")
    };

    let model_bytes = if is_dir {
        fs::read_dir(model_path)
            .map(|rd| rd.flatten().map(|e| e.metadata().map(|m| m.len()).unwrap_or(0)).sum())
            .unwrap_or(0)
    } else {
        fs::metadata(model_path).map(|m| m.len()).unwrap_or(0)
    };

    let model_size_mb = model_bytes as f64 / 1_048_576.0;
    let needed_bytes = model_bytes.saturating_add(RESERVE_BYTES);
    let ram_needed_mb = needed_bytes as f64 / 1_048_576.0;

    let tier = if is_safetensors {
        EngineTier::Tier3LargeSafetensor
    } else if prefer_gpu || needed_bytes <= ram_bytes {
        EngineTier::Tier1Fast
    } else {
        EngineTier::Tier2LargeGguf
    };

    RouteDecision {
        tier,
        model_size_mb,
        ram_available_mb: ram_mb,
        ram_needed_mb,
        is_safetensors,
    }
}

pub struct ExecutionStats {
    pub model_name: String,
    pub tokens_generated: usize,
    pub elapsed_sec: f64,
    pub tok_per_sec: f64,
    pub avail_ram_gb: f64,
}

#[derive(Serialize)]
struct GenerateOptions {
    cache_prompt: bool,
    slot_id: i32,
}

#[derive(Serialize)]
struct GenerateReq<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Deserialize)]
struct StreamChunk {
    response: Option<String>,
    done: Option<bool>,
    eval_count: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Thinking,
    Response,
}

/// Real token-by-token streaming query runner over local HTTP endpoint (Tier 1 fast).
pub async fn query_tier1_stream(
    endpoint: &str,
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    let client = reqwest::Client::new();
    let start = Instant::now();
    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));

    // Resolve model tag from Ollama endpoint if available
    let available_tags = fetch_ollama_models(endpoint).await;
    let mut resolved_model = model_name.to_string();

    if !available_tags.is_empty() && !available_tags.contains(&resolved_model) {
        let stripped = model_name.trim_end_matches(".gguf").to_string();
        if available_tags.contains(&stripped) {
            resolved_model = stripped;
        } else if let Some(matched) = available_tags.iter().find(|t| t == &&stripped || t.starts_with(&stripped)) {
            resolved_model = matched.clone();
        } else {
            resolved_model = available_tags[0].clone();
        }
    }

    let mut attempt = 0;
    let max_attempts = 3;
    let mut delay_ms = 200u64;

    let mut res = loop {
        attempt += 1;
        let req_builder = client.post(&url).json(&GenerateReq {
            model: &resolved_model,
            prompt,
            system: system_prompt,
            stream: true,
            options: GenerateOptions {
                cache_prompt: true,
                slot_id: 0,
            },
        });

        match req_builder.send().await {
            Ok(resp) if resp.status().is_success() => break resp,
            Ok(resp) if resp.status().is_server_error() && attempt < max_attempts => {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                delay_ms *= 2;
            }
            Ok(resp) => break resp,
            Err(_err) if attempt < max_attempts => {
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                delay_ms *= 2;
            }
            Err(err) => {
                anyhow::bail!("Unable to connect to local LLM engine at {}: {}", endpoint, err);
            }
        }
    };

    // Retry once with raw model_name if resolved tag returned 404
    if res.status() == reqwest::StatusCode::NOT_FOUND && resolved_model != model_name {
        if let Ok(retry_res) = client
            .post(&url)
            .json(&GenerateReq {
                model: model_name,
                prompt,
                system: system_prompt,
                stream: true,
                options: GenerateOptions {
                    cache_prompt: true,
                    slot_id: 0,
                },
            })
            .send()
            .await
        {
            if retry_res.status().is_success() {
                res = retry_res;
            }
        }
    }

    if !res.status().is_success() {
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!(
                "Model '{}' not loaded on local LLM engine (HTTP 404).\nRun: ollama run qwen2.5:0.5b  or  cynapse pull {}",
                model_name, model_name
            );
        }
        anyhow::bail!("Local LLM engine returned HTTP error: {}", res.status());
    }

    let mut stream = res.bytes_stream();
    let mut tokens_generated = 0usize;
    let mut is_thinking = false;
    let mut buffer = String::new();

    while let Some(item) = stream.next().await {
        let chunk_bytes = item.context("Error reading stream chunk from LLM engine")?;
        let text = String::from_utf8_lossy(&chunk_bytes);
        buffer.push_str(&text);

        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer.drain(..=pos);

            if line.is_empty() {
                continue;
            }

            if let Ok(parsed) = serde_json::from_str::<StreamChunk>(&line) {
                if let Some(token) = parsed.response {
                    if !token.is_empty() {
                        tokens_generated += 1;

                        if token.contains("<think>") {
                            is_thinking = true;
                            let clean = token.replace("<think>", "");
                            if !clean.is_empty() {
                                on_token(TokenType::Thinking, &clean);
                            }
                        } else if token.contains("</think>") {
                            let clean = token.replace("</think>", "");
                            if !clean.is_empty() {
                                on_token(TokenType::Thinking, &clean);
                            }
                            is_thinking = false;
                        } else {
                            let ttype = if is_thinking { TokenType::Thinking } else { TokenType::Response };
                            on_token(ttype, &token);
                        }
                    }
                }
                if parsed.done == Some(true) {
                    if let Some(ec) = parsed.eval_count {
                        if ec > 0 {
                            tokens_generated = ec;
                        }
                    }
                }
            }
        }
    }

    let elapsed_sec = start.elapsed().as_secs_f64().max(0.001);
    let tok_per_sec = tokens_generated as f64 / elapsed_sec;
    let avail_ram_gb = available_ram_mb() as f64 / 1024.0;

    Ok(ExecutionStats {
        model_name: resolved_model,
        tokens_generated,
        elapsed_sec,
        tok_per_sec,
        avail_ram_gb,
    })
}
