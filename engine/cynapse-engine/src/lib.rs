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
struct NativeTagsResp {
    models: Option<Vec<NativeModelItem>>,
}

#[derive(Deserialize)]
struct NativeModelItem {
    name: String,
}

/// Locate absolute GGUF file path on host system
pub fn find_model_file_path(model_name: &str) -> Option<std::path::PathBuf> {
    let p = std::path::PathBuf::from(shellexpand::tilde(model_name).as_ref());
    if p.exists() && p.is_file() {
        return Some(p);
    }

    let mut dirs_to_search = vec![
        std::path::PathBuf::from("./models"),
        std::path::PathBuf::from("../models"),
        std::path::PathBuf::from("models"),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let home_path = std::path::PathBuf::from(&home);
        dirs_to_search.push(home_path.join(".cynapse").join("models"));
        dirs_to_search.push(home_path.join("Downloads").join("models"));
        dirs_to_search.push(home_path.join("Downloads"));
    }

    let lower_model = model_name.to_lowercase();
    let stripped = lower_model.trim_end_matches(".gguf").to_string();

    // 1. Exact filename match
    for dir in &dirs_to_search {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                        let lower_fname = fname.to_lowercase();
                        if lower_fname == lower_model {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    // 2. Base name / fuzzy match
    for dir in &dirs_to_search {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                        let lower_fname = fname.to_lowercase();
                        let stripped_fname = lower_fname.trim_end_matches(".gguf");

                        if stripped_fname == stripped
                            || stripped_fname.contains(&stripped)
                            || stripped.contains(stripped_fname)
                        {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Synchronous model scanner for local GGUF models on disk (no async runtime required)
pub fn fetch_native_models_sync() -> Vec<String> {
    let mut models = Vec::new();

    let mut search_dirs = vec![
        Path::new("./models").to_path_buf(),
        Path::new("../models").to_path_buf(),
        Path::new("models").to_path_buf(),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let home_path = Path::new(&home);
        search_dirs.push(home_path.join(".cynapse").join("models"));
        search_dirs.push(home_path.join("Downloads").join("models"));
        search_dirs.push(home_path.join("Downloads"));
    }

    for dir in &search_dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("gguf") {
                    if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                        if !models.contains(&fname.to_string()) {
                            models.push(fname.to_string());
                        }
                    }
                }
            }
        }
    }
    models
}

/// Fetch list of available models from Cynapse Native Engine endpoint or local GGUF directories
pub async fn fetch_native_models(endpoint: &str) -> Vec<String> {
    let mut models = fetch_native_models_sync();

    // Query endpoint /api/tags if available
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .ok();
    if let Some(c) = client {
        let url = format!("{}/api/tags", endpoint.trim_end_matches('/'));
        if let Ok(res) = c.get(&url).send().await {
            if let Ok(parsed) = res.json::<NativeTagsResp>().await {
                if let Some(endpoint_models) = parsed.models {
                    for m in endpoint_models {
                        if !models.contains(&m.name) {
                            models.push(m.name);
                        }
                    }
                }
            }
        }
    }
    models
}

pub async fn fetch_ollama_models(endpoint: &str) -> Vec<String> {
    fetch_native_models(endpoint).await
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

/// Smart fuzzy model tag resolver matching GGUF model filenames against Ollama/llama-server registered model tags.
pub fn resolve_model_tag(model_name: &str, available_tags: &[String]) -> String {
    if available_tags.is_empty() {
        return model_name.to_string();
    }

    let lower_model = model_name.to_lowercase();
    let stripped = lower_model.trim_end_matches(".gguf").to_string();

    // 1. Exact match (case-insensitive)
    for tag in available_tags {
        let lower_tag = tag.to_lowercase();
        if lower_tag == lower_model || lower_tag == stripped {
            return tag.clone();
        }
    }

    // 2. Tag base name matching (e.g. "ornith:9b" -> base "ornith")
    for tag in available_tags {
        let tag_base = tag.split(':').next().unwrap_or(tag).to_lowercase();
        let clean_base = tag_base.split('/').last().unwrap_or(&tag_base);
        if !clean_base.is_empty() && (stripped.contains(clean_base) || clean_base.contains(&stripped)) {
            return tag.clone();
        }
    }

    // 3. Main model keyword token matching
    let keywords = [
        "ornith", "qwen", "ministral", "mistral", "llama", "gemma", "phi", "deepseek",
        "smollm", "starcoder", "command", "granite", "internlm", "baichuan", "chatglm",
        "minimax", "falcon", "yi", "nemotron", "cohere", "ocr", "nomic",
    ];
    for kw in keywords {
        if stripped.contains(kw) {
            if let Some(matched) = available_tags.iter().find(|t| t.to_lowercase().contains(kw)) {
                return matched.clone();
            }
        }
    }

    // 4. Token overlap scoring fallback
    let stripped_tokens: Vec<&str> = stripped.split(['-', '_', '.', ':', '/']).filter(|s| !s.is_empty()).collect();
    let mut best_match: Option<(&String, usize)> = None;

    for tag in available_tags {
        let tag_lower = tag.to_lowercase();
        let tag_tokens: Vec<&str> = tag_lower.split(['-', '_', '.', ':', '/']).filter(|s| !s.is_empty()).collect();
        let mut score = 0;
        for st in &stripped_tokens {
            if tag_tokens.contains(st) {
                score += 1;
            }
        }
        if score > 0 {
            if let Some((_, best_score)) = best_match {
                if score > best_score {
                    best_match = Some((tag, score));
                }
            } else {
                best_match = Some((tag, score));
            }
        }
    }

    if let Some((best_tag, _)) = best_match {
        return best_tag.clone();
    }

    model_name.to_string()
}

/// Direct in-process native Leafcutter Rust GGUF stream runner
pub fn query_native_leafcutter_stream(
    model_path: &Path,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    use leafcutter::api::NativeStreamingEngine;

    let path_str = model_path.to_string_lossy();
    let engine = NativeStreamingEngine::load(&path_str)
        .map_err(|e| anyhow::anyhow!("Failed to load native Leafcutter GGUF engine for {}: {}", model_path.display(), e))?;

    let full_prompt = if system_prompt.is_empty() {
        prompt.to_string()
    } else {
        format!("<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", system_prompt, prompt)
    };

    let start = Instant::now();
    let mut is_thinking = false;

    let (_text, tokens) = engine
        .generate_stream(&full_prompt, 2048, 0.7, 0.9, |token| {
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
                on_token(ttype, token);
            }
        })
        .map_err(|e| anyhow::anyhow!("Native Leafcutter generation error: {}", e))?;

    let elapsed_sec = start.elapsed().as_secs_f64().max(0.001);
    let tokens_generated = tokens.len().max(1);
    let tok_per_sec = tokens_generated as f64 / elapsed_sec;
    let avail_ram_gb = available_ram_mb() as f64 / 1024.0;

    Ok(ExecutionStats {
        model_name: model_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        tokens_generated,
        elapsed_sec,
        tok_per_sec,
        avail_ram_gb,
    })
}

/// Real token-by-token streaming query runner (native Leafcutter runner or Tier 1 HTTP endpoint).
pub async fn query_tier1_stream(
    endpoint: &str,
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    // 1. Direct native Leafcutter execution if GGUF file exists on disk
    if let Some(local_path) = find_model_file_path(model_name) {
        let p = local_path.clone();
        let pr = prompt.to_string();
        let sys = system_prompt.to_string();

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(TokenType, String)>();
        let handle = tokio::task::spawn_blocking(move || {
            query_native_leafcutter_stream(&p, &pr, &sys, |ttype, text| {
                let _ = tx.send((ttype, text.to_string()));
            })
        });

        while let Some((ttype, text)) = rx.recv().await {
            on_token(ttype, &text);
        }

        if let Ok(res) = handle.await {
            if let Ok(stats) = res {
                return Ok(stats);
            }
        }
    }

    let client = reqwest::Client::new();
    let start = Instant::now();
    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));

    // Resolve model tag from Ollama endpoint if available
    let available_tags = fetch_ollama_models(endpoint).await;
    let resolved_model = resolve_model_tag(model_name, &available_tags);

    let payload = serde_json::json!({
        "model": resolved_model,
        "prompt": prompt,
        "system": system_prompt,
        "stream": true,
        "options": {
            "num_ctx": 4096,
            "temperature": 0.7
        }
    });

    let mut attempt = 0;
    let max_attempts = 3;
    let mut delay_ms = 200u64;

    let mut res = loop {
        attempt += 1;
        let req_builder = client.post(&url).json(&payload);

        match req_builder.send().await {
            Ok(resp) if resp.status().is_success() => break resp,
            Ok(resp) if resp.status().is_server_error() && attempt < max_attempts => {
                let jitter = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0) % 100) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms + jitter)).await;
                delay_ms *= 2;
            }
            Ok(resp) => break resp,
            Err(_err) if attempt < max_attempts => {
                let jitter = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(0) % 100) as u64;
                tokio::time::sleep(std::time::Duration::from_millis(delay_ms + jitter)).await;
                delay_ms *= 2;
            }
            Err(err) => {
                anyhow::bail!("Unable to connect to local LLM engine at {}: {}", endpoint, err);
            }
        }
    };

    // Retry once with raw model_name if resolved tag returned 404
    if res.status() == reqwest::StatusCode::NOT_FOUND && resolved_model != model_name {
        let retry_payload = serde_json::json!({
            "model": model_name,
            "prompt": prompt,
            "system": system_prompt,
            "stream": true,
            "options": {
                "num_ctx": 4096,
                "temperature": 0.7
            }
        });

        if let Ok(retry_res) = client
            .post(&url)
            .json(&retry_payload)
            .send()
            .await
        {
            if retry_res.status().is_success() {
                res = retry_res;
            }
        }
    }

    if !res.status().is_success() {
        let status = res.status();
        let err_body = res.text().await.unwrap_or_default();
        let clean_err = err_body.trim();

        if status == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!(
                "Model '{}' not found in local Cynapse engine catalog (HTTP 404).\nAvailable models: [{}]\nPlace GGUF models into ./models/ or download via Cynapse TUI.",
                model_name, available_tags.join(", ")
            );
        }

        if !clean_err.is_empty() {
            anyhow::bail!("Local LLM engine returned HTTP error {}: {}", status, clean_err);
        } else {
            anyhow::bail!("Local LLM engine returned HTTP error: {}", status);
        }
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

