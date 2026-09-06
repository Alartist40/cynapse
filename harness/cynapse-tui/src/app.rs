//! Cynapse Ratatui Visual TUI Application — Inspired by jcode & colibri.
//!
//! Features Colibri-style Left Sidebar layout, system hardware telemetry (RAM/CPU/GPU),
//! Jcode-style background ASCII art in chat viewport, smooth rounded borders (BorderType::Rounded),
//! RAII terminal protection, non-blocking Tokio MPSC token streaming, dynamic model auto-detection,
//! paragraph text wrapping, interactive slash command dropdown, theme presets, session persistence,
//! clean prompt input, and 3D Galaxy Memory Atlas visualizer.

use std::fs;
use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::EnterAlternateScreen,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use tokio::sync::mpsc;

use cynapse_core::offline_agent::{validate_gbnf_tool_call, LoopGuard};
use cynapse_core::session::{SessionData, SessionManager, SessionMessage};
use cynapse_engine::{fetch_ollama_models, probe_hardware_info, query_tier1_stream, SystemHardwareInfo, TokenType};
use cynapse_memory::context::DendriteContext;
use cynapse_memory::graph::{Dendrite, NodeType};
use cynapse_memory::store::DendriteStore;
use crate::terminal::TuiRuntimeGuard;
use crate::theme::AppTheme;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub thinking: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Help,
    MemoryGraph,
    MemoryDrawer,
    ModelList,
    ModelPuller,
    SessionList,
    Doctor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PullerStep {
    CuratedList,
    CustomInput,
    QuantSelect,
    Downloading,
}

pub const QUANT_OPTIONS: &[&str] = &["Q4_K_M", "Q5_K_M", "Q8_0", "F16"];

#[derive(Debug, Clone)]
pub struct DownloadProgressState {
    pub model_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_mbps: f64,
    pub pct: f64,
    pub is_done: bool,
    pub error: Option<String>,
}

pub enum StreamEvent {
    Token { ttype: TokenType, text: String },
    Done { tok_per_sec: f64, elapsed_sec: f64 },
    Error(String),
}

#[derive(Debug, Clone)]
pub struct ModelItem {
    pub name: String,
    pub source: String, // "Local File" or "Leafcutter Engine"
    pub quant: String,
    pub size_str: String,
}

pub struct SlashCommand {
    pub name: &'static str,
    pub description: &'static str,
}

pub const SLASH_COMMANDS: &[SlashCommand] = &[
    SlashCommand { name: "/help", description: "Display keyboard shortcuts & help menu" },
    SlashCommand { name: "/model", description: "Open interactive model selector" },
    SlashCommand { name: "/pull", description: "Download GGUF model from HuggingFace (hardware curated)" },
    SlashCommand { name: "/doctor", description: "Run self-healing Cynapse Doctor system diagnostic & recovery" },
    SlashCommand { name: "/memory", description: "View 3D Galaxy Memory Atlas topology" },
    SlashCommand { name: "/drawer", description: "Open interactive Dendrite Memory drawer inspector" },
    SlashCommand { name: "/thinking", description: "Toggle collapsible model thinking/reasoning blocks" },
    SlashCommand { name: "/theme", description: "Cycle visual color theme (Dark Slate, Neon, Amber, Matrix)" },
    SlashCommand { name: "/session", description: "Open saved sessions manager" },
    SlashCommand { name: "/clear", description: "Clear conversation history" },
    SlashCommand { name: "/exit", description: "Exit Cynapse TUI" },
];

pub const ASCII_BANNER: &[&str] = &[
    "                                                          ",
    "                              +####+.                     ",
    "                            =##***=:..::                  ",
    "                            +#****::..::.                 ",
    "                    -=-     ##*****=..:                   ",
    "                   :+***: :   +####*+#=                   ",
    "                -=-  -++       +#     ++=+:               ",
    "              =+==--: :-       =:     +##%%#=             ",
    "              =##++##:  .      -     :######+             ",
    "          +@@@@#++*=     :+####=  :==*#####*:             ",
    "          @@@@@@        .#####*### ...+++++=..            ",
    "           %@@=:..:=+-..++++##*##%+  .::......            ",
    "             .......:++--##****###   .........            ",
    "             ........=:  #***+*#*-     .   ..             ",
    "              ......---:-    -=:   =- :**++++             ",
    "              .......      =*=.:+***-.=+=-::::.           ",
    "             .........     +--===****===    ..-           ",
    "            .....  .::      --=++**+==**:....--           ",
    "             ..:=====::.    -=*++****. .=+.++%@#          ",
    "              .=====**=++ .  :-***=-       *@@@@@         ",
    "               =***+**                  +++*%%%%*         ",
    "               =@@@%%+      -      .  +*: .##.            ",
    "                 #@#*:     =+       =:      -:            ",
    "                     .:  :*#+      -=+:                   ",
    "                     ::..-***#*+=   +**=                  ",
    "                    .:....=***#.                          ",
    "                     .:.:=***#+                           ",
    "                       -#####=                            ",
];

pub struct TuiApp {
    pub models_dir: PathBuf,
    pub active_model_name: String,
    pub active_model_quant: String,
    pub active_model_size: String,
    pub active_model_source: String,
    pub tier1_endpoint: String,
    pub graph: Arc<Dendrite>,
    pub store: Option<Arc<DendriteStore>>,
    pub dendrite_ctx: Arc<DendriteContext>,
    pub input: String,
    pub input_cursor: usize,
    pub messages: Vec<ChatMessage>,
    pub modal: ActiveModal,
    pub theme: AppTheme,
    pub is_generating: bool,
    pub last_tok_per_sec: f64,
    pub last_latency_sec: f64,
    pub scroll_offset: u16,
    pub last_max_scroll: AtomicU16,
    pub auto_scroll: bool,
    pub session_mgr: SessionManager,
    pub session_id: String,
    pub stream_rx: Option<mpsc::UnboundedReceiver<StreamEvent>>,
    pub current_thinking_buf: String,
    pub current_response_buf: String,

    // Navigation & Animation states
    pub autocomplete_idx: usize,
    pub selected_model_idx: usize,
    pub selected_session_idx: usize,
    pub selected_memory_idx: usize,
    pub anim_tick: usize,

    // Hardware Telemetry
    pub hw_info: SystemHardwareInfo,

    // 3D Galaxy Camera state
    pub galaxy_yaw: f32,
    pub galaxy_pitch: f32,
    pub galaxy_auto_spin: bool,
    pub show_thinking: bool,
    pub loop_guard: LoopGuard,
    pub agent_step_count: usize,

    // Model Downloader state
    pub puller_step: PullerStep,
    pub selected_pull_idx: usize,
    pub custom_pull_url: String,
    pub custom_pull_cursor: usize,
    pub selected_quant_idx: usize,
    pub download_progress_rx: Option<mpsc::UnboundedReceiver<DownloadProgressState>>,
    pub current_download_state: Option<DownloadProgressState>,

    // Doctor Self-Healing Diagnostic state
    pub doctor_report: Option<cynapse_core::doctor::DoctorReport>,
}

impl TuiApp {
    pub fn new(
        models_dir: PathBuf,
        active_model_name: String,
        tier1_endpoint: String,
        graph: Arc<Dendrite>,
        store: Option<Arc<DendriteStore>>,
        dendrite_ctx: Arc<DendriteContext>,
    ) -> Self {
        let session_mgr = SessionManager::new();
        let session_id = SessionManager::generate_id();
        let hw_info = probe_hardware_info();
        Self {
            models_dir,
            active_model_name,
            active_model_quant: "Q4_K_M".into(),
            active_model_size: "398 MB".into(),
            active_model_source: "Local File".into(),
            tier1_endpoint,
            graph,
            store,
            dendrite_ctx,
            input: String::new(),
            input_cursor: 0,
            messages: vec![ChatMessage {
                role: "system".into(),
                content: "Welcome to CYNAPSE — Pure Rust AI Agent System with Dendrite Graph Memory.".into(),
                thinking: None,
            }],
            modal: ActiveModal::None,
            theme: AppTheme::AmberCRT,
            is_generating: false,
            last_tok_per_sec: 4.8,
            last_latency_sec: 0.0,
            scroll_offset: 0,
            last_max_scroll: AtomicU16::new(0),
            auto_scroll: true,
            show_thinking: true,
            session_mgr,
            session_id,
            stream_rx: None,
            current_thinking_buf: String::new(),
            current_response_buf: String::new(),
            autocomplete_idx: 0,
            selected_model_idx: 0,
            selected_session_idx: 0,
            selected_memory_idx: 0,
            anim_tick: 0,
            hw_info,
            galaxy_yaw: 0.4,
            galaxy_pitch: 0.3,
            galaxy_auto_spin: true,
            loop_guard: LoopGuard::default(),
            agent_step_count: 0,
            puller_step: PullerStep::CuratedList,
            selected_pull_idx: 0,
            custom_pull_url: String::new(),
            custom_pull_cursor: 0,
            selected_quant_idx: 0,
            download_progress_rx: None,
            current_download_state: None,
            doctor_report: None,
        }
    }

    /// Auto-detect available models on disk and Leafcutter engine, setting a valid active model.
    pub async fn auto_detect_model(&mut self) {
        let scanned = self.scan_all_models().await;
        if scanned.is_empty() {
            if self.active_model_name.is_empty() || self.active_model_name == "ministral-3:3b" {
                self.active_model_name = "(No model loaded — use /pull)".into();
            }
            return;
        }

        // Prioritize local .gguf / .safetensors file in models_dir if active model is default "ministral-3:3b" or not on disk
        let local_matches = scanned.iter().find(|m| m.source == "Local File");
        if self.active_model_name == "ministral-3:3b" || !scanned.iter().any(|m| m.name == self.active_model_name) {
            if let Some(local) = local_matches {
                self.active_model_name = local.name.clone();
                self.active_model_quant = local.quant.clone();
                self.active_model_size = local.size_str.clone();
                self.active_model_source = local.source.clone();
                return;
            }
        }

        // Otherwise update metadata for current active model name if present
        if let Some(found) = scanned.iter().find(|m| m.name == self.active_model_name) {
            self.active_model_quant = found.quant.clone();
            self.active_model_size = found.size_str.clone();
            self.active_model_source = found.source.clone();
        } else if let Some(first) = scanned.first() {
            self.active_model_name = first.name.clone();
            self.active_model_quant = first.quant.clone();
            self.active_model_size = first.size_str.clone();
            self.active_model_source = first.source.clone();
        }
    }

