use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

pub mod daemon;
pub mod slots;

/// Dynamic headroom reserve calculation: scales based on model parameter/disk footprint.
pub fn compute_reserve_bytes(model_bytes: u64) -> u64 {
    let model_gb = model_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if model_gb <= 3.0 {
        512 * 1024 * 1024 // 512 MiB for small models (<= 3B)
    } else if model_gb <= 12.0 {
        1536 * 1024 * 1024 // 1.5 GiB for medium models (3B - 12B, e.g. 9B needs ~1.5 GB KV cache)
    } else if model_gb <= 20.0 {
        3072 * 1024 * 1024 // 3.0 GiB for 13B-20B models
    } else {
        8192 * 1024 * 1024 // 8.0 GiB for 35B+ models (KV cache + large activations)
    }
}

pub const DEFAULT_RESERVE_BYTES: u64 = 1536 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineTier {
    Tier1Fast,
    Tier2LargeGguf,
    Tier3LargeSafetensor,
}

impl EngineTier {
    pub fn label(self) -> &'static str {
        match self {
            EngineTier::Tier1Fast => "Tier 1 Fast (llama.cpp)",
            EngineTier::Tier2LargeGguf => "Tier 2 Stream (Leafcutter)",
            EngineTier::Tier3LargeSafetensor => "Tier 3 Stream (Safetensors)",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            EngineTier::Tier1Fast => "Tier 1 Fast",
            EngineTier::Tier2LargeGguf => "Tier 2 Stream",
            EngineTier::Tier3LargeSafetensor => "Tier 3 Stream",
        }
    }
}

/// Provider backend kind for multi-provider resilience and fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderKind {
    LlamaServer,
    Ollama,
    NativeLeafcutter,
}

impl ProviderKind {
    pub fn label(&self) -> &'static str {
        match self {
            ProviderKind::LlamaServer => "llama-server",
            ProviderKind::Ollama => "Ollama",
            ProviderKind::NativeLeafcutter => "Native Leafcutter",
        }
    }
}

/// Dynamic Provider Fallback Chain with escalating cooldown and lazy probe recovery.
#[derive(Debug, Clone)]
pub struct ProviderFallbackChain {
    pub primary: ProviderKind,
    pub fallbacks: Vec<ProviderKind>,
    pub consecutive_failures: usize,
    pub active_override: Option<ProviderKind>,
    pub override_until: Option<Instant>,
    pub last_failure_reason: Option<String>,
}

impl Default for ProviderFallbackChain {
    fn default() -> Self {
        Self::new(
            ProviderKind::LlamaServer,
            vec![ProviderKind::Ollama, ProviderKind::NativeLeafcutter],
        )
    }
}

impl ProviderFallbackChain {
    pub fn new(primary: ProviderKind, fallbacks: Vec<ProviderKind>) -> Self {
        Self {
            primary,
            fallbacks,
            consecutive_failures: 0,
            active_override: None,
            override_until: None,
            last_failure_reason: None,
        }
    }

    /// Resolves the provider to use for the active turn.
    /// Returns (ProviderKind, is_probe).
    pub fn resolve_provider(&self) -> (ProviderKind, bool) {
        if let Some(override_provider) = self.active_override {
            if let Some(until) = self.override_until {
                if Instant::now() < until {
                    return (override_provider, false);
                } else {
                    // Cooldown elapsed -> lazy probe of primary provider
                    return (self.primary, true);
                }
            }
            return (override_provider, false);
        }
        (self.primary, false)
    }

    /// Records a successful turn, resetting failure counters and clearing active overrides.
    pub fn record_success(&mut self, _provider: ProviderKind) {
        self.consecutive_failures = 0;
        self.active_override = None;
        self.override_until = None;
        self.last_failure_reason = None;
    }

