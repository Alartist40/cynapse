//! Cynapse Curated Model Catalog, Hardware Recommendation Engine, & HuggingFace Downloader.
//!
//! Inspired by atomic-agent model installer architecture. Provides curated GGUF model definitions,
//! automatic host RAM/CPU hardware recommendations, custom HuggingFace URL parsing, quantization selection,
//! and async progress streaming into local models storage.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedModelDef {
    pub id: &'static str,
    pub name: &'static str,
    pub repo_url: &'static str,
    pub filename: &'static str,
    pub size_str: &'static str,
    pub min_ram_gb: usize,
    pub recommended_ram_gb: usize,
    pub description: &'static str,
    pub family: &'static str,
}

pub const CURATED_MODELS_CATALOG: &[CuratedModelDef] = &[
    CuratedModelDef {
        id: "qwen2.5-0.5b",
        name: "Qwen 2.5 0.5B Instruct (Ultra Fast)",
        repo_url: "Qwen/Qwen2.5-0.5B-Instruct-GGUF",
        filename: "qwen2.5-0.5b-instruct-q4_k_m.gguf",
        size_str: "398 MB",
        min_ram_gb: 2,
        recommended_ram_gb: 4,
        description: "Ultra-fast low memory footprint model for basic tasks",
        family: "qwen",
    },
    CuratedModelDef {
        id: "qwen2.5-1.5b",
        name: "Qwen 2.5 1.5B Instruct (Balanced)",
        repo_url: "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        filename: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        size_str: "980 MB",
        min_ram_gb: 4,
        recommended_ram_gb: 6,
        description: "Fast multi-lingual reasoning with low latency",
        family: "qwen",
    },
    CuratedModelDef {
        id: "qwen2.5-3b",
        name: "Qwen 2.5 3B Instruct (Recommended)",
        repo_url: "Qwen/Qwen2.5-3B-Instruct-GGUF",
        filename: "qwen2.5-3b-instruct-q4_k_m.gguf",
        size_str: "2.0 GB",
        min_ram_gb: 6,
        recommended_ram_gb: 8,
        description: "High quality instruction following & code generation",
        family: "qwen",
    },
    CuratedModelDef {
        id: "ministral-8b",
        name: "Ministral 8B Instruct",
        repo_url: "mistralai/Ministral-8B-Instruct-2410",
        filename: "ministral-8b-instruct-q4_k_m.gguf",
        size_str: "4.9 GB",
        min_ram_gb: 10,
        recommended_ram_gb: 16,
        description: "High performance dense model with long context window",
        family: "mistral",
    },
    CuratedModelDef {
        id: "qwen2.5-7b",
        name: "Qwen 2.5 7B Instruct",
        repo_url: "Qwen/Qwen2.5-7B-Instruct-GGUF",
        filename: "qwen2.5-7b-instruct-q4_k_m.gguf",
        size_str: "4.7 GB",
        min_ram_gb: 12,
        recommended_ram_gb: 16,
        description: "Advanced reasoning, multi-turn chat, and complex coding",
        family: "qwen",
    },
    CuratedModelDef {
        id: "gemma-4-12b",
        name: "Gemma 4 12B Instruct QAT",
        repo_url: "unsloth/gemma-4-12B-it-qat-GGUF",
        filename: "gemma-4-12B-it-qat-UD-Q4_K_XL.gguf",
        size_str: "6.7 GB",
        min_ram_gb: 16,
        recommended_ram_gb: 24,
        description: "Deep reasoning model with quantization-aware training",
        family: "gemma",
    },
];

/// Returns index of best recommended curated model for host system RAM.
pub fn recommend_model_for_hardware(total_ram_mb: u64) -> usize {
    let ram_gb = (total_ram_mb / 1024) as usize;
    if ram_gb <= 4 {
        0 // Qwen 0.5B
    } else if ram_gb <= 8 {
        2 // Qwen 3B
    } else if ram_gb <= 16 {
        4 // Qwen 7B
    } else {
        5 // Gemma 12B
    }
}

/// Resolves a custom HuggingFace repo/URL string and quantization into a direct download URL and filename.
pub fn resolve_hf_download_url(input: &str, quant: &str) -> (String, String) {
    let clean = input.trim().trim_end_matches('/');

    if clean.ends_with(".gguf") || clean.ends_with(".safetensors") {
        let filename = Path::new(clean).file_name().unwrap_or_default().to_string_lossy().to_string();
        let url = if clean.starts_with("http://") || clean.starts_with("https://") {
            clean.to_string()
        } else {
            format!("https://huggingface.co/{}", clean)
        };
        (url, filename)
    } else {
        let repo_path = clean
            .trim_start_matches("https://huggingface.co/")
            .trim_start_matches("http://huggingface.co/");
        
        let repo_name = repo_path.split('/').last().unwrap_or("model");
        let filename = format!("{}-{}.gguf", repo_name, quant.to_lowercase());
        let url = format!("https://huggingface.co/{}/resolve/main/{}", repo_path, filename);
        (url, filename)
    }
}

pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_mbps: f64,
    pub pct: f64,
}

/// Stream downloads HuggingFace model with real-time progress callbacks.
pub async fn stream_download_hf_model<F>(
    download_url: &str,
    target_path: &Path,
    mut progress_cb: F,
) -> Result<PathBuf>
where
    F: FnMut(DownloadProgress),
{
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let client = reqwest::Client::new();
    let res = client
        .get(download_url)
        .header("User-Agent", "Cynapse-Agent-Downloader/1.2")
        .send()
        .await
        .with_context(|| format!("Failed to connect to HuggingFace URL: {}", download_url))?;

    if !res.status().is_success() {
        anyhow::bail!("HuggingFace server returned HTTP status: {}", res.status());
    }

    let total_bytes = res.content_length().unwrap_or(0);
    let mut file = File::create(target_path)
        .with_context(|| format!("Failed to create destination file: {}", target_path.display()))?;

    let mut stream = res.bytes_stream();
    let mut downloaded_bytes = 0u64;
    let start_time = Instant::now();
    let mut last_emit = Instant::now();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.context("Error downloading chunk from stream")?;
        file.write_all(&bytes)?;
        downloaded_bytes += bytes.len() as u64;

        if last_emit.elapsed().as_millis() > 100 || downloaded_bytes == total_bytes {
            last_emit = Instant::now();
            let elapsed_sec = start_time.elapsed().as_secs_f64().max(0.001);
            let speed_mbps = (downloaded_bytes as f64 / 1_048_576.0) / elapsed_sec;
            let pct = if total_bytes > 0 {
                (downloaded_bytes as f64 / total_bytes as f64) * 100.0
            } else {
                0.0
            };

            progress_cb(DownloadProgress {
                downloaded_bytes,
                total_bytes,
                speed_mbps,
                pct,
            });
        }
    }

    Ok(target_path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_recommendations() {
        assert_eq!(recommend_model_for_hardware(4096), 0);  // 4GB -> Qwen 0.5B
        assert_eq!(recommend_model_for_hardware(8192), 2);  // 8GB -> Qwen 3B
        assert_eq!(recommend_model_for_hardware(16384), 4); // 16GB -> Qwen 7B
    }

    #[test]
    fn test_resolve_hf_url() {
        let (url, filename) = resolve_hf_download_url("TheBloke/Llama-2-7B-GGUF", "Q4_K_M");
        assert_eq!(url, "https://huggingface.co/TheBloke/Llama-2-7B-GGUF/resolve/main/Llama-2-7B-GGUF-q4_k_m.gguf");
        assert_eq!(filename, "Llama-2-7B-GGUF-q4_k_m.gguf");
    }
}