    pub fn scroll_up(&mut self, delta: u16) {
        let max_scroll = self.last_max_scroll.load(Ordering::Relaxed);
        if self.auto_scroll {
            self.scroll_offset = max_scroll;
            self.auto_scroll = false;
        }
        self.scroll_offset = self.scroll_offset.saturating_sub(delta);
    }

    pub fn scroll_down(&mut self, delta: u16) {
        let max_scroll = self.last_max_scroll.load(Ordering::Relaxed);
        if self.auto_scroll {
            return;
        }
        self.scroll_offset = self.scroll_offset.saturating_add(delta);
        if self.scroll_offset >= max_scroll {
            self.scroll_offset = max_scroll;
            self.auto_scroll = true;
        }
    }

    pub fn delete_word_backward(&mut self) {
        if self.input_cursor == 0 || self.input.is_empty() {
            return;
        }
        let text_before = &self.input[..self.input_cursor];
        let mut chars: Vec<(usize, char)> = text_before.char_indices().collect();
        if chars.is_empty() {
            return;
        }

        // 1. Skip trailing whitespace
        while let Some(&(_, ch)) = chars.last() {
            if ch.is_whitespace() {
                chars.pop();
            } else {
                break;
            }
        }

        // 2. Delete word characters
        while let Some(&(_, ch)) = chars.last() {
            if !ch.is_whitespace() {
                chars.pop();
            } else {
                break;
            }
        }

        let target_idx = chars.last().map(|&(idx, ch)| idx + ch.len_utf8()).unwrap_or(0);
        self.input.drain(target_idx..self.input_cursor);
        self.input_cursor = target_idx;
    }

    fn execute_tool_and_format(&mut self, call: &cynapse_core::offline_agent::ToolCall) -> (String, bool) {
        let name = &call.name;
        let args = &call.arguments;

        let arg1 = args.get("path")
            .or_else(|| args.get("query"))
            .or_else(|| args.get("command"))
            .or_else(|| args.get("arg1"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let arg2 = args.get("content")
            .or_else(|| args.get("dir"))
            .or_else(|| args.get("arg2"))
            .and_then(|v| v.as_str());

        match cynapse_core::execute_tool(name, arg1, arg2) {
            Ok(output) => (output, true),
            Err(e) => (format!("Tool execution error: {}", e), false),
        }
    }

    pub fn load_session(&mut self, session_id: &str) -> Result<()> {
        self.autocomplete_idx = 0;
        let data = self.session_mgr.load_session(session_id)?;
        self.session_id = data.session_id;
        self.active_model_name = data.model_name;
        self.messages = data
            .messages
            .into_iter()
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content,
                thinking: m.thinking,
            })
            .collect();
        Ok(())
    }

    pub fn save_current_session(&self) {
        let session_msgs = self
            .messages
            .iter()
            .map(|m| SessionMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                thinking: m.thinking.clone(),
            })
            .collect();

        let data = SessionData {
            session_id: self.session_id.clone(),
            created_at: 0,
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            model_name: self.active_model_name.clone(),
            messages: session_msgs,
        };