    /// Records a provider failure, escalating cooldown (30s -> 60s -> 300s) and setting sticky fallback.
    pub fn record_failure(&mut self, failed_provider: ProviderKind, reason: &str) {
        self.consecutive_failures += 1;
        self.last_failure_reason = Some(reason.to_string());

        let cooldown_secs = match self.consecutive_failures {
            1 => 30,
            2 => 60,
            _ => 300,
        };

        // Pick next fallback provider
        let next_fallback = self.fallbacks
            .iter()
            .find(|&&p| p != failed_provider)
            .copied()
            .unwrap_or(self.primary);

        self.active_override = Some(next_fallback);
        self.override_until = Some(Instant::now() + Duration::from_secs(cooldown_secs));
    }
}

pub fn shared_fallback_chain() -> &'static std::sync::Mutex<ProviderFallbackChain> {
    static CHAIN: std::sync::OnceLock<std::sync::Mutex<ProviderFallbackChain>> = std::sync::OnceLock::new();
    CHAIN.get_or_init(|| std::sync::Mutex::new(ProviderFallbackChain::default()))
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

pub fn total_ram_mb() -> u64 {
    if let Ok(text) = fs::read_to_string("/proc/meminfo") {
        for line in text.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(val) = kb.parse::<u64>() {
                        return val / 1024;
                    }
                }
            }
        }
    }
    16384
}

/// Identifies dedicated or integrated GPU hardware, driver, and acceleration capabilities.
pub fn detect_gpu_info() -> String {
    // 1. Query lspci for VGA/3D display controllers
    if let Ok(output) = std::process::Command::new("lspci").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let lower = line.to_lowercase();
                if lower.contains("vga compatible controller") || lower.contains("3d controller") || lower.contains("display controller") {
                    if let Some(idx) = line.find(':') {
                        let after_first = &line[idx + 1..];
                        let desc = if let Some(idx2) = after_first.find(':') {
                            after_first[idx2 + 1..].trim()
                        } else {
                            after_first.trim()
                        };
                        let clean = desc
                            .trim_start_matches("Advanced Micro Devices, Inc. [AMD/ATI] ")
                            .trim_start_matches("NVIDIA Corporation ")
                            .trim_start_matches("Intel Corporation ");

                        let vulkan_available = Path::new("/usr/bin/vulkaninfo").exists();
                        let vulkan_tag = if vulkan_available { " (Vulkan)" } else { "" };

                        if clean.contains("Cezanne") {
                            return format!("AMD Radeon Vega (Cezanne){}", vulkan_tag);
                        } else if clean.contains("Renoir") {
                            return format!("AMD Radeon Graphics{}", vulkan_tag);
                        } else if clean.contains("Radeon") {
                            let part = clean.split(" [").next().unwrap_or(clean);
                            return format!("AMD {}{}", part, vulkan_tag);
                        } else if clean.len() > 24 {
                            return format!("{}{}", &clean[..24], vulkan_tag);
                        } else {
                            return format!("{}{}", clean, vulkan_tag);
                        }
                    }
                }
            }
        }
    }

    // 2. Sysfs DRM card scanning
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let fname_str = fname.to_string_lossy();
            if fname_str.starts_with("card") && fname_str[4..].chars().all(|c| c.is_ascii_digit()) {
                let uevent_path = entry.path().join("device").join("uevent");
                if let Ok(uevent) = fs::read_to_string(&uevent_path) {
                    if uevent.contains("DRIVER=amdgpu") {
                        return "AMD Radeon Graphics (Vulkan)".to_string();
                    } else if uevent.contains("DRIVER=nvidia") {
                        return "NVIDIA GPU (CUDA / Vulkan)".to_string();
                    } else if uevent.contains("DRIVER=i915") || uevent.contains("DRIVER=xe") {
                        return "Intel Graphics (Vulkan)".to_string();
                    }
                }
            }
        }
    }

    if Path::new("/proc/driver/nvidia/gpus").exists() {
        return "NVIDIA GPU (CUDA)".to_string();
    }

    "CPU Tier (Host RAM)".to_string()
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

    let gpu_info = detect_gpu_info();

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
        .connect_timeout(std::time::Duration::from_millis(250))
        .timeout(std::time::Duration::from_millis(500))
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