/// Send keep_alive: 0 payload to local LLM engine to immediately free memory.
pub async fn unload_model(endpoint: &str, model_name: &str) {
    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model_name,
        "keep_alive": 0
    });
    let _ = client.post(&url).json(&payload).send().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_model_tag_fuzzy() {
        let available = vec![
            "nomic-embed-text-v2-moe:latest".to_string(),
            "ministral-3:3b".to_string(),
            "frob/unlimited-ocr:f16".to_string(),
            "frob/unlimited-ocr:q8_0".to_string(),
            "ornith:9b".to_string(),
        ];

        // User case: Ornith-1.5-9B-Q4_K_M.gguf -> ornith:9b
        assert_eq!(
            resolve_model_tag("Ornith-1.5-9B-Q4_K_M.gguf", &available),
            "ornith:9b"
        );

        // Case: ministral-8b-instruct-q4_k_m.gguf -> ministral-3:3b
        assert_eq!(
            resolve_model_tag("ministral-8b-instruct-q4_k_m.gguf", &available),
            "ministral-3:3b"
        );

        // Case: exact match
        assert_eq!(
            resolve_model_tag("nomic-embed-text-v2-moe:latest", &available),
            "nomic-embed-text-v2-moe:latest"
        );

        // Case: unlisted model stays unchanged
        assert_eq!(
            resolve_model_tag("unknown-custom-model.gguf", &available),
            "unknown-custom-model.gguf"
        );
    }
}