        let _ = self.session_mgr.save_session(&data);
    }

    pub async fn run(&mut self) -> Result<()> {
        self.auto_detect_model().await;

        let _guard = TuiRuntimeGuard::enter()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        self.event_loop(&mut terminal).await
    }

    pub fn start_hf_download(&mut self, download_url: String, target_filename: String) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.download_progress_rx = Some(rx);
        self.puller_step = PullerStep::Downloading;
        self.modal = ActiveModal::ModelPuller;

        let models_dir = self.models_dir.clone();
        let target_path = models_dir.join(&target_filename);

        tokio::spawn(async move {
            let res = cynapse_core::downloader::stream_download_hf_model(
                &download_url,
                &target_path,
                |prog| {
                    let _ = tx.send(DownloadProgressState {
                        model_name: target_filename.clone(),
                        downloaded_bytes: prog.downloaded_bytes,
                        total_bytes: prog.total_bytes,
                        speed_mbps: prog.speed_mbps,
                        pct: prog.pct,
                        is_done: false,
                        error: None,
                    });
                },
            )
            .await;

            match res {
                Ok(_path) => {
                    let _ = tx.send(DownloadProgressState {
                        model_name: target_filename,
                        downloaded_bytes: 0,
                        total_bytes: 0,
                        speed_mbps: 0.0,
                        pct: 100.0,
                        is_done: true,
                        error: None,
                    });
                }
                Err(e) => {
                    let _ = tx.send(DownloadProgressState {
                        model_name: target_filename,
                        downloaded_bytes: 0,
                        total_bytes: 0,
                        speed_mbps: 0.0,
                        pct: 0.0,
                        is_done: true,
                        error: Some(e.to_string()),
                    });
                }
            }
        });
    }

    fn poll_download_events(&mut self) {
        let mut latest = None;
        if let Some(rx) = &mut self.download_progress_rx {
            while let Ok(evt) = rx.try_recv() {
                latest = Some(evt);
            }
        }
        if let Some(st) = latest {
            if st.is_done {
                if let Some(err) = &st.error {
                    self.messages.push(ChatMessage {
                        role: "error".into(),
                        content: format!("Model Download Failed: {}", err),
                        thinking: None,
                    });
                } else {
                    self.active_model_name = st.model_name.clone();
                    self.messages.push(ChatMessage {
                        role: "system".into(),
                        content: format!("✓ Download Complete: Saved and activated model '{}'", st.model_name),
                        thinking: None,
                    });
                }
                self.download_progress_rx = None;
                self.modal = ActiveModal::None;
            }
            self.current_download_state = Some(st);
        }
    }

    async fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            self.anim_tick += 1;
            if self.anim_tick % 30 == 0 {
                self.hw_info = probe_hardware_info();
            }
            if self.modal == ActiveModal::MemoryGraph && self.galaxy_auto_spin {
                self.galaxy_yaw += 0.05;
            }

            self.poll_stream_events();
            self.poll_download_events();

            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(30))? {
                match event::read()? {
                    Event::Key(key) => {
                        // Global Interrupt
                        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                            self.save_current_session();
                            break;
                        }

                        // Handle Modal Inputs
                        if self.modal != ActiveModal::None {
                            if self.modal == ActiveModal::ModelPuller {
                                match self.puller_step {
                                    PullerStep::CuratedList => match key.code {
                                        KeyCode::Esc | KeyCode::Char('q') => {
                                            self.modal = ActiveModal::None;
                                        }
                                        KeyCode::Up => {
                                            let cat_len = cynapse_core::downloader::CURATED_MODELS_CATALOG.len();
                                            if self.selected_pull_idx > 0 {
                                                self.selected_pull_idx -= 1;
                                            } else {
                                                self.selected_pull_idx = cat_len.saturating_sub(1);
                                            }
                                        }
                                        KeyCode::Down => {
                                            let cat_len = cynapse_core::downloader::CURATED_MODELS_CATALOG.len();
                                            if self.selected_pull_idx + 1 < cat_len {
                                                self.selected_pull_idx += 1;
                                            } else {
                                                self.selected_pull_idx = 0;
                                            }
                                        }
                                        KeyCode::Char('c') | KeyCode::Tab => {
                                            self.puller_step = PullerStep::CustomInput;
                                            self.custom_pull_url.clear();
                                            self.custom_pull_cursor = 0;
                                        }
                                        KeyCode::Enter => {
                                            let cat = cynapse_core::downloader::CURATED_MODELS_CATALOG;
                                            if !cat.is_empty() {
                                                let idx = self.selected_pull_idx.min(cat.len() - 1);
                                                let item = &cat[idx];
                                                let (url, filename) = cynapse_core::downloader::resolve_hf_download_url(item.repo_url, "Q4_K_M");
                                                self.start_hf_download(url, filename);
                                            }
                                        }
                                        _ => {}
                                    },
                                    PullerStep::CustomInput => match key.code {
                                        KeyCode::Esc => {
                                            self.puller_step = PullerStep::CuratedList;
                                        }
                                        KeyCode::Left => {
                                            self.custom_pull_cursor = self.custom_pull_cursor.saturating_sub(1);
                                        }
                                        KeyCode::Right => {
                                            if self.custom_pull_cursor < self.custom_pull_url.len() {
                                                self.custom_pull_cursor += 1;
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            self.custom_pull_url.insert(self.custom_pull_cursor, c);
                                            self.custom_pull_cursor += 1;
                                        }
                                        KeyCode::Backspace => {
                                            if self.custom_pull_cursor > 0 {
                                                self.custom_pull_url.remove(self.custom_pull_cursor - 1);
                                                self.custom_pull_cursor -= 1;
                                            }
                                        }
                                        KeyCode::Enter => {
                                            if !self.custom_pull_url.trim().is_empty() {
                                                self.selected_quant_idx = 0;
                                                self.puller_step = PullerStep::QuantSelect;
                                            }
                                        }
                                        _ => {}
                                    },
                                    PullerStep::QuantSelect => match key.code {
                                        KeyCode::Esc => {
                                            self.puller_step = PullerStep::CustomInput;
                                        }
                                        KeyCode::Up => {
                                            self.selected_quant_idx = self.selected_quant_idx.saturating_sub(1);
                                        }
                                        KeyCode::Down => {
                                            if self.selected_quant_idx + 1 < QUANT_OPTIONS.len() {
                                                self.selected_quant_idx += 1;
                                            }
                                        }
                                        KeyCode::Enter => {
                                            let quant = QUANT_OPTIONS[self.selected_quant_idx.min(QUANT_OPTIONS.len() - 1)];
                                            let (url, filename) = cynapse_core::downloader::resolve_hf_download_url(&self.custom_pull_url, quant);
                                            self.start_hf_download(url, filename);
                                        }
                                        _ => {}
                                    },
                                    PullerStep::Downloading => match key.code {
                                        KeyCode::Esc | KeyCode::Char('q') => {
                                            self.modal = ActiveModal::None;
                                        }
                                        _ => {}
                                    },
                                }
                                continue;
                            }

                            match key.code {
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Tab => {
                                    self.modal = ActiveModal::None;
                                }
                                KeyCode::Left => match self.modal {
                                    ActiveModal::MemoryGraph => {
                                        self.galaxy_yaw -= 0.15;
                                    }
                                    _ => {}
                                },
                                KeyCode::Right => match self.modal {
                                    ActiveModal::MemoryGraph => {
                                        self.galaxy_yaw += 0.15;
                                    }
                                    _ => {}
                                },
                                KeyCode::Up => match self.modal {
                                    ActiveModal::MemoryGraph => {
                                        self.galaxy_pitch -= 0.15;
                                    }
                                    ActiveModal::MemoryDrawer => {
                                        self.selected_memory_idx = self.selected_memory_idx.saturating_sub(1);
                                    }
                                    ActiveModal::ModelList => {
                                        self.selected_model_idx = self.selected_model_idx.saturating_sub(1);
                                    }
                                    ActiveModal::SessionList => {
                                        self.selected_session_idx = self.selected_session_idx.saturating_sub(1);
                                    }
                                    _ => {}
                                },
                                KeyCode::Down => match self.modal {
                                    ActiveModal::MemoryGraph => {
                                        self.galaxy_pitch += 0.15;
                                    }
                                    ActiveModal::MemoryDrawer => {
                                        self.selected_memory_idx = self.selected_memory_idx.saturating_add(1);
                                    }
                                    ActiveModal::ModelList => {
                                        self.selected_model_idx = self.selected_model_idx.saturating_add(1);
                                    }
                                    ActiveModal::SessionList => {
                                        self.selected_session_idx = self.selected_session_idx.saturating_add(1);
                                    }
                                    _ => {}
                                },
                                KeyCode::Char('d') | KeyCode::Delete => match self.modal {
                                    ActiveModal::MemoryDrawer => {
                                        let nodes = self.graph.all();
                                        if !nodes.is_empty() {
                                            let idx = self.selected_memory_idx.min(nodes.len() - 1);
                                            let target_id = nodes[idx].id.clone();
                                            self.graph.delete(&target_id);
                                            if let Some(store) = &self.store {
                                                let _ = store.delete(&target_id);
                                            }
                                            self.messages.push(ChatMessage {
                                                role: "system".into(),
                                                content: format!("Deleted memory node: [[{}]]", target_id),
                                                thinking: None,
                                            });
                                        }
                                    }
                                    _ => {}
                                },
                                KeyCode::Char('s') | KeyCode::Char(' ') => match self.modal {
                                    ActiveModal::MemoryGraph => {
                                        self.galaxy_auto_spin = !self.galaxy_auto_spin;
                                    }
                                    _ => {}
                                },
                                KeyCode::Char('r') | KeyCode::F(5) => match self.modal {
                                    ActiveModal::Doctor => {
                                        let db_path = dirs::home_dir().map(|h| h.join(".cynapse").join("dendrite.db")).unwrap_or_else(|| PathBuf::from("data/dendrite.db"));
                                        let doctor = cynapse_core::doctor::CynapseDoctor::new(self.models_dir.clone(), db_path, true);
                                        self.doctor_report = Some(doctor.run_diagnostics());
                                    }
                                    _ => {}
                                },
                                KeyCode::Enter => match self.modal {
                                    ActiveModal::ModelList => {
                                        let scanned = self.scan_models_sync();
                                        if !scanned.is_empty() {
                                            let idx = self.selected_model_idx.min(scanned.len() - 1);
                                            self.active_model_name = scanned[idx].name.clone();
                                            self.active_model_quant = scanned[idx].quant.clone();
                                            self.active_model_size = scanned[idx].size_str.clone();
                                            self.active_model_source = scanned[idx].source.clone();
                                            self.messages.push(ChatMessage {
                                                role: "system".into(),
                                                content: format!("Switched active model to: {}", self.active_model_name),
                                                thinking: None,
                                            });
                                        }
                                        self.modal = ActiveModal::None;
                                    }
                                    ActiveModal::SessionList => {
                                        let sessions = self.session_mgr.list_sessions();
                                        if !sessions.is_empty() {
                                            let idx = self.selected_session_idx.min(sessions.len() - 1);
                                            let sid = sessions[idx].session_id.clone();
                                            let _ = self.load_session(&sid);
                                        }
                                        self.modal = ActiveModal::None;
                                    }
                                    _ => {
                                        self.modal = ActiveModal::None;
                                    }
                                },
                                _ => {}
                            }
                            continue;
                        }

                        // Autocomplete Dropdown Navigation
                        let matching_cmds = self.get_matching_commands();
                        let is_autocomplete_active = self.input.starts_with('/') && !self.input.contains(' ') && !matching_cmds.is_empty();

                        if is_autocomplete_active {
                            match key.code {
                                KeyCode::Up => {
                                    if self.autocomplete_idx > 0 {
                                        self.autocomplete_idx -= 1;
                                    } else {
                                        self.autocomplete_idx = matching_cmds.len().saturating_sub(1);
                                    }
                                    continue;
                                }
                                KeyCode::Down => {
                                    if self.autocomplete_idx + 1 < matching_cmds.len() {
                                        self.autocomplete_idx += 1;
                                    } else {
                                        self.autocomplete_idx = 0;
                                    }
                                    continue;
                                }
                                KeyCode::Tab | KeyCode::Right => {
                                    if let Some(cmd) = matching_cmds.get(self.autocomplete_idx) {
                                        self.input = format!("{} ", cmd.name);
                                        self.input_cursor = self.input.len();
                                        self.autocomplete_idx = 0;
                                    }
                                    continue;
                                }
                                KeyCode::Enter => {
                                    if let Some(cmd) = matching_cmds.get(self.autocomplete_idx) {
                                        self.input = cmd.name.to_string();
                                        self.input_cursor = self.input.len();
                                        self.autocomplete_idx = 0;
                                    }
                                }
                                _ => {}
                            }
                        }

                        // Control Key Shortcuts
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('w') | KeyCode::Char('h') | KeyCode::Backspace => {
                                    self.delete_word_backward();
                                    continue;
                                }
                                KeyCode::Char('t') => {
                                    self.show_thinking = !self.show_thinking;
                                    continue;
                                }
                                KeyCode::Char('a') => {
                                    self.input_cursor = 0;
                                    continue;
                                }
                                KeyCode::Char('e') => {
                                    self.input_cursor = self.input.len();
                                    continue;
                                }
                                KeyCode::Char('u') => {
                                    self.input.clear();
                                    self.input_cursor = 0;
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        // General Input Field & Viewport Navigation Handling
                        match key.code {
                            KeyCode::Tab => {
                                if !self.input.starts_with('/') {
                                    self.selected_memory_idx = 0;
                                    self.modal = ActiveModal::MemoryDrawer;
                                }
                            }
                            KeyCode::Esc => {
                                self.input.clear();
                                self.input_cursor = 0;
                                self.modal = ActiveModal::None;
                            }
                            KeyCode::Left => {
                                self.input_cursor = self.input_cursor.saturating_sub(1);
                            }
                            KeyCode::Right => {
                                if self.input_cursor < self.input.len() {
                                    self.input_cursor += 1;
                                }
                            }
                            KeyCode::Home => {
                                self.input_cursor = 0;
                            }
                            KeyCode::End => {
                                self.input_cursor = self.input.len();
                            }
                            KeyCode::Char(c) => {
                                self.input.insert(self.input_cursor, c);
                                self.input_cursor += 1;
                                self.autocomplete_idx = 0;
                            }
                            KeyCode::Backspace => {
                                if self.input_cursor > 0 {
                                    self.input.remove(self.input_cursor - 1);
                                    self.input_cursor -= 1;
                                }
                                self.autocomplete_idx = 0;
                            }
                            KeyCode::Delete => {
                                if self.input_cursor < self.input.len() {
                                    self.input.remove(self.input_cursor);
                                }
                                self.autocomplete_idx = 0;
                            }
                            KeyCode::PageUp => {
                                self.scroll_up(5);
                            }
                            KeyCode::PageDown => {
                                self.scroll_down(5);
                            }
                            KeyCode::Up => {
                                self.scroll_up(1);
                            }
                            KeyCode::Down => {
                                self.scroll_down(1);
                            }
                            KeyCode::Enter => {
                                let trimmed = self.input.trim().to_string();
                                if trimmed.is_empty() {
                                    continue;
                                }
                                self.input.clear();
                                self.input_cursor = 0;

                                // Execute Slash Commands
                                if trimmed == "/exit" || trimmed == "exit" || trimmed == "quit" {
                                    self.save_current_session();
                                    break;
                                }

                                if trimmed == "/clear" || trimmed == "/cls" {
                                    self.messages.clear();
                                    self.scroll_offset = 0;
                                    self.auto_scroll = true;
                                    continue;
                                }

                                if trimmed == "/help" {
                                    self.modal = ActiveModal::Help;
                                    continue;
                                }

                                if trimmed == "/theme" {
                                    self.theme = self.theme.next();
                                    self.messages.push(ChatMessage {
                                        role: "system".into(),
                                        content: format!("Switched visual theme to: {}", self.theme.name()),
                                        thinking: None,
                                    });
                                    continue;
                                }

                                if trimmed == "/thinking" {
                                    self.show_thinking = !self.show_thinking;
                                    self.messages.push(ChatMessage {
                                        role: "system".into(),
                                        content: format!("Model reasoning/thinking blocks are now: {}", if self.show_thinking { "Expanded (Visible)" } else { "Collapsed (Hidden)" }),
                                        thinking: None,
                                    });
                                    continue;
                                }

                                if trimmed == "/pull" || trimmed == "/download" {
                                    let rec_idx = cynapse_core::downloader::recommend_model_for_hardware(self.hw_info.ram_total_mb);
                                    self.selected_pull_idx = rec_idx;
                                    self.puller_step = PullerStep::CuratedList;
                                    self.modal = ActiveModal::ModelPuller;
                                    continue;
                                }

                                if trimmed.starts_with("/pull ") || trimmed.starts_with("/download ") {
                                    let arg = trimmed.split_whitespace().nth(1).unwrap_or("");
                                    if !arg.is_empty() {
                                        self.custom_pull_url = arg.to_string();
                                        self.custom_pull_cursor = self.custom_pull_url.len();
                                        self.selected_quant_idx = 0;
                                        self.puller_step = PullerStep::QuantSelect;
                                        self.modal = ActiveModal::ModelPuller;
                                    }
                                    continue;
                                }

                                if trimmed == "/session" || trimmed == "/sessions" {
                                    self.selected_session_idx = 0;
                                    self.modal = ActiveModal::SessionList;
                                    continue;
                                }

                                if trimmed == "/doctor" || trimmed == "/doc" || trimmed == "/heal" {
                                    let db_path = dirs::home_dir().map(|h| h.join(".cynapse").join("dendrite.db")).unwrap_or_else(|| PathBuf::from("data/dendrite.db"));
                                    let doctor = cynapse_core::doctor::CynapseDoctor::new(self.models_dir.clone(), db_path, true);
                                    self.doctor_report = Some(doctor.run_diagnostics());
                                    self.modal = ActiveModal::Doctor;
                                    continue;
                                }

                                if trimmed == "/drawer" || trimmed == "/inspector" {
                                    self.selected_memory_idx = 0;
                                    self.modal = ActiveModal::MemoryDrawer;
                                    continue;
                                }

                                if trimmed == "/memory" || trimmed == "/dendrite" || trimmed == "/graph" || trimmed == "/mem" {
                                    self.modal = ActiveModal::MemoryGraph;
                                    continue;
                                }

                                if trimmed == "/model" || trimmed == "/list" || trimmed == "/ls" {
                                    self.selected_model_idx = 0;
                                    self.modal = ActiveModal::ModelList;
                                    continue;
                                }

                                if trimmed.starts_with("/model ") || trimmed.starts_with("/run ") {
                                    let arg = trimmed.split_whitespace().nth(1).unwrap_or("");
                                    if !arg.is_empty() {
                                        self.active_model_name = arg.to_string();
                                        self.messages.push(ChatMessage {
                                            role: "system".into(),
                                            content: format!("Active model updated to: {}", self.active_model_name),
                                            thinking: None,
                                        });
                                    }
                                    continue;
                                }

                                // User Prompt Execution
                                self.messages.push(ChatMessage {
                                    role: "user".into(),
                                    content: trimmed.clone(),
                                    thinking: None,
                                });

                                self.agent_step_count = 0;
                                self.is_generating = true;
                                self.current_thinking_buf.clear();
                                self.current_response_buf.clear();
                                self.auto_scroll = true; // Lock scroll to bottom for incoming response

                                // Build System Prompt with Dendrite Memory Injection
                                let system_prompt = self.dendrite_ctx.build_prompt(&trimmed, 4000);

                                // Spawn Non-blocking Async LLM Task
                                let (tx, rx) = mpsc::unbounded_channel();
                                self.stream_rx = Some(rx);

                                let endpoint = self.tier1_endpoint.clone();
                                let model_name = self.active_model_name.clone();
                                let prompt = trimmed.clone();

                                tokio::spawn(async move {
                                    let res = query_tier1_stream(
                                        &endpoint,
                                        &model_name,
                                        &prompt,
                                        &system_prompt,
                                        |ttype, token| {
                                            let _ = tx.send(StreamEvent::Token {
                                                ttype,
                                                text: token.to_string(),
                                            });
                                        },
                                    )
                                    .await;

                                    match res {
                                        Ok(stats) => {
                                            let _ = tx.send(StreamEvent::Done {
                                                tok_per_sec: stats.tok_per_sec,
                                                elapsed_sec: stats.elapsed_sec,
                                            });
                                        }
                                        Err(e) => {
                                            let _ = tx.send(StreamEvent::Error(e.to_string()));
                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
                    }
                    Event::Mouse(mouse) => {
                        match mouse.kind {
                            MouseEventKind::ScrollUp => {
                                self.scroll_up(2);
                            }
                            MouseEventKind::ScrollDown => {
                                self.scroll_down(2);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn poll_stream_events(&mut self) {
        let mut events = Vec::new();
        if let Some(rx) = &mut self.stream_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }

        let mut finished = false;
        for event in events {
            match event {
                StreamEvent::Token { ttype, text } => match ttype {
                    TokenType::Thinking => self.current_thinking_buf.push_str(&text),
                    TokenType::Response => self.current_response_buf.push_str(&text),
                },
                StreamEvent::Done { tok_per_sec, elapsed_sec } => {
                    self.last_tok_per_sec = tok_per_sec;
                    self.last_latency_sec = elapsed_sec;

                    let mut triggered_reprompt = false;
                    const MAX_AGENT_STEPS: usize = 5;

                    // Offline Agent: GBNF tool call check & circular LoopGuard intervention
                    if let Ok(tool_call) = validate_gbnf_tool_call(&self.current_response_buf) {
                        if self.agent_step_count >= MAX_AGENT_STEPS {
                            self.messages.push(ChatMessage {
                                role: "system".into(),
                                content: format!("⚠️ MAX AGENT STEPS REACHED ({} steps): Automated tool execution loop paused to prevent runaway execution.", MAX_AGENT_STEPS),
                                thinking: None,
                            });
                        } else {
                            match self.loop_guard.record_and_check(&tool_call) {
                                Ok(()) => {
                                    self.agent_step_count += 1;
                                    let (tool_output, ok) = self.execute_tool_and_format(&tool_call);
                                    self.messages.push(ChatMessage {
                                        role: "system".into(),
                                        content: format!("🔧 Tool Call [{}] Executed (Step {}/{}):\n{}", tool_call.name, self.agent_step_count, MAX_AGENT_STEPS, tool_output),
                                        thinking: None,
                                    });

                                    if ok {
                                        triggered_reprompt = true;
                                        let (tx, rx) = mpsc::unbounded_channel();
                                        self.stream_rx = Some(rx);
                                        self.is_generating = true;
                                        self.current_thinking_buf.clear();
                                        self.current_response_buf.clear();

                                        let user_msg = self.messages.iter().rev().find(|m| m.role == "user").map(|m| m.content.as_str()).unwrap_or("").to_string();
                                        let system_prompt = self.dendrite_ctx.build_prompt(&tool_output, 4000);
                                        let endpoint = self.tier1_endpoint.clone();
                                        let model_name = self.active_model_name.clone();
                                        let prompt = if user_msg.is_empty() {
                                            format!("Tool Result for {}:\n{}\n\nContinue resolution.", tool_call.name, tool_output)
                                        } else {
                                            format!("User Request: {}\n\nTool Result for {}:\n{}\n\nContinue resolution.", user_msg, tool_call.name, tool_output)
                                        };

                                        tokio::spawn(async move {
                                            let res = query_tier1_stream(
                                                &endpoint,
                                                &model_name,
                                                &prompt,
                                                &system_prompt,
                                                |ttype, token| {
                                                    let _ = tx.send(StreamEvent::Token { ttype, text: token.to_string() });
                                                },
                                            )
                                            .await;

                                            match res {
                                                Ok(stats) => {
                                                    let _ = tx.send(StreamEvent::Done {
                                                        tok_per_sec: stats.tok_per_sec,
                                                        elapsed_sec: stats.elapsed_sec,
                                                    });
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(StreamEvent::Error(e.to_string()));
                                                }
                                            }
                                        });
                                    }
                                }
                                Err(loop_warn) => {
                                    self.messages.push(ChatMessage {
                                        role: "system".into(),
                                        content: loop_warn,
                                        thinking: None,
                                    });
                                }
                            }
                        }
                    }

                    if !triggered_reprompt {
                        self.messages.push(ChatMessage {
                            role: "assistant".into(),
                            content: self.current_response_buf.clone(),
                            thinking: if self.current_thinking_buf.is_empty() {
                                None
                            } else {
                                Some(self.current_thinking_buf.clone())
                            },
                        });

                        // Store Turn Log Node into Dendrite Graph & SQLite Store
                        let user_msg = self.messages.iter().rev().find(|m| m.role == "user").map(|m| m.content.clone()).unwrap_or_default();
                        if !user_msg.is_empty() {
                            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
                            let turn_id = format!("turn_{}", now);
                            let excerpt = if user_msg.len() > 25 { format!("{}...", &user_msg[..25]) } else { user_msg.clone() };
                            let title = format!("Turn: {}", excerpt);
                            let content = format!("User: {}\n\nAssistant: {}", user_msg, self.current_response_buf);

                            let node = self.graph.upsert(&turn_id, &title, &content, NodeType::TurnLog, Some(vec!["#conversation".into(), "#turn".into()]));
                            if let Some(ref st) = self.store {
                                let _ = st.save(&node);
                            }

                            // Automatic Atomic Fact & Topic Extraction
                            let lower_msg = user_msg.to_lowercase();
                            if lower_msg.contains("favourite") || lower_msg.contains("favorite") || lower_msg.contains("love") || lower_msg.contains("like") || lower_msg.contains("remember") || lower_msg.contains("is ") {
                                let fact_id = format!("fact_{}", now);
                                let fact_title = format!("User Fact: {}", excerpt);
                                let fact_content = format!("User preference / fact: {}", user_msg);

                                let fact_node = self.graph.upsert(&fact_id, &fact_title, &fact_content, NodeType::AtomicFact, Some(vec!["#preference".into(), "#user_fact".into(), "#memory".into()]));
                                if let Some(ref st) = self.store {
                                    let _ = st.save(&fact_node);
                                }
                            }
                        }

                        self.is_generating = false;
                    }
                }
                StreamEvent::Error(err) => {
                    self.messages.push(ChatMessage {
                        role: "error".into(),
                        content: format!("Error querying engine: {}", err),
                        thinking: None,
                    });
                    self.is_generating = false;
                    finished = true;
                }
            }
        }
        if finished {
            self.stream_rx = None;
        }
    }

    fn get_matching_commands(&self) -> Vec<&'static SlashCommand> {
        if !self.input.starts_with('/') {
            return Vec::new();
        }
        SLASH_COMMANDS
            .iter()
            .filter(|cmd| cmd.name.starts_with(&self.input))
            .collect()
    }

    async fn scan_all_models(&self) -> Vec<ModelItem> {
        let mut items = self.scan_models_sync();
        let ollama_models = fetch_ollama_models(&self.tier1_endpoint).await;
        for name in ollama_models {
            if !items.iter().any(|i| i.name == name) {
                items.push(ModelItem {
                    name,
                    source: "Leafcutter Engine".into(),
                    quant: "GGUF".into(),
                    size_str: "Loaded".into(),
                });
            }
        }
        items
    }

    fn scan_models_sync(&self) -> Vec<ModelItem> {
        let mut items = Vec::new();
        static GGUF_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = GGUF_RE.get_or_init(|| {
            regex::Regex::new(r"(?i)(Q[0-9]_[K0-9_A-Z]+|F16|F32|IQ[0-9]_[A-Z]+)").unwrap()
        });

        let mut search_dirs = vec![
            self.models_dir.clone(),
            PathBuf::from("./models"),
        ];
        if let Some(home) = dirs::home_dir() {
            search_dirs.push(home.join(".cynapse").join("models"));
        }

        for dir in &search_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            if ext == "gguf" || ext == "safetensors" || ext == "bin" {
                                let filename = path.file_name().unwrap().to_string_lossy();
                                if filename == "README.md" || items.iter().any(|i: &ModelItem| i.name == filename) {
                                    continue;
                                }

                                let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                                let size_mb = bytes / 1024 / 1024;
                                let size_str = if size_mb > 1024 {
                                    format!("{:.2} GB", size_mb as f64 / 1024.0)
                                } else {
                                    format!("{} MB", size_mb)
                                };

                                let quant = if let Some(mat) = re.find(&filename) {
                                    mat.as_str().to_uppercase()
                                } else if filename.ends_with(".safetensors") {
                                    "SAFETENSORS".to_string()
                                } else {
                                    "GGUF".to_string()
                                };

                                items.push(ModelItem {
                                    name: filename.to_string(),
                                    source: "Local File".into(),
                                    quant,
                                    size_str,
                                });
                            }
                        }
                    }
                }
            }
        }
        items
    }

pub fn render_markdown_lines(text: &str, theme: AppTheme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();

    for raw_line in text.lines() {
        let trimmed = raw_line.trim_start();
        if trimmed.starts_with("```") {
            if in_code_block {
                lines.push(Line::from(Span::styled("    └───", Style::default().fg(Color::Cyan))));
                in_code_block = false;
                code_lang.clear();
            } else {
                in_code_block = true;
                code_lang = trimmed.trim_start_matches('`').trim().to_string();
                let lang_label = if code_lang.is_empty() { "CODE".to_string() } else { code_lang.to_uppercase() };
                lines.push(Line::from(Span::styled(format!("    ┌── [ {} ] ────────────────────────────────────────", lang_label), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
            }
            continue;
        }

        if in_code_block {
            lines.push(Line::from(vec![
                Span::styled("    │ ", Style::default().fg(Color::Cyan)),
                Span::styled(raw_line.to_string(), Style::default().fg(Color::LightGreen)),
            ]));
            continue;
        }

        // Markdown Headers
        if trimmed.starts_with("# ") {
            lines.push(Line::from(Span::styled(format!("    {}", raw_line), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
        } else if trimmed.starts_with("## ") {
            lines.push(Line::from(Span::styled(format!("    {}", raw_line), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
        } else if trimmed.starts_with("### ") {
            lines.push(Line::from(Span::styled(format!("    {}", raw_line), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))));
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            lines.push(Line::from(vec![
                Span::styled("    • ", Style::default().fg(Color::Cyan)),
                Span::styled(trimmed[2..].to_string(), theme.assistant_text()),
            ]));
        } else if trimmed.starts_with("> ") {
            lines.push(Line::from(vec![
                Span::styled("    │ ", Style::default().fg(Color::DarkGray)),
                Span::styled(trimmed[2..].to_string(), Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)),
            ]));
        } else {
            lines.push(Line::from(Span::styled(format!("    {}", raw_line), theme.assistant_text())));
        }
    }
    lines
}

    fn ui(&self, f: &mut Frame) {
        let t = self.theme;

        // Dynamic Input Box Height Calculation (Expand downward dynamically as prompt grows)
        let full_width = f.area().width as usize;
        let input_inner_width = full_width.saturating_sub(2).max(1);
        let input_total_cols = 4 + self.input.chars().count();
        let wrapped_input_lines = (input_total_cols + input_inner_width - 1) / input_inner_width;
        let input_height = (wrapped_input_lines as u16 + 2).clamp(3, 8);

        // Top Header Bar (3 lines), Middle Content (Min 5), Bottom Input (Dynamic 3..8 lines)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(input_height),
            ])
            .split(f.area());

        // 1. Top Header Bar (Clean rounded block with BorderType::Rounded)
        let header_text = vec![Line::from(vec![
            Span::styled("CYNAPSE TUI", t.header_title()),
        ])];

        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("").border_style(t.active_border_style()));
        f.render_widget(header, main_chunks[0]);

        // 2. Middle Content Area: COLIBRI STYLE REARRANGEMENT
        // Split horizontally: [LEFT SIDEBAR (26%), RIGHT CHAT VIEWPORT (74%)]
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(26),
                Constraint::Percentage(74),
            ])
            .split(main_chunks[1]);

        let sidebar_area = middle_chunks[0];
        let viewport_area = middle_chunks[1];

        // 2A. LEFT SIDEBAR PANEL (Clean System Telemetry & Model Stats with Rounded Borders)
        let (nodes, edges) = self.graph.topology();

        // Calculate RAM usage bar
        let used_gb = self.hw_info.ram_used_mb as f64 / 1024.0;
        let total_gb = self.hw_info.ram_total_mb as f64 / 1024.0;
        let pct = self.hw_info.ram_used_pct.min(100.0).max(0.0);
        let filled_blocks = ((pct / 100.0) * 8.0) as usize;
        let ram_bar_str = format!("[{}{}] {:.0}%", "█".repeat(filled_blocks), "░".repeat(8 - filled_blocks), pct);

        let cpu_short = if self.hw_info.cpu_brand.len() > 22 {
            format!("{}...", &self.hw_info.cpu_brand[..20])
        } else {
            self.hw_info.cpu_brand.clone()
        };

        let sidebar_lines = vec![
            Line::from(Span::styled("CYNAPSE CORE", t.header_title())),
            Line::from(""),
            Line::from(Span::styled("HARDWARE TELEMETRY", t.header_title())),
            Line::from(vec![Span::styled(" CPU: ", Style::default().fg(Color::DarkGray)), Span::styled(format!("{} ({}c)", cpu_short, self.hw_info.cpu_cores), Style::default().fg(Color::White))]),
            Line::from(vec![Span::styled(" RAM: ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{:.1}/{:.1} GB", used_gb, total_gb))]),
            Line::from(vec![Span::styled(" Bar: ", Style::default().fg(Color::DarkGray)), Span::styled(ram_bar_str, Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled(" GPU: ", Style::default().fg(Color::DarkGray)), Span::styled(&self.hw_info.gpu_info, Style::default().fg(Color::Green))]),
            Line::from(""),
            Line::from(Span::styled("MODEL DETAILS", t.header_title())),
            Line::from(vec![Span::styled(" Name: ", Style::default().fg(Color::DarkGray)), Span::styled(&self.active_model_name, t.active_model())]),
            Line::from(vec![Span::styled(" Quant: ", Style::default().fg(Color::DarkGray)), Span::styled(&self.active_model_quant, Style::default().fg(Color::Green))]),
            Line::from(vec![Span::styled(" Size:  ", Style::default().fg(Color::DarkGray)), Span::styled(&self.active_model_size, Style::default().fg(Color::Magenta))]),
            Line::from(vec![Span::styled(" Src:   ", Style::default().fg(Color::DarkGray)), Span::raw(&self.active_model_source)]),
            Line::from(""),
            Line::from(Span::styled("ENGINE TIER", t.header_title())),
            Line::from(vec![Span::styled(" Tier:  ", Style::default().fg(Color::DarkGray)), Span::styled("Tier 1 Fast", Style::default().fg(Color::Green))]),
            Line::from(vec![Span::styled(" Speed: ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{:.1} tok/s", self.last_tok_per_sec))]),
            Line::from(vec![Span::styled(" Lat:   ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{:.2} s", self.last_latency_sec))]),
            Line::from(""),
            Line::from(Span::styled("EXECUTION PIPELINE", t.header_title())),
            Line::from(vec![Span::styled(" FTS5:  ", Style::default().fg(Color::DarkGray)), Span::styled("✓ Active", Style::default().fg(Color::Green))]),
            Line::from(vec![Span::styled(" Ranker:", Style::default().fg(Color::DarkGray)), Span::styled("✓ BM25 + Spec", Style::default().fg(Color::Green))]),
            Line::from(vec![Span::styled(" GBNF:  ", Style::default().fg(Color::DarkGray)), Span::styled("✓ Schema Check", Style::default().fg(Color::Green))]),
            Line::from(vec![Span::styled(" RAG:   ", Style::default().fg(Color::DarkGray)), Span::styled("✓ 4k Budget", Style::default().fg(Color::Green))]),
            Line::from(vec![Span::styled(" Engine:", Style::default().fg(Color::DarkGray)), Span::styled(if self.is_generating { "• Running..." } else { "✓ Idle" }, if self.is_generating { Style::default().fg(Color::Yellow) } else { Style::default().fg(Color::Green) })]),
            Line::from(""),
            Line::from(Span::styled("VISUAL THEME", t.header_title())),
            Line::from(vec![Span::styled(" Theme: ", Style::default().fg(Color::DarkGray)), Span::styled(t.name(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(Span::styled("DENDRITE MEMORY", t.header_title())),
            Line::from(vec![Span::styled(" Nodes: ", Style::default().fg(Color::DarkGray)), Span::styled(nodes.len().to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled(" Links: ", Style::default().fg(Color::DarkGray)), Span::styled(edges.len().to_string(), Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled(" DB:    ", Style::default().fg(Color::DarkGray)), Span::raw("FTS5 + Spec Ranker")]),
        ];

        let sidebar = Paragraph::new(sidebar_lines)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Sidebar ").border_style(t.border_style()));
        f.render_widget(sidebar, sidebar_area);

        // 2B. RIGHT CHAT HISTORY VIEWPORT (Jcode-style Background ASCII Art & Paragraph Text Wrap)
        let mut chat_lines = Vec::new();

        let has_user_prompts = self.messages.iter().any(|m| m.role == "user");

        // Render full Centered ASCII Artwork logo when starting / clear state
        if !has_user_prompts {
            chat_lines.push(Line::from(""));
            for line in ASCII_BANNER {
                chat_lines.push(Line::from(Span::styled(*line, t.header_title())));
            }
            chat_lines.push(Line::from(""));
            chat_lines.push(Line::from(Span::styled("                      CYNAPSE LOCAL AGENT SYSTEM", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            chat_lines.push(Line::from(Span::styled("             Pure Rust LLM Engine + Dendrite 4-Tier Memory Graph", Style::default().fg(Color::DarkGray))));
            chat_lines.push(Line::from(""));
            chat_lines.push(Line::from(Span::styled(" Type your prompt below to start conversation... (Try /help, /model, /memory, /thinking)", Style::default().fg(Color::Cyan))));
            chat_lines.push(Line::from(""));
        } else {
            for msg in &self.messages {
                match msg.role.as_str() {
                    "user" => {
                        chat_lines.push(Line::from(vec![
                            Span::styled("User: ", t.user_header()),
                            Span::styled(&msg.content, t.user_text()),
                        ]));
                        chat_lines.push(Line::from(""));
                    }
                    "assistant" => {
                        chat_lines.push(Line::from(Span::styled("Cynapse:", t.assistant_header())));
                        if let Some(think) = &msg.thinking {
                            if self.show_thinking {
                                chat_lines.push(Line::from(Span::styled("  ▼ [Thinking... (Press Ctrl+T to collapse)]", t.thinking_header())));
                                for t_line in think.lines() {
                                    chat_lines.push(Line::from(Span::styled(format!("    {}", t_line), t.thinking_text())));
                                }
                            } else {
                                chat_lines.push(Line::from(Span::styled("  ▶ [Thinking... (Collapsed - Press Ctrl+T to expand)]", t.thinking_header())));
                            }
                            chat_lines.push(Line::from(""));
                        }
                        chat_lines.push(Line::from(Span::styled("  [Response]:", Style::default().fg(Color::Green))));
                        let parsed_lines = Self::render_markdown_lines(&msg.content, t);
                        chat_lines.extend(parsed_lines);
                        chat_lines.push(Line::from(""));
                    }
                    "system" => {
                        chat_lines.push(Line::from(Span::styled(format!("Info: {}", msg.content), t.system_text())));
                        chat_lines.push(Line::from(""));
                    }
                    _ => {
                        chat_lines.push(Line::from(Span::styled(format!("Error: {}", msg.content), t.error_text())));
                        chat_lines.push(Line::from(""));
                    }
                }
            }
        }

        // Pulse loading animation frames during stream generation
        let pulse_frames = ["・>・・", "・・>・", "・・・>", "・・・・"];
        let pulse_str = pulse_frames[self.anim_tick % pulse_frames.len()];

        if self.is_generating {
            chat_lines.push(Line::from(Span::styled(format!("Generating {}", pulse_str), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            if !self.current_thinking_buf.is_empty() {
                if self.show_thinking {
                    chat_lines.push(Line::from(Span::styled("  ▼ [Thinking... (Press Ctrl+T to collapse)]", t.thinking_header())));
                    for l in self.current_thinking_buf.lines() {
                        chat_lines.push(Line::from(Span::styled(format!("    {}", l), t.thinking_text())));
                    }
                } else {
                    chat_lines.push(Line::from(Span::styled("  ▶ [Thinking... (Collapsed - Press Ctrl+T to expand)]", t.thinking_header())));
                }
            }
            if !self.current_response_buf.is_empty() {
                chat_lines.push(Line::from(Span::styled("  [Streaming Response]:", Style::default().fg(Color::Green))));
                let parsed_lines = Self::render_markdown_lines(&self.current_response_buf, t);
                chat_lines.extend(parsed_lines);
            }
            chat_lines.push(Line::from(""));
        }

        let inner_width = viewport_area.width.saturating_sub(2) as usize;
        let inner_height = viewport_area.height.saturating_sub(2) as usize;

        let total_visual_lines = calculate_visual_lines(&chat_lines, inner_width);
        let max_scroll = (total_visual_lines as u16).saturating_sub(inner_height as u16);
        self.last_max_scroll.store(max_scroll, Ordering::Relaxed);
        let effective_scroll = if self.auto_scroll {
            max_scroll
        } else {
            self.scroll_offset.min(max_scroll)
        };

        let scroll_title = if max_scroll > 0 {
            let pct = (effective_scroll as f32 / max_scroll as f32 * 100.0) as u32;
            format!(" Conversation Viewport [▲ Scroll {}% ▼] (Up/Down/PgUp/PgDn) ", pct)
        } else {
            " Conversation Viewport (PgUp/PgDn to scroll) ".to_string()
        };

        let viewport = Paragraph::new(chat_lines)
            .wrap(Wrap { trim: false })
            .scroll((effective_scroll, 0))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(scroll_title).border_style(t.border_style()));
        f.render_widget(viewport, viewport_area);

        // 3. Bottom Prompt Input Bar (Dynamic Height & Multi-line Wrap with '・> ' prefix)
        let input_text = vec![Line::from(vec![
            Span::styled("・> ", t.prompt_prefix()),
            Span::raw(&self.input),
        ])];

        let input_bar = Paragraph::new(input_text)
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("").border_style(t.prompt_prefix()));
        f.render_widget(input_bar, main_chunks[2]);

        // Render terminal blinking cursor at input cursor position (multi-line aware)
        if self.modal == ActiveModal::None {
            let input_area = main_chunks[2];
            let inner_w = input_area.width.saturating_sub(2) as usize;
            if inner_w > 0 {
                let total_offset = 4 + self.input[..self.input_cursor].chars().count();
                let row_offset = (total_offset / inner_w) as u16;
                let col_offset = (total_offset % inner_w) as u16;
                let cursor_x = input_area.x + 1 + col_offset;
                let cursor_y = (input_area.y + 1 + row_offset).min(input_area.y + input_area.height.saturating_sub(2));
                f.set_cursor_position((cursor_x, cursor_y));
            }
        }

        // 4. Floating Slash Command Dropdown Popup
        let matching_cmds = self.get_matching_commands();
        if self.input.starts_with('/') && !self.input.contains(' ') && !matching_cmds.is_empty() {
            let popup_height = (matching_cmds.len() as u16 + 2).min(8);
            let popup_area = Rect {
                x: main_chunks[2].x + 2,
                y: main_chunks[2].y.saturating_sub(popup_height),
                width: main_chunks[2].width.saturating_sub(4).min(65),
                height: popup_height,
            };

            f.render_widget(Clear, popup_area);

            let dropdown_items: Vec<ListItem> = matching_cmds
                .iter()
                .enumerate()
                .map(|(idx, cmd)| {
                    let is_selected = idx == self.autocomplete_idx;
                    let style = if is_selected {
                        t.highlight_item()
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let line = Line::from(vec![
                        Span::styled(format!(" {:<10} ", cmd.name), style),
                        Span::styled(cmd.description, Style::default().fg(Color::DarkGray)),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let dropdown_list = List::new(dropdown_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Autocomplete Commands (Tab/Up/Down) ")
                    .border_style(t.active_border_style()),
            );
            f.render_widget(dropdown_list, popup_area);
        }

        // 5. Modal Overlays
        match self.modal {
            ActiveModal::Help => {
                let area = centered_rect(70, 60, f.area());
                f.render_widget(Clear, area);

                let help_text = vec![
                    Line::from(Span::styled("CYNAPSE AGENT COMMANDS & SHORTCUTS", t.header_title())),
                    Line::from("──────────────────────────────────────────────────────────"),
                    Line::from(vec![Span::styled(" /model ", t.prompt_prefix()), Span::raw("  Open interactive model selector")]),
                    Line::from(vec![Span::styled(" /pull  ", t.prompt_prefix()), Span::raw("  Download GGUF model from HuggingFace")]),
                    Line::from(vec![Span::styled(" /doctor", t.prompt_prefix()), Span::raw("  Run self-healing Cynapse Doctor system diagnostic")]),
                    Line::from(vec![Span::styled(" /memory", t.prompt_prefix()), Span::raw("  View 3D Galaxy Memory Atlas topology")]),
                    Line::from(vec![Span::styled(" /theme ", t.prompt_prefix()), Span::raw("  Cycle visual color theme presets")]),
                    Line::from(vec![Span::styled(" /session", t.prompt_prefix()), Span::raw(" Open saved session manager (resume past runs)")]),
                    Line::from(vec![Span::styled(" /clear ", t.prompt_prefix()), Span::raw("  Clear conversation history")]),
                    Line::from(vec![Span::styled(" /exit  ", t.prompt_prefix()), Span::raw("  Quit Cynapse TUI")]),
                    Line::from("──────────────────────────────────────────────────────────"),
                    Line::from("Keybindings:"),
                    Line::from("  • Tab / Right Arrow : Autocomplete highlighted command"),
                    Line::from("  • Up / Down Arrow   : Navigate dropdown & modal items / Rotate 3D Pitch"),
                    Line::from("  • Left / Right      : Rotate 3D Yaw in Galaxy Memory"),
                    Line::from("  • PgUp / PgDn       : Scroll conversation viewport"),
                    Line::from("  • Esc / q           : Dismiss popup or close modal"),
                    Line::from(""),
                    Line::from(Span::styled("Press Esc or q to return", Style::default().fg(Color::DarkGray))),
                ];

                let modal = Paragraph::new(help_text)
                    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Help & Documentation ").border_style(t.active_border_style()));
                f.render_widget(modal, area);
            }
            ActiveModal::Doctor => {
                let area = centered_rect(85, 75, f.area());
                f.render_widget(Clear, area);

                let mut lines = Vec::new();
                lines.push(Line::from(Span::styled("🩺 CYNAPSE AGENT SELF-HEALING SYSTEM DOCTOR", t.header_title())));
                lines.push(Line::from("──────────────────────────────────────────────────────────"));

                if let Some(rep) = &self.doctor_report {
                    let score_style = if rep.health_score >= 90 {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else if rep.health_score >= 70 {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                    };

                    lines.push(Line::from(vec![
                        Span::raw("System Health Score: "),
                        Span::styled(format!("[ {}% HEALTHY ]", rep.health_score), score_style),
                        Span::raw(format!("  ({} Pass | {} Warning | {} Repaired | {} Failed)", rep.total_pass, rep.total_warn, rep.total_repaired, rep.total_fail)),
                    ]));
                    lines.push(Line::from("──────────────────────────────────────────────────────────"));

                    for item in &rep.items {
                        let badge_style = match item.status {
                            cynapse_core::doctor::DoctorStatus::Pass => Style::default().fg(Color::Green),
                            cynapse_core::doctor::DoctorStatus::Warning => Style::default().fg(Color::Yellow),
                            cynapse_core::doctor::DoctorStatus::Repaired => Style::default().fg(Color::Cyan),
                            cynapse_core::doctor::DoctorStatus::Failed => Style::default().fg(Color::Red),
                        };

                        lines.push(Line::from(vec![
                            Span::styled(format!("{:<13} ", item.status.badge()), badge_style),
                            Span::styled(format!("[{:<12}] ", item.subsystem), Style::default().fg(Color::Cyan)),
                            Span::styled(format!("{:<30} ", item.check_name), Style::default().add_modifier(Modifier::BOLD)),
                        ]));
                        lines.push(Line::from(Span::styled(format!("  └─> {}", item.detail), Style::default().fg(Color::Gray))));

                        if let Some(fix) = &item.fix_recommendation {
                            lines.push(Line::from(Span::styled(format!("      💡 Recommendation: {}", fix), Style::default().fg(Color::Yellow))));
                        }
                    }
                } else {
                    lines.push(Line::from("Running system diagnostics..."));
                }

                lines.push(Line::from("──────────────────────────────────────────────────────────"));
                lines.push(Line::from(vec![
                    Span::styled(" Press r / F5 ", t.prompt_prefix()),
                    Span::raw(" to re-run diagnostic & auto-repair   "),
                    Span::styled(" Esc / q ", t.prompt_prefix()),
                    Span::raw(" to close"),
                ]));

                let modal = Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Cynapse Self-Healing Doctor ").border_style(t.active_border_style()));
                f.render_widget(modal, area);
            }
            ActiveModal::MemoryGraph => {
                let area = centered_rect(88, 80, f.area());
                f.render_widget(Clear, area);

                self.render_3d_galaxy_atlas(f, area);
            }
            ActiveModal::ModelList => {
                let area = centered_rect(80, 65, f.area());
                f.render_widget(Clear, area);

                let scanned = self.scan_models_sync();
                let mut items = Vec::new();
                for (idx, m) in scanned.iter().enumerate() {
                    let is_selected = idx == self.selected_model_idx;
                    let prefix = if is_selected { "> " } else { "  " };
                    let style = if is_selected { t.highlight_item() } else { Style::default().fg(Color::White) };

                    let line = Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(format!("{:<38} ", m.name), style),
                        Span::styled(format!("{:<12} ", m.quant), Style::default().fg(Color::Green)),
                        Span::styled(m.size_str.clone(), Style::default().fg(Color::Magenta)),
                    ]);
                    items.push(ListItem::new(line));
                }

                if items.is_empty() {
                    items.push(ListItem::new(Line::from(" (No local models found in models directory — use /pull)")));
                }

                let list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Interactive Model Selector (Active: {}) - Up/Down/Enter ", self.active_model_name))
                        .border_style(t.active_border_style()),
                );
                f.render_widget(list, area);
            }
            ActiveModal::SessionList => {
                let area = centered_rect(80, 65, f.area());
                f.render_widget(Clear, area);

                let sessions = self.session_mgr.list_sessions();
                let mut items = Vec::new();

                for (idx, s) in sessions.iter().enumerate() {
                    let is_selected = idx == self.selected_session_idx;
                    let prefix = if is_selected { "> " } else { "  " };
                    let style = if is_selected { t.highlight_item() } else { Style::default().fg(Color::White) };

                    let msg_count = s.messages.len();
                    let line = Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(format!("{:<28} ", s.session_id), style),
                        Span::styled(format!("Model: {:<20} ", s.model_name), Style::default().fg(Color::Yellow)),
                        Span::styled(format!("({} msgs)", msg_count), Style::default().fg(Color::DarkGray)),
                    ]);
                    items.push(ListItem::new(line));
                }

                if items.is_empty() {
                    items.push(ListItem::new(Line::from(" (No saved sessions found in ~/.cynapse/sessions/)")));
                }

                let list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" Saved Sessions Manager (Up/Down to navigate | Enter to resume) ")
                        .border_style(t.active_border_style()),
                );
                f.render_widget(list, area);
            }
            ActiveModal::MemoryDrawer => {
                let area = centered_rect(86, 75, f.area());
                f.render_widget(Clear, area);

                let nodes = self.graph.all();
                let mut items = Vec::new();

                for (idx, n) in nodes.iter().enumerate() {
                    let is_selected = idx == self.selected_memory_idx;
                    let prefix = if is_selected { "> " } else { "  " };
                    let style = if is_selected { t.highlight_item() } else { Style::default().fg(Color::White) };

                    let cat = n.category();
                    let cat_color = match cat {
                        cynapse_memory::graph::NodeCategory::Personal => Color::LightMagenta,
                        cynapse_memory::graph::NodeCategory::Engineering => Color::Cyan,
                        cynapse_memory::graph::NodeCategory::Preferences => Color::Yellow,
                        cynapse_memory::graph::NodeCategory::Meta => Color::Green,
                        cynapse_memory::graph::NodeCategory::Episodic => Color::White,
                        cynapse_memory::graph::NodeCategory::Transient => Color::DarkGray,
                    };

                    let spec = n.spec_index();
                    let line = Line::from(vec![
                        Span::styled(prefix, style),
                        Span::styled(format!("[{:<12}] ", cat.as_str()), Style::default().fg(cat_color).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{:<28} ", crate::memory_render::truncate_smart(&n.title, 28)), style),
                        Span::styled(format!("spec:{:.2} ", spec), Style::default().fg(Color::LightYellow)),
                        Span::styled(if !n.tags.is_empty() { format!("#{}", n.tags.join(" #")) } else { "".into() }, Style::default().fg(Color::Blue)),
                    ]);
                    items.push(ListItem::new(line));
                }

                if items.is_empty() {
                    items.push(ListItem::new(Line::from(" (Graph is currently empty — run queries to build memory nodes)")));
                }

                let list = List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Dendrite Memory Inspector ({} Nodes) — Up/Down to navigate | d/Delete to erase | Esc/Tab to close ", nodes.len()))
                        .border_style(t.active_border_style()),
                );
                f.render_widget(list, area);
            }
            ActiveModal::ModelPuller => {
                let area = centered_rect(84, 75, f.area());
                f.render_widget(Clear, area);

                match self.puller_step {
                    PullerStep::CuratedList => {
                        let cat = cynapse_core::downloader::CURATED_MODELS_CATALOG;
                        let rec_idx = cynapse_core::downloader::recommend_model_for_hardware(self.hw_info.ram_total_mb);
                        let mut items = Vec::new();

                        for (idx, m) in cat.iter().enumerate() {
                            let is_selected = idx == self.selected_pull_idx;
                            let prefix = if is_selected { "> " } else { "  " };
                            let style = if is_selected { t.highlight_item() } else { Style::default().fg(Color::White) };
                            let is_rec = idx == rec_idx;

                            let rec_badge = if is_rec {
                                Span::styled(" [★ Recommended]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                            } else {
                                Span::raw("")
                            };

                            let line = Line::from(vec![
                                Span::styled(prefix, style),
                                Span::styled(format!("{:<34} ", m.name), style),
                                Span::styled(format!("{:<8} ", m.size_str), Style::default().fg(Color::Magenta)),
                                rec_badge,
                            ]);
                            items.push(ListItem::new(line));
                        }

                        let list = List::new(items).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .title(format!(" HuggingFace Model Downloader (Host RAM: {:.1} GB) — Up/Down: Select │ Enter: Download │ c/Tab: Custom HF URL ", self.hw_info.ram_total_mb as f64 / 1024.0))
                                .border_style(t.active_border_style()),
                        );
                        f.render_widget(list, area);
                    }
                    PullerStep::CustomInput => {
                        let text = vec![
                            Line::from(Span::styled("CUSTOM HUGGINGFACE MODEL DOWNLOAD", t.header_title())),
                            Line::from("──────────────────────────────────────────────────────────"),
                            Line::from("Type or paste HuggingFace repository URL or identifier:"),
                            Line::from("Examples:"),
                            Line::from("  • Qwen/Qwen2.5-0.5B-Instruct-GGUF"),
                            Line::from("  • TheBloke/Llama-2-7B-GGUF"),
                            Line::from("  • unsloth/gemma-4-12B-it-qat-GGUF"),
                            Line::from(""),
                            Line::from(vec![
                                Span::styled(" URL: ", t.prompt_prefix()),
                                Span::raw(&self.custom_pull_url),
                            ]),
                            Line::from(""),
                            Line::from(Span::styled("Press Enter to choose quantization tier (Q4_K_M, Q5_K_M, Q8_0, F16) │ Esc to back", Style::default().fg(Color::DarkGray))),
                        ];

                        let modal = Paragraph::new(text).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .title(" Custom HuggingFace Write-In ")
                                .border_style(t.active_border_style()),
                        );
                        f.render_widget(modal, area);
                    }
                    PullerStep::QuantSelect => {
                        let mut items = Vec::new();
                        for (idx, q) in QUANT_OPTIONS.iter().enumerate() {
                            let is_selected = idx == self.selected_quant_idx;
                            let prefix = if is_selected { "> " } else { "  " };
                            let style = if is_selected { t.highlight_item() } else { Style::default().fg(Color::White) };
                            let line = Line::from(vec![
                                Span::styled(prefix, style),
                                Span::styled(format!("Quantization: {:<12}", q), style),
                            ]);
                            items.push(ListItem::new(line));
                        }

                        let list = List::new(items).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .title(format!(" Select Quantization for '{}' — Up/Down/Enter ", self.custom_pull_url))
                                .border_style(t.active_border_style()),
                        );
                        f.render_widget(list, area);
                    }
                    PullerStep::Downloading => {
                        let mut lines = Vec::new();
                        lines.push(Line::from(Span::styled("DOWNLOADING HUGGINGFACE MODEL...", t.header_title())));
                        lines.push(Line::from("──────────────────────────────────────────────────────────"));

                        if let Some(st) = &self.current_download_state {
                            let filled = ((st.pct / 100.0) * 30.0) as usize;
                            let bar_str = format!("[{}{}] {:.1}%", "█".repeat(filled), "░".repeat(30 - filled.min(30)), st.pct);
                            let downloaded_mb = st.downloaded_bytes as f64 / 1_048_576.0;
                            let total_mb = st.total_bytes as f64 / 1_048_576.0;

                            lines.push(Line::from(vec![Span::styled(" Model: ", Style::default().fg(Color::DarkGray)), Span::styled(&st.model_name, t.active_model())]));
                            lines.push(Line::from(vec![Span::styled(" Speed: ", Style::default().fg(Color::DarkGray)), Span::styled(format!("{:.2} MB/s", st.speed_mbps), Style::default().fg(Color::Green))]));
                            lines.push(Line::from(vec![Span::styled(" Size:  ", Style::default().fg(Color::DarkGray)), Span::raw(format!("{:.1} / {:.1} MB", downloaded_mb, total_mb))]));
                            lines.push(Line::from(""));
                            lines.push(Line::from(Span::styled(bar_str, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                        } else {
                            lines.push(Line::from(" Initializing download connection..."));
                        }

                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled("Download running in background. Press Esc/q to dismiss window.", Style::default().fg(Color::DarkGray))));

                        let modal = Paragraph::new(lines).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_type(BorderType::Rounded)
                                .title(" Downloading GGUF Model ")
                                .border_style(t.active_border_style()),
                        );
                        f.render_widget(modal, area);
                    }
                }
            }
            ActiveModal::None => {}
        }
    }

    /// Render Multi-Galaxy 3D Dendrite Memory Atlas Visualizer inside Ratatui modal
    fn render_3d_galaxy_atlas(&self, f: &mut Frame, area: Rect) {
        let t = self.theme;
        let width = area.width.saturating_sub(4) as usize;
        let height = area.height.saturating_sub(6) as usize;

        let (nodes, edges) = self.graph.topology();

        // Canvas grid buffer
        let mut grid: Vec<Vec<(char, Style)>> = vec![vec![(' ', Style::default()); width]; height];

        let yaw = self.galaxy_yaw;
        let pitch = self.galaxy_pitch;
        let center_x = (width / 2) as f32;
        let center_y = (height / 2) as f32;

        let cos_y = yaw.cos();
        let sin_y = yaw.sin();
        let cos_p = pitch.cos();
        let sin_p = pitch.sin();

        // 1. Central Core Mass / Black Hole Center
        let cx1 = 0.0 * cos_y - 0.0 * sin_y;
        let cz1 = 0.0 * sin_y + 0.0 * cos_y;
        let cy1 = 0.0 * cos_p - cz1 * sin_p;
        let cpx = (center_x + cx1 * 0.7) as i32;
        let cpy = (center_y + cy1 * 0.35) as i32;
        if cpx >= 0 && cpx < width as i32 && cpy >= 0 && cpy < height as i32 {
            grid[cpy as usize][cpx as usize] = ('✸', Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        }

        // 2. Background Starfield
        let star_seeds = [
            (-24.0, 10.0, -15.0), (22.0, -14.0, 18.0), (-18.0, -20.0, -10.0),
            (25.0, 18.0, 12.0), (-30.0, 5.0, 20.0), (15.0, -25.0, -22.0),
            (-10.0, 28.0, 5.0), (28.0, -8.0, -18.0), (-5.0, -32.0, 14.0),
        ];

        for (sx, sy, sz) in star_seeds {
            let x1 = sx * cos_y - sz * sin_y;
            let z1 = sx * sin_y + sz * cos_y;
            let y1 = sy * cos_p - z1 * sin_p;

            let px = (center_x + x1 * 0.7) as i32;
            let py = (center_y + y1 * 0.35) as i32;

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                grid[py as usize][px as usize] = ('.', Style::default().fg(Color::DarkGray));
            }
        }

        // 3. Category Cluster Center Orbits around Central Star
        let anim_spin = (self.anim_tick as f32) * 0.02;

        let category_clusters = [
            (cynapse_memory::graph::NodeCategory::Meta, 6.0, 0.0 + anim_spin, Color::Green, "Meta"),
            (cynapse_memory::graph::NodeCategory::Preferences, 10.0, 1.2 + anim_spin, Color::Yellow, "Preferences"),
            (cynapse_memory::graph::NodeCategory::Personal, 14.0, 2.4 + anim_spin, Color::LightMagenta, "Personal"),
            (cynapse_memory::graph::NodeCategory::Engineering, 18.0, 3.6 + anim_spin, Color::Cyan, "Engineering"),
            (cynapse_memory::graph::NodeCategory::Episodic, 22.0, 4.8 + anim_spin, Color::White, "Episodic"),
            (cynapse_memory::graph::NodeCategory::Transient, 26.0, 5.8 + anim_spin, Color::DarkGray, "Oort Cloud"),
        ];

        let mut cluster_centers: std::collections::HashMap<cynapse_memory::graph::NodeCategory, (f32, f32, f32)> = std::collections::HashMap::new();
        for (cat, dist, base_angle, _color, _name) in category_clusters {
            let cx = dist * base_angle.cos();
            let cz = dist * base_angle.sin();
            let cy = (base_angle * 0.5).sin() * 2.0;
            cluster_centers.insert(cat, (cx, cy, cz));
        }

        let mut node_coords: std::collections::HashMap<String, (i32, i32)> = std::collections::HashMap::new();

        // 4. Map Dendrite Nodes into Sub-Galaxy Orbits
        for (idx, node) in nodes.iter().enumerate() {
            let cat = node.category();
            let (ccx, ccy, ccz) = cluster_centers.get(&cat).cloned().unwrap_or((0.0, 0.0, 0.0));

            let local_radius = 2.0 + ((idx % 4) as f32 * 1.5);
            let local_angle = (idx as f32 * 1.4) + anim_spin * 1.5;

            let raw_x = ccx + local_radius * local_angle.cos();
            let raw_z = ccz + local_radius * local_angle.sin();
            let raw_y = ccy + ((idx % 3) as f32 - 1.0) * 1.5;

            // 3D rotation transform
            let x1 = raw_x * cos_y - raw_z * sin_y;
            let z1 = raw_x * sin_y + raw_z * cos_y;
            let y1 = raw_y * cos_p - z1 * sin_p;

            let px = (center_x + x1 * 0.7) as i32;
            let py = (center_y + y1 * 0.35) as i32;

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                node_coords.insert(node.id.clone(), (px, py));

                let spec = node.spec_index();
                let cat_color = match cat {
                    cynapse_memory::graph::NodeCategory::Personal => Color::LightMagenta,
                    cynapse_memory::graph::NodeCategory::Engineering => Color::Cyan,
                    cynapse_memory::graph::NodeCategory::Preferences => Color::Yellow,
                    cynapse_memory::graph::NodeCategory::Meta => Color::Green,
                    cynapse_memory::graph::NodeCategory::Episodic => Color::White,
                    cynapse_memory::graph::NodeCategory::Transient => Color::DarkGray,
                };

                let (ch, style) = if spec > 0.75 {
                    ('★', Style::default().fg(cat_color).add_modifier(Modifier::BOLD))
                } else if spec > 0.5 {
                    ('✦', Style::default().fg(cat_color))
                } else {
                    ('●', Style::default().fg(cat_color).add_modifier(Modifier::DIM))
                };

                grid[py as usize][px as usize] = (ch, style);
            }
        }

        // 5. Draw Synapse Edge Links between connected memory nodes
        for edge in &edges {
            if let (Some(&(x1, y1)), Some(&(x2, y2))) = (node_coords.get(&edge.0), node_coords.get(&edge.1)) {
                let mid_x = (x1 + x2) / 2;
                let mid_y = (y1 + y2) / 2;
                if mid_x >= 0 && mid_x < width as i32 && mid_y >= 0 && mid_y < height as i32 {
                    if grid[mid_y as usize][mid_x as usize].0 == ' ' {
                        grid[mid_y as usize][mid_x as usize] = ('·', Style::default().fg(Color::DarkGray));
                    }
                }
            }
        }

        // 3. Convert grid to lines
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("DENDRITE 3D GALAXY MEMORY ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" (Nodes: {} | Edges: {} | Auto-Spin: {})", nodes.len(), edges.len(), if self.galaxy_auto_spin { "ON" } else { "OFF" })),
        ]));
        lines.push(Line::from("──────────────────────────────────────────────────────────────────────────"));

        for row in grid {
            let mut spans = Vec::new();
            for (ch, st) in row {
                spans.push(Span::styled(ch.to_string(), st));
            }
            lines.push(Line::from(spans));
        }

        lines.push(Line::from("──────────────────────────────────────────────────────────────────────────"));
        lines.push(Line::from(vec![
            Span::styled("Controls: ", Style::default().fg(Color::DarkGray)),
            Span::raw("Arrow Keys: Rotate 3D Yaw/Pitch │ Space/s: Toggle Auto-Spin │ Esc/q: Exit Dendrite"),
        ]));

        let atlas_widget = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 3D Dendrite Memory Galaxy Visualizer ")
                .border_style(t.active_border_style()),
        );

        f.render_widget(atlas_widget, area);
    }
}

fn calculate_visual_lines(lines: &[Line], width: usize) -> usize {
    if width == 0 {
        return lines.len();
    }
    let mut total = 0;
    for line in lines {
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        if text.is_empty() {
            total += 1;
            continue;
        }
        let mut cur_len = 0;
        for word in text.split_whitespace() {
            let word_len = word.chars().count();
            if cur_len == 0 {
                cur_len = word_len;
            } else if cur_len + 1 + word_len <= width {
                cur_len += 1 + word_len;
            } else {
                total += 1;
                cur_len = word_len.min(width);
            }
        }
        if cur_len > 0 {
            total += 1;
        }
    }
    total
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