pub fn route_model(model_path: &Path, _prefer_gpu: bool) -> RouteDecision {
    let ram_mb = available_ram_mb();
    let total_mb = total_ram_mb();
    let total_bytes = total_mb * 1024 * 1024;

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

    let reserve_bytes = compute_reserve_bytes(model_bytes);
    let needed_bytes = model_bytes.saturating_add(reserve_bytes);
    let model_size_mb = model_bytes as f64 / 1_048_576.0;
    let ram_needed_mb = needed_bytes as f64 / 1_048_576.0;

    let model_gb = model_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    // Models exceeding 85% of host physical RAM capacity (e.g. 35B/70B models > 16GB on a 16GB host)
    // require Tier 2 Leafcutter pure-Rust layer streaming from disk.
    // Models fitting within host RAM capacity (e.g. 0.5B, 3B, 7B, 9B <= 13GB on a 16GB host)
    // route to Tier 1 Fast (llama.cpp / Ollama mmap with GPU offload).
    let max_tier1_bytes = (total_bytes as f64 * 0.85) as u64;
    let tier = if is_safetensors {
        EngineTier::Tier3LargeSafetensor
    } else if (model_gb > 16.0 && needed_bytes > max_tier1_bytes) || model_bytes > max_tier1_bytes {
        EngineTier::Tier2LargeGguf
    } else {
        EngineTier::Tier1Fast
    };

    RouteDecision {
        tier,
        model_size_mb,
        ram_available_mb: ram_mb,
        ram_needed_mb,
        is_safetensors,
    }
}

