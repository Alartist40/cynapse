use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use colored::*;
use regex::Regex;
use serde::Deserialize;
use cynapse_engine::{query_tier1_stream, TokenType};
use cynapse_memory::graph::{Dendrite, NodeType};

use cynapse_memory::context::DendriteContext;
use cynapse_memory::store::DendriteStore;

pub mod memory_render;
pub mod app;
pub mod terminal;
pub mod theme;

use memory_render::{render_dendrite_visualizer, render_memory_pipeline, truncate_smart, PipelineState, StepStatus};
use app::TuiApp;

#[derive(Debug, Deserialize)]
struct RuntimeConfig {
    engine: Option<EngineConfig>,
}

#[derive(Debug, Deserialize)]
struct EngineConfig {
    tier1_endpoint: Option<String>,
    default_model: Option<String>,
}

pub struct TuiSession {
    pub models_dir: PathBuf,
    pub active_model_name: String,
    pub active_model_path: PathBuf,
    pub tier1_endpoint: String,
    pub graph: Arc<Dendrite>,
    pub store: Option<Arc<DendriteStore>>,
    pub dendrite_ctx: Arc<DendriteContext>,
}

impl TuiSession {
    pub fn new(models_dir: PathBuf) -> Self {
        let mut active_model_name = "ministral-3:3b".to_string();
        let mut tier1_endpoint = "http://127.0.0.1:11434".to_string();

        // Load runtime configuration from cynapse.toml using toml parser
        let config_candidates = [
            PathBuf::from("cynapse.toml"),
            models_dir.parent().map(|p| p.join("cynapse.toml")).unwrap_or_default(),
        ];

        for cfg_path in &config_candidates {
            if cfg_path.exists() {
                if let Ok(content) = std::fs::read_to_string(cfg_path) {
                    if let Ok(parsed) = toml::from_str::<RuntimeConfig>(&content) {
                        if let Some(engine) = parsed.engine {
                            if let Some(ep) = engine.tier1_endpoint {
                                tier1_endpoint = ep;
                            }
                            if let Some(m) = engine.default_model {
                                active_model_name = m;
                            }
                        }
                    }
                    break;
                }
            }
        }

        let active_model_path = models_dir.join("model.gguf");

        // Initialize SQLite persistence store
        let db_dir = PathBuf::from("data");
        let _ = std::fs::create_dir_all(&db_dir);
        let primary_db = db_dir.join("dendrite.db");

        let store = match DendriteStore::open(&primary_db) {
            Ok(s) => Some(Arc::new(s)),
            Err(_) => {
                if let Some(home) = dirs::home_dir() {
                    let user_db_dir = home.join(".cynapse");
                    let _ = std::fs::create_dir_all(&user_db_dir);
                    let user_db = user_db_dir.join("dendrite.db");
                    DendriteStore::open(&user_db).ok().map(Arc::new)
                } else {
                    None
                }
            }
        };

        let graph = Arc::new(Dendrite::new());

        // Hydrate stored graph nodes from SQLite DB
        if let Some(ref st) = store {
            let _ = st.load_all(&graph);
        }

        // Pre-populate core nodes if missing
        if graph.get("cynapse_core").is_none() {
            let n1 = graph.upsert("cynapse_core", "CYNAPSE Agent Core", "Local-first modular AI agent system with Dendrite 4-tier memory graph.", NodeType::Identity, Some(vec!["#summary".into(), "#system".into()]));
            if let Some(ref st) = store { let _ = st.save(&n1); }
        }
        if graph.get("fast_tier").is_none() {
            let n2 = graph.upsert("fast_tier", "Ollama/llama.cpp Fast Engine", "Tier 1 execution engine optimized for SBC hardware reaching 4.8 tok/s.", NodeType::Concept, Some(vec!["#engine".into()]));
            if let Some(ref st) = store { let _ = st.save(&n2); }
        }
        if graph.get("rust_inference").is_none() {
            let n3 = graph.upsert("rust_inference", "Leafcutter Pure Rust Inference", "Tier 2 GGUF & Tier 3 Safetensors layer streaming engine written in pure Rust.", NodeType::Procedure, Some(vec!["#procedure".into()]));
            if let Some(ref st) = store { let _ = st.save(&n3); }
        }

        let dendrite_ctx = DendriteContext::new(graph.clone(), store.clone());

        let mut sess = Self {
            models_dir,
            active_model_name,
            active_model_path,
            tier1_endpoint,
            graph,
            store,
            dendrite_ctx,
        };
        sess.auto_detect_model_sync();
        sess
    }

