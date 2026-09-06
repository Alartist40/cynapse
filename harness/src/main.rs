use std::path::PathBuf;
use anyhow::Result;
use clap::{Parser, Subcommand};
use cynapse_core::pull_huggingface_model;
use cynapse_engine::route_model;
use cynapse_tui::TuiSession;

#[derive(Parser)]
#[command(name = "cynapse")]
#[command(about = "Pure Rust AI Agent System: 3-tier LLM engine, REPL TUI, atomic-agent local tools, and Dendrite graph memory.")]
struct Cli {
    /// Force full visual Ratatui TUI mode
    #[arg(long)]
    tui: bool,

    /// Force simple line-by-line CLI text mode
    #[arg(long)]
    cli: bool,

    /// Resume a past conversation session by ID
    #[arg(long)]
    resume: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List downloaded models in models directory
    List,
    /// Select and run a model by number or name
    Run {
        target: Option<String>,
    },
    /// Route model execution through Semantic Hardware Tier Router
    Route {
        model_path: Option<PathBuf>,
    },
    /// Pull model from HuggingFace
    Pull {
        url_or_repo: String,
    },
    /// Render visual 4-tier Dendrite memory overview & graph topology
    Memory,
    /// Run self-healing Cynapse Doctor system diagnostic & recovery
    Doctor {
        /// Enable automatic self-healing repair mode
        #[arg(long)]
        fix: bool,
    },
}

/// Resolve models directory dynamically relative to executable, workspace, or user home folder.
fn resolve_models_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("./models"),
        dirs::home_dir().map(|h| h.join(".cynapse").join("models")).unwrap_or_default(),
    ];
    for cand in &candidates {
        if cand.exists() {
            return cand.clone();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("models");
            if candidate.exists() {
                return candidate;
            }
        }
    }
    if let Some(home) = dirs::home_dir() {
        let home_models = home.join(".cynapse").join("models");
        let _ = std::fs::create_dir_all(&home_models);
        return home_models;
    }
    PathBuf::from("models")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let models_dir = resolve_models_dir();
    std::fs::create_dir_all(&models_dir)?;

    let mut session = TuiSession::new(models_dir.clone());

    match cli.command {
        Some(Commands::List) => {
            session.list_models();
        }
        Some(Commands::Route { model_path }) => {
            let target = model_path.unwrap_or_else(|| models_dir.join("model.gguf"));
            let decision = route_model(&target, false);
            println!("======================================================================");
            println!("             🧠 CYNAPSE SEMANTIC HARDWARE & MODEL ROUTER              ");
            println!("======================================================================");
            println!("Target Model Path:  {}", target.display());
            println!("Model Disk Size:    {:.2} MB", decision.model_size_mb);
            println!("Available Host RAM: {} MB", decision.ram_available_mb);
            println!("Memory Needed:      {:.2} MB (Model + 1.5GB Headroom Reserve)", decision.ram_needed_mb);
            println!("----------------------------------------------------------------------");
            println!("SELECTED ENGINE TIER: {}", decision.tier.label());
            println!("======================================================================");
        }
        Some(Commands::Pull { url_or_repo }) => {
            let target_path = pull_huggingface_model(&url_or_repo, &models_dir).await?;
            println!("✓ Model successfully downloaded to: {}", target_path.display());
        }
        Some(Commands::Memory) => {
            let db_path = dirs::home_dir().map(|h| h.join(".cynapse").join("dendrite.db")).unwrap_or_else(|| PathBuf::from("data/dendrite.db"));
            if db_path.exists() {
                if let Ok(store) = cynapse_memory::store::DendriteStore::open(&db_path) {
                    let _ = store.load_all(&session.graph);
                }
            }
            cynapse_tui::memory_render::render_dendrite_visualizer(&session.graph);
        }
        Some(Commands::Doctor { fix }) => {
            let db_path = dirs::home_dir().map(|h| h.join(".cynapse").join("dendrite.db")).unwrap_or_else(|| PathBuf::from("data/dendrite.db"));
            let doctor = cynapse_core::doctor::CynapseDoctor::new(models_dir.clone(), db_path, fix);
            let report = doctor.run_diagnostics();

            println!("======================================================================");
            println!("           🩺 CYNAPSE AGENT SYSTEM SELF-HEALING DOCTOR                ");
            println!("======================================================================");
            println!("Overall System Health Score: [ {}% ]", report.health_score);
            println!("Summary: {} Pass | {} Warning | {} Repaired | {} Failed", report.total_pass, report.total_warn, report.total_repaired, report.total_fail);
            println!("----------------------------------------------------------------------");
            for item in &report.items {
                println!("{}{} [{:<14}] {:<35} - {}", item.status.color_code(), item.status.badge(), item.subsystem, item.check_name, item.detail);
                if let Some(fix) = &item.fix_recommendation {
                    println!("    └─> Fix Recommendation: {}", fix);
                }
                print!("\x1b[0m"); // reset color
            }
            println!("======================================================================");
            if report.total_fail > 0 && !fix {
                println!("💡 Tip: Run 'cynapse doctor --fix' to execute automatic self-healing repairs.");
            }
        }
        Some(Commands::Run { target }) => {
            if let Some(t) = target {
                session.active_model_name = t;
            }
            if cli.cli {
                session.run_cli_loop_with_resume(cli.resume.as_deref()).await?;
            } else {
                session.run_tui_app_with_resume(cli.resume.as_deref()).await?;
            }
        }
        None => {
            if cli.cli {
                session.run_cli_loop_with_resume(cli.resume.as_deref()).await?;
            } else {
                session.run_tui_app_with_resume(cli.resume.as_deref()).await?;
            }
        }
    }

    Ok(())
}