/// Unified streaming query router: automatically dispatches to Tier 1 (fast llama.cpp/Ollama)
/// or Tier 2 (Leafcutter pure Rust layer streaming) based on the model's routed tier.
pub async fn query_model_stream(
    tier: EngineTier,
    endpoint: &str,
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    match tier {
        EngineTier::Tier1Fast => {
            query_tier1_stream(endpoint, model_name, prompt, system_prompt, on_token).await
        }
        EngineTier::Tier2LargeGguf | EngineTier::Tier3LargeSafetensor => {
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

                match handle.await {
                    Ok(Ok(stats)) => Ok(stats),
                    Ok(Err(e)) => anyhow::bail!("Native Leafcutter Tier 2 streaming error for '{}': {}", model_name, e),
                    Err(join_err) => anyhow::bail!("Native Leafcutter task panicked: {}", join_err),
                }
            } else {
                // Fallback to Tier 1 endpoint if local GGUF file is missing
                query_tier1_stream(endpoint, model_name, prompt, system_prompt, on_token).await
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    pub model_name: String,
    pub tokens_generated: usize,
    pub elapsed_sec: f64,
    pub tok_per_sec: f64,
    pub avail_ram_gb: f64,
}


#[derive(Deserialize, Default)]
struct StreamMessage {
    content: Option<String>,
}

#[derive(Deserialize, Default)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Deserialize, Default)]
struct StreamChoice {
    delta: Option<StreamDelta>,
    text: Option<String>,
}

#[derive(Deserialize, Default)]
struct StreamChunk {
    response: Option<String>,
    content: Option<String>,
    message: Option<StreamMessage>,
    choices: Option<Vec<StreamChoice>>,
    #[allow(dead_code)]
    done: Option<bool>,
    #[allow(dead_code)]
    eval_count: Option<usize>,
}

impl StreamChunk {
    fn extract_token(&self) -> Option<String> {
        if let Some(r) = &self.response {
            if !r.is_empty() {
                return Some(r.clone());
            }
        }
        if let Some(c) = &self.content {
            if !c.is_empty() {
                return Some(c.clone());
            }
        }
        if let Some(m) = &self.message {
            if let Some(c) = &m.content {
                if !c.is_empty() {
                    return Some(c.clone());
                }
            }
        }
        if let Some(choices) = &self.choices {
            if let Some(first) = choices.first() {
                if let Some(d) = &first.delta {
                    if let Some(c) = &d.content {
                        if !c.is_empty() {
                            return Some(c.clone());
                        }
                    }
                }
                if let Some(t) = &first.text {
                    if !t.is_empty() {
                        return Some(t.clone());
                    }
                }
            }
        }
        None
    }
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};

static NATIVE_ENGINE_CACHE: OnceLock<StdMutex<HashMap<String, Arc<leafcutter::api::NativeStreamingEngine>>>> = OnceLock::new();

fn get_or_load_native_engine(path_str: &str) -> Result<Arc<leafcutter::api::NativeStreamingEngine>> {
    let cache = NATIVE_ENGINE_CACHE.get_or_init(|| StdMutex::new(HashMap::new()));
    let mut guard = cache.lock().map_err(|_| anyhow::anyhow!("Engine cache lock poisoned"))?;

    if let Some(engine) = guard.get(path_str) {
        return Ok(Arc::clone(engine));
    }

    let engine = leafcutter::api::NativeStreamingEngine::load(path_str)
        .map_err(|e| anyhow::anyhow!("Failed to load native Leafcutter GGUF engine for {}: {}", path_str, e))?;
    let arc_engine = Arc::new(engine);
    guard.insert(path_str.to_string(), Arc::clone(&arc_engine));
    Ok(arc_engine)
}

/// Checks if a model belongs to a reasoning family that generates thinking tokens
pub fn is_reasoning_model(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("ornith") || lower.contains("qwq") || lower.contains("deepseek") || lower.contains("r1")
}

/// Direct in-process native Leafcutter Rust GGUF stream runner
pub fn query_native_leafcutter_stream(
    model_path: &Path,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    let path_str = model_path.to_string_lossy();
    let engine = get_or_load_native_engine(&path_str)?;

    let full_prompt = if system_prompt.is_empty() {
        prompt.to_string()
    } else {
        format!("<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", system_prompt, prompt)
    };

    let start = Instant::now();
    let is_reasoning = is_reasoning_model(&path_str);
    let mut is_thinking = is_reasoning;

    let (_text, tokens) = engine
        .generate_stream(&full_prompt, 4096, 0.2, 0.95, |token| {
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

pub fn shared_http_client() -> &'static reqwest::Client {
    static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_millis(500))
            .pool_idle_timeout(Some(Duration::from_secs(60)))
            .tcp_keepalive(Some(Duration::from_secs(30)))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

async fn parse_http_stream(
    res: reqwest::Response,
    model_name: &str,
    start: Instant,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    let mut stream = res.bytes_stream();
    let mut tokens_generated = 0usize;
    let is_reasoning = is_reasoning_model(model_name);
    let mut is_thinking = is_reasoning;
    let mut buffer = String::with_capacity(65536);

    while let Some(item) = stream.next().await {
        let chunk_bytes = item.context("Error reading stream chunk from LLM engine")?;
        let text = String::from_utf8_lossy(&chunk_bytes);
        buffer.push_str(&text);

        while let Some(pos) = buffer.find('\n') {
            let mut line = buffer[..pos].trim().to_string();
            buffer.drain(..=pos);

            if line.is_empty() {
                continue;
            }

            if line.starts_with("data:") {
                line = line.trim_start_matches("data:").trim().to_string();
            }

            if line == "[DONE]" {
                break;
            }

            if let Ok(parsed) = serde_json::from_str::<StreamChunk>(&line) {
                if let Some(token) = parsed.extract_token() {
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
            }
        }
    }

    let elapsed_sec = start.elapsed().as_secs_f64().max(0.001);
    let tok_per_sec = tokens_generated as f64 / elapsed_sec;
    let avail_ram_gb = available_ram_mb() as f64 / 1024.0;

    Ok(ExecutionStats {
        model_name: model_name.to_string(),
        tokens_generated: tokens_generated.max(1),
        elapsed_sec,
        tok_per_sec,
        avail_ram_gb,
    })
}

pub async fn try_llama_server_stream(
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    let daemon_port = daemon::DEFAULT_DAEMON_PORT;
    let daemon_ready = if daemon::LlamaServerDaemon::is_healthy(daemon_port) {
        true
    } else if let Some(local_path) = find_model_file_path(model_name) {
        daemon::LlamaServerDaemon::get_or_spawn_daemon(&local_path, daemon_port)
    } else {
        false
    };

    if !daemon_ready {
        anyhow::bail!("llama-server daemon not running and cannot be spawned for {}", model_name);
    }

    let client = shared_http_client();
    let llama_url = format!("http://127.0.0.1:{}/completion", daemon_port);
    let full_p = if system_prompt.is_empty() {
        format!("<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", prompt)
    } else {
        format!("<|im_start|>system\n{}<|im_end|>\n<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n", system_prompt, prompt)
    };
    let payload = serde_json::json!({
        "prompt": full_p,
        "stream": true,
        "cache_prompt": true,
        "keep_alive": -1,
        "slot_id": 0,
        "id_slot": 0,
        "temperature": 0.2,
        "top_p": 0.95,
        "top_k": 40,
        "repeat_penalty": 1.1,
        "repeat_last_n": 256
    });

    let start = Instant::now();
    let resp = client
        .post(&llama_url)
        .header("Accept", "application/x-ndjson, text/event-stream, application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to llama-server daemon")?;

    if !resp.status().is_success() {
        anyhow::bail!("llama-server returned HTTP error: {}", resp.status());
    }

    parse_http_stream(resp, model_name, start, on_token).await
}

pub async fn try_ollama_stream(
    endpoint: &str,
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    let client = shared_http_client();
    let available_tags = fetch_ollama_models(endpoint).await;
    let resolved = resolve_model_tag(model_name, &available_tags);
    let ollama_url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": resolved,
        "prompt": prompt,
        "system": system_prompt,
        "stream": true,
        "cache_prompt": true,
        "keep_alive": -1,
        "slot_id": 0,
        "id_slot": 0,
        "options": {
            "num_ctx": 4096,
            "temperature": 0.2,
            "top_p": 0.95,
            "top_k": 40,
            "repeat_penalty": 1.1,
            "repeat_last_n": 256
        }
    });

    let start = Instant::now();
    let resp = client
        .post(&ollama_url)
        .header("Accept", "application/x-ndjson, text/event-stream, application/json")
        .json(&payload)
        .send()
        .await
        .context("Failed to connect to Ollama endpoint")?;

    if !resp.status().is_success() {
        if resp.status() == reqwest::StatusCode::NOT_FOUND && resolved != model_name {
            let retry_payload = serde_json::json!({
                "model": model_name,
                "prompt": prompt,
                "system": system_prompt,
                "stream": true,
                "cache_prompt": true,
                "slot_id": 0,
                "id_slot": 0,
                "options": {
                    "num_ctx": 4096,
                    "temperature": 0.2,
                    "top_p": 0.95,
                    "top_k": 40,
                    "repeat_penalty": 1.1,
                    "repeat_last_n": 256
                }
            });
            if let Ok(retry_resp) = client
                .post(&ollama_url)
                .header("Accept", "application/x-ndjson, text/event-stream, application/json")
                .json(&retry_payload)
                .send()
                .await
            {
                if retry_resp.status().is_success() {
                    return parse_http_stream(retry_resp, model_name, start, on_token).await;
                }
            }
        }
        anyhow::bail!("Ollama returned HTTP error: {}", resp.status());
    }

    parse_http_stream(resp, model_name, start, on_token).await
}

pub async fn try_native_leafcutter_stream(
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
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

        match handle.await {
            Ok(Ok(stats)) => Ok(stats),
            Ok(Err(e)) => anyhow::bail!("Native Leafcutter engine error for '{}' ({}): {}", model_name, local_path.display(), e),
            Err(join_err) => anyhow::bail!("Native Leafcutter task panicked: {}", join_err),
        }
    } else {
        anyhow::bail!("No local GGUF file found for '{}'", model_name)
    }
}

/// Real token-by-token streaming query runner with dynamic multi-provider fallback.
pub async fn query_tier1_stream(
    endpoint: &str,
    model_name: &str,
    prompt: &str,
    system_prompt: &str,
    mut on_token: impl FnMut(TokenType, &str),
) -> Result<ExecutionStats> {
    let (resolved_provider, _is_probe) = {
        let chain = shared_fallback_chain().lock().map_err(|_| anyhow::anyhow!("Fallback chain lock poisoned"))?;
        chain.resolve_provider()
    };

    let mut candidate_providers = vec![resolved_provider];
    for p in [ProviderKind::LlamaServer, ProviderKind::Ollama, ProviderKind::NativeLeafcutter] {
        if !candidate_providers.contains(&p) {
            candidate_providers.push(p);
        }
    }

    let mut last_error = String::new();

    for provider in candidate_providers {
        let result = match provider {
            ProviderKind::LlamaServer => {
                try_llama_server_stream(model_name, prompt, system_prompt, &mut on_token).await
            }
            ProviderKind::Ollama => {
                try_ollama_stream(endpoint, model_name, prompt, system_prompt, &mut on_token).await
            }
            ProviderKind::NativeLeafcutter => {
                try_native_leafcutter_stream(model_name, prompt, system_prompt, &mut on_token).await
            }
        };

        match result {
            Ok(stats) => {
                if let Ok(mut chain) = shared_fallback_chain().lock() {
                    chain.record_success(provider);
                }
                return Ok(stats);
            }
            Err(e) => {
                let err_msg = e.to_string();
                if let Ok(mut chain) = shared_fallback_chain().lock() {
                    chain.record_failure(provider, &err_msg);
                }
                last_error = err_msg;
            }
        }
    }

    anyhow::bail!(
        "All providers in fallback chain failed for '{}'. Last error: {}",
        model_name,
        last_error
    )
}

/// Send keep_alive: 0 payload to local LLM engine to immediately free memory.
pub async fn unload_model(endpoint: &str, model_name: &str) {
    let client = shared_http_client();
    let url = format!("{}/api/generate", endpoint.trim_end_matches('/'));
    let payload = serde_json::json!({
        "model": model_name,
        "keep_alive": 0
    });
    let _ = client.post(&url).json(&payload).send().await;
}

/// Preload and pin model into memory (keep_alive: -1 for Ollama, or spawn daemon for local GGUF).
pub async fn preload_model(endpoint: &str, model_name: &str) -> Result<()> {
    let client = shared_http_client();
    let endpoint_clean = endpoint.trim_end_matches('/');

    // 1. Check if Ollama is running and pin with keep_alive: -1
    let tags_url = format!("{}/api/tags", endpoint_clean);
    if let Ok(resp) = client.get(&tags_url).timeout(Duration::from_millis(500)).send().await {
        if resp.status().is_success() {
            let available_tags = fetch_ollama_models(endpoint).await;
            let resolved = resolve_model_tag(model_name, &available_tags);
            let gen_url = format!("{}/api/generate", endpoint_clean);
            let payload = serde_json::json!({
                "model": resolved,
                "keep_alive": -1
            });
            let _ = client.post(&gen_url)
                .json(&payload)
                .timeout(Duration::from_secs(30))
                .send()
                .await;
            return Ok(());
        }
    }

    // 2. If daemon can be spawned for local path
    let daemon_port = daemon::DEFAULT_DAEMON_PORT;
    if !daemon::LlamaServerDaemon::is_healthy(daemon_port) {
        if let Some(local_path) = find_model_file_path(model_name) {
            let _ = daemon::LlamaServerDaemon::get_or_spawn_daemon(&local_path, daemon_port);
        }
    }
    Ok(())
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

    #[test]
    fn test_compute_reserve_bytes_scaling() {
        // Small models (<=3B) -> 512 MiB reserve
        assert_eq!(compute_reserve_bytes(500 * 1024 * 1024), 512 * 1024 * 1024);
        // Medium models (3B - 12B) -> 1.5 GiB reserve (e.g. 9B needs ~1.5 GB KV cache)
        assert_eq!(compute_reserve_bytes(5 * 1024 * 1024 * 1024), 1536 * 1024 * 1024);
        // Large 35B+ models -> 8.0 GiB reserve
        assert_eq!(compute_reserve_bytes(21 * 1024 * 1024 * 1024), 8192 * 1024 * 1024);
    }

    #[test]
    fn test_routing_boundaries() {
        // Ornith-9B (5.38 GB) with 16 GB total RAM should comfortably route to Tier 1 Fast
        let model_bytes_9b = (5.38 * 1024.0 * 1024.0 * 1024.0) as u64;
        let total_bytes_16gb = (16.0 * 1024.0 * 1024.0 * 1024.0) as u64;
        let max_tier1_bytes = (total_bytes_16gb as f64 * 0.85) as u64;
        let model_gb_9b = 5.38;
        let needed_bytes_9b = model_bytes_9b + compute_reserve_bytes(model_bytes_9b);

        let tier_9b = if (model_gb_9b > 16.0 && needed_bytes_9b > max_tier1_bytes) || model_bytes_9b > max_tier1_bytes {
            EngineTier::Tier2LargeGguf
        } else {
            EngineTier::Tier1Fast
        };
        assert_eq!(tier_9b, EngineTier::Tier1Fast);

        // Ornith-35B (20.7 GB) with 16 GB total RAM must route to Tier 2 Leafcutter layer streaming
        let model_bytes_35b = (20.7 * 1024.0 * 1024.0 * 1024.0) as u64;
        let model_gb_35b = 20.7;
        let needed_bytes_35b = model_bytes_35b + compute_reserve_bytes(model_bytes_35b);

        let tier_35b = if (model_gb_35b > 16.0 && needed_bytes_35b > max_tier1_bytes) || model_bytes_35b > max_tier1_bytes {
            EngineTier::Tier2LargeGguf
        } else {
            EngineTier::Tier1Fast
        };
        assert_eq!(tier_35b, EngineTier::Tier2LargeGguf);
    }

    #[test]
    fn test_engine_tier_labels() {
        assert_eq!(EngineTier::Tier1Fast.label(), "Tier 1 Fast (llama.cpp)");
        assert_eq!(EngineTier::Tier2LargeGguf.label(), "Tier 2 Stream (Leafcutter)");
        assert_eq!(EngineTier::Tier3LargeSafetensor.label(), "Tier 3 Stream (Safetensors)");
    }

    #[test]
    fn test_provider_fallback_chain() {
        let mut chain = ProviderFallbackChain::new(
            ProviderKind::LlamaServer,
            vec![ProviderKind::Ollama, ProviderKind::NativeLeafcutter],
        );

        // 1. Initially resolves to primary (llama-server)
        let (p, is_probe) = chain.resolve_provider();
        assert_eq!(p, ProviderKind::LlamaServer);
        assert!(!is_probe);

        // 2. On failure, escalates to Ollama with sticky override
        chain.record_failure(ProviderKind::LlamaServer, "Connection refused");
        let (p, is_probe) = chain.resolve_provider();
        assert_eq!(p, ProviderKind::Ollama);
        assert!(!is_probe);
        assert_eq!(chain.consecutive_failures, 1);

        // 3. On success on fallback, resets back to primary
        chain.record_success(ProviderKind::Ollama);
        let (p, is_probe) = chain.resolve_provider();
        assert_eq!(p, ProviderKind::LlamaServer);
        assert!(!is_probe);
        assert_eq!(chain.consecutive_failures, 0);

        // 4. Consecutive failure cooldown escalation
        chain.record_failure(ProviderKind::LlamaServer, "Err 1");
        chain.record_failure(ProviderKind::Ollama, "Err 2");
        assert_eq!(chain.consecutive_failures, 2);
    }

    #[tokio::test]
    async fn test_query_tier1_stream_all_fail() {
        // Querying non-existent model on unreachable port should try chain and return clean Err
        let res = query_tier1_stream(
            "http://127.0.0.1:59999",
            "non-existent-model-12345.gguf",
            "hello",
            "",
            |_ttype, _token| {},
        ).await;

        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("All providers in fallback chain failed"));
    }
}