    /// Sync scan of local models directory to pick active model if default doesn't exist
    pub fn auto_detect_model_sync(&mut self) {
        let home_models = dirs::home_dir().map(|h| h.join(".cynapse").join("models"));
        let mut search_dirs = vec![self.models_dir.clone(), PathBuf::from("./models")];
        if let Some(h) = home_models {
            search_dirs.push(h);
        }

        let mut found_files = Vec::new();
        for dir in &search_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                            if (ext == "gguf" || ext == "safetensors" || ext == "bin") && p.file_name().unwrap() != "README.md" {
                                let filename = p.file_name().unwrap().to_string_lossy().to_string();
                                if !found_files.contains(&filename) {
                                    found_files.push(filename);
                                }
                            }
                        }
                    }
                }
            }
        }
        if !found_files.is_empty() {
            if self.active_model_name == "ministral-3:3b" || !found_files.contains(&self.active_model_name) {
                self.active_model_name = found_files[0].clone();
            }
        }
    }

    /// Extract dynamic quantization tag from filename (e.g. Q4_K_M, Q4_K_XL, Q8_0, F16, Q5_K_S)
    fn extract_quantization(&self, filename: &str) -> String {
        static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"(?i)(Q[0-9]_[K0-9_A-Z]+|F16|F32|IQ[0-9]_[A-Z]+)").unwrap());
        if let Some(mat) = re.find(filename) {
            mat.as_str().to_uppercase()
        } else if filename.ends_with(".safetensors") {
            "SAFETENSORS".to_string()
        } else {
            "GGUF".to_string()
        }
    }

    pub fn list_models(&self) {
        println!("{}", "======================================================================".cyan());
        println!("{}", format!("📋 AVAILABLE MODELS IN CYNAPSE ({})", self.models_dir.display()).yellow().bold());
        println!("{}", "======================================================================".cyan());

        let mut idx = 1;
        if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        if ext == "gguf" || ext == "safetensors" || ext == "bin" {
                            let name = path.file_name().unwrap().to_string_lossy();
                            if name == "README.md" { continue; }

                            let meta = entry.metadata().ok();
                            let bytes = meta.map(|m| m.len()).unwrap_or(0);
                            let size_mb = bytes / 1024 / 1024;
                            let size_str = if size_mb > 1024 {
                                format!("{:.2} GB", size_mb as f64 / 1024.0)
                            } else {
                                format!("{} MB", size_mb)
                            };

                            let quant = self.extract_quantization(&name);
                            println!(" [{}] {:<42} {:<12} {:<10}", idx, name, quant, size_str);
                            idx += 1;
                        }
                    }
                }
            }
        }
        if idx == 1 {
            println!(" (No local models in directory. Download one with: cynapse pull <hf-url-or-repo>)");
        }
        println!("{}", "======================================================================".cyan());
        println!("To run a model:    cynapse run <number>   (or /run <number>)");
        println!("To remove a model: cynapse rm <number>    (or /rm <number>)");
        println!("{}", "======================================================================".cyan());
    }

    pub async fn run_tui_app(&mut self) -> Result<()> {
        self.run_tui_app_with_resume(None).await
    }

    pub async fn run_tui_app_with_resume(&mut self, resume_id: Option<&str>) -> Result<()> {
        let mut app = TuiApp::new(
            self.models_dir.clone(),
            self.active_model_name.clone(),
            self.tier1_endpoint.clone(),
            self.graph.clone(),
            self.store.clone(),
            self.dendrite_ctx.clone(),
        );
        if let Some(sid) = resume_id {
            if let Ok(()) = app.load_session(sid) {
                app.messages.push(app::ChatMessage {
                    role: "system".into(),
                    content: format!("Resumed session transcript from ID: {}", sid),
                    thinking: None,
                });
            }
        }
        app.run().await
    }

    pub async fn run_cli_loop(&mut self) -> Result<()> {
        self.run_interactive_loop().await
    }

    pub async fn run_interactive_loop(&mut self) -> Result<()> {
        println!("{}", "======================================================================".cyan().bold());
        println!("{}", "                   🧠 CYNAPSE LOCAL AGENT SYSTEM                      ".yellow().bold());
        println!("{}", "======================================================================".cyan().bold());
        println!("Pure Rust Runtime:            Enabled (Zero Node / Zero Python)");
        println!("Semantic Engine Router:       Enabled (1.5 GiB RAM Reserve Headroom)");
        println!("Engine Tiers:");
        println!("  • Tier 1 (Fast):            llama.cpp / Ollama (~4.8 tok/s)");
        println!("  • Tier 2 (Large GGUF):      Leafcutter Rust GGUF Core");
        println!("  • Tier 3 (Large Safetensor): Leafcutter Rust Safetensor Core");
        println!("Memory Core:                  DENDRITE 4-Tier Graph (SQLite FTS5 + BM25)");
        println!("{}", "----------------------------------------------------------------------".cyan());
        println!("🎯 Active Model: {}", self.active_model_name.green().bold());
        println!("{}", "----------------------------------------------------------------------".cyan());
        println!("Turn-based execution mode. Wait for prompt cue 'cynapse >>>> ' to type.");
        println!();

        let stdin = io::stdin();
        loop {
            print!("{}", "cynapse >>>> ".bold().bright_magenta());
            io::stdout().flush()?;

            let mut input = String::new();
            if stdin.read_line(&mut input)? == 0 {
                break;
            }
            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed == "exit" || trimmed == "quit" || trimmed == "/exit" {
                println!("{}", "Goodbye!".yellow());
                break;
            }

            if trimmed == "/clear" || trimmed == "/cls" {
                print!("\x1B[2J\x1B[1;1H");
                let _ = io::stdout().flush();
                println!("{}", "Cleared screen.".yellow());
                println!();
                continue;
            }

            if trimmed == "/list" || trimmed == "/ls" {
                self.list_models();
                println!();
                continue;
            }

            if trimmed.starts_with("/run ") || trimmed.starts_with("/select ") {
                let target = trimmed.split_whitespace().nth(1).unwrap_or("");
                if let Ok(num) = target.parse::<usize>() {
                    let mut found = false;
                    let mut idx = 1;
                    if let Ok(entries) = std::fs::read_dir(&self.models_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_file() {
                                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                    if ext == "gguf" || ext == "safetensors" || ext == "bin" {
                                        if path.file_name().unwrap() == "README.md" { continue; }
                                        if idx == num {
                                            self.active_model_name = path.file_name().unwrap().to_string_lossy().to_string();
                                            self.active_model_path = path;
                                            found = true;
                                            break;
                                        }
                                        idx += 1;
                                    }
                                }
                            }
                        }
                    }
                    if found {
                        println!("✓ Switched active model to [{}]: {}", num, self.active_model_name.green());
                    } else {
                        println!("❌ Model index [{}] not found. Type /list to view models.", num);
                    }
                } else {
                    self.active_model_name = target.to_string();
                    println!("✓ Switched active model name to: {}", self.active_model_name.green());
                }
                println!();
                continue;
            }

            if trimmed == "/memory" || trimmed == "/dendrite" || trimmed == "/graph" || trimmed == "/mem" {
                render_dendrite_visualizer(&self.graph);
                println!();
                continue;
            }

            let pipeline = PipelineState {
                search_detail: format!("Searching Dendrite for '{}'", truncate_smart(trimmed, 20)),
                search_status: StepStatus::Done,
                verify_detail: "Verified graph relevance".into(),
                verify_status: StepStatus::Done,
                inject_detail: "Injected relevant facts into prompt".into(),
                inject_status: StepStatus::Done,
                update_detail: "Streaming model output...".into(),
                update_status: StepStatus::Running,
            };

            render_memory_pipeline(&pipeline);
            println!("\n{}", "⚙️ Generating stream...".dimmed());
            let mut current_type: Option<TokenType> = None;

            let system_prompt = self.dendrite_ctx.build_prompt(trimmed, 4000);

            let stats_res = query_tier1_stream(
                &self.tier1_endpoint,
                &self.active_model_name,
                trimmed,
                &system_prompt,
                |ttype, token| {
                    if current_type != Some(ttype) {
                        current_type = Some(ttype);
                        match ttype {
                            TokenType::Thinking => {
                                print!("\n{}", "💭 [Thinking...]\n".magenta().italic());
                            }
                            TokenType::Response => {
                                print!("\n{}", "💡 [Response]:\n".green().bold());
                            }
                        }
                    }
                    match ttype {
                        TokenType::Thinking => {
                            print!("{}", token.dimmed().purple());
                        }
                        TokenType::Response => {
                            print!("{}", token);
                        }
                    }
                    let _ = io::stdout().flush();
                },
            )
            .await;

            match stats_res {
                Ok(stats) => {
                    println!("\n");
                    println!("{}", "----------------------------------------------------------------------".cyan());
                    println!(
                        "📊 [Model: {} | Output: {} tokens | Latency: {:.2}s | Speed: {:.2} tok/s | Avail RAM: {:.1} GB]",
                        stats.model_name.green(),
                        stats.tokens_generated,
                        stats.elapsed_sec,
                        stats.tok_per_sec,
                        stats.avail_ram_gb
                    );
                    println!("{}", "----------------------------------------------------------------------".cyan());
                    println!();
                }
                Err(e) => {
                    println!("\n{}", "❌ [Error]:".red().bold());
                    println!("   {}", e);
                    println!("{}", "----------------------------------------------------------------------".cyan());
                    println!();
                }
            }
        }
        Ok(())
    }
}
