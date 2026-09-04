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
use std::sync::Arc;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
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

use cynapse_core::session::{SessionData, SessionManager, SessionMessage};
use cynapse_engine::{fetch_ollama_models, probe_hardware_info, query_tier1_stream, SystemHardwareInfo, TokenType};
use cynapse_memory::graph::Dendrite;
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
    ModelList,
    SessionList,
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
    SlashCommand { name: "/memory", description: "View 3D Galaxy Memory Atlas topology" },
    SlashCommand { name: "/theme", description: "Cycle visual color theme (Dark Slate, Neon, Amber, Matrix)" },
    SlashCommand { name: "/session", description: "Open saved sessions manager" },
    SlashCommand { name: "/clear", description: "Clear conversation history" },
    SlashCommand { name: "/exit", description: "Exit Cynapse TUI" },
];

pub const ASCII_BANNER: &[&str] = &[
    "       +####+.",
    "     =##***=:..::",
    "     +#****::..::.",
    "     ##*****=..:",
    " :+***: : +####*+#=",
    " -=-  -++   +#   ++=+:",
    " =##++##:  .  -  :######+",
    " +@@@@#++*=  :+####=  :==*#####*:",
];

pub struct TuiApp {
    pub models_dir: PathBuf,
    pub active_model_name: String,
    pub active_model_quant: String,
    pub active_model_size: String,
    pub active_model_source: String,
    pub tier1_endpoint: String,
    pub graph: Arc<Dendrite>,
    pub input: String,
    pub messages: Vec<ChatMessage>,
    pub modal: ActiveModal,
    pub theme: AppTheme,
    pub is_generating: bool,
    pub last_tok_per_sec: f64,
    pub last_latency_sec: f64,
    pub scroll_offset: u16,
    pub session_mgr: SessionManager,
    pub session_id: String,
    pub stream_rx: Option<mpsc::UnboundedReceiver<StreamEvent>>,
    pub current_thinking_buf: String,
    pub current_response_buf: String,

    // Navigation & Animation states
    pub autocomplete_idx: usize,
    pub selected_model_idx: usize,
    pub selected_session_idx: usize,
    pub anim_tick: usize,

    // Hardware Telemetry
    pub hw_info: SystemHardwareInfo,

    // 3D Galaxy Camera state
    pub galaxy_yaw: f32,
    pub galaxy_pitch: f32,
    pub galaxy_auto_spin: bool,
}

impl TuiApp {
    pub fn new(models_dir: PathBuf, active_model_name: String, tier1_endpoint: String, graph: Arc<Dendrite>) -> Self {
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
            input: String::new(),
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
            session_mgr,
            session_id,
            stream_rx: None,
            current_thinking_buf: String::new(),
            current_response_buf: String::new(),
            autocomplete_idx: 0,
            selected_model_idx: 0,
            selected_session_idx: 0,
            anim_tick: 0,
            hw_info,
            galaxy_yaw: 0.4,
            galaxy_pitch: 0.3,
            galaxy_auto_spin: true,
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

    pub fn load_session(&mut self, session_id: &str) -> Result<()> {
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

            terminal.draw(|f| self.ui(f))?;

            if event::poll(std::time::Duration::from_millis(30))? {
                if let Event::Key(key) = event::read()? {
                    // Global Interrupt
                    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                        self.save_current_session();
                        break;
                    }

                    // Handle Modal Inputs
                    if self.modal != ActiveModal::None {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
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
                                ActiveModal::ModelList => {
                                    self.selected_model_idx = self.selected_model_idx.saturating_add(1);
                                }
                                ActiveModal::SessionList => {
                                    self.selected_session_idx = self.selected_session_idx.saturating_add(1);
                                }
                                _ => {}
                            },
                            KeyCode::Char('s') | KeyCode::Char(' ') => match self.modal {
                                ActiveModal::MemoryGraph => {
                                    self.galaxy_auto_spin = !self.galaxy_auto_spin;
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
                                    self.autocomplete_idx = 0;
                                }
                                continue;
                            }
                            KeyCode::Enter => {
                                if let Some(cmd) = matching_cmds.get(self.autocomplete_idx) {
                                    self.input = cmd.name.to_string();
                                    self.autocomplete_idx = 0;
                                }
                            }
                            _ => {}
                        }
                    }

                    // General Input Field Key Handling
                    match key.code {
                        KeyCode::Esc => {
                            self.input.clear();
                            self.modal = ActiveModal::None;
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                            self.autocomplete_idx = 0;
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                            self.autocomplete_idx = 0;
                        }
                        KeyCode::PageUp => {
                            self.scroll_offset = self.scroll_offset.saturating_add(3);
                        }
                        KeyCode::PageDown => {
                            self.scroll_offset = self.scroll_offset.saturating_sub(3);
                        }
                        KeyCode::Enter => {
                            let trimmed = self.input.trim().to_string();
                            if trimmed.is_empty() {
                                continue;
                            }
                            self.input.clear();

                            // Execute Slash Commands
                            if trimmed == "/exit" || trimmed == "exit" || trimmed == "quit" {
                                self.save_current_session();
                                break;
                            }

                            if trimmed == "/clear" || trimmed == "/cls" {
                                self.messages.clear();
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

                            if trimmed == "/session" || trimmed == "/sessions" {
                                self.selected_session_idx = 0;
                                self.modal = ActiveModal::SessionList;
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

                            self.is_generating = true;
                            self.current_thinking_buf.clear();
                            self.current_response_buf.clear();
                            self.scroll_offset = 0; // Auto scroll to bottom

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
                                    "You are CYNAPSE — a local-first, modular, precise AI companion.",
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
                    self.messages.push(ChatMessage {
                        role: "assistant".into(),
                        content: self.current_response_buf.clone(),
                        thinking: if self.current_thinking_buf.is_empty() {
                            None
                        } else {
                            Some(self.current_thinking_buf.clone())
                        },
                    });
                    self.is_generating = false;
                    self.save_current_session();
                    finished = true;
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
        let re = regex::Regex::new(r"(?i)(Q[0-9]_[K0-9_A-Z]+|F16|F32|IQ[0-9]_[A-Z]+)").unwrap();

        let search_dirs = [
            self.models_dir.clone(),
            PathBuf::from("/home/xander/Documents/portfolio/cynapse-mini/models"),
            PathBuf::from("./models"),
        ];

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

    fn ui(&self, f: &mut Frame) {
        let t = self.theme;

        // Top Header Bar (3 lines), Middle Content (Min 5), Bottom Input (3 lines)
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
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
            Line::from(Span::styled("VISUAL THEME", t.header_title())),
            Line::from(vec![Span::styled(" Theme: ", Style::default().fg(Color::DarkGray)), Span::styled(t.name(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(Span::styled("DENDRITE MEMORY", t.header_title())),
            Line::from(vec![Span::styled(" Nodes: ", Style::default().fg(Color::DarkGray)), Span::styled(nodes.len().to_string(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))]),
            Line::from(vec![Span::styled(" Links: ", Style::default().fg(Color::DarkGray)), Span::styled(edges.len().to_string(), Style::default().fg(Color::Cyan))]),
            Line::from(vec![Span::styled(" DB:    ", Style::default().fg(Color::DarkGray)), Span::raw("FTS5 + BM25")]),
        ];

        let sidebar = Paragraph::new(sidebar_lines)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Sidebar ").border_style(t.border_style()));
        f.render_widget(sidebar, sidebar_area);

        // 2B. RIGHT CHAT HISTORY VIEWPORT (Jcode-style Background ASCII Art & Paragraph Text Wrap)
        let mut chat_lines = Vec::new();

        // Render Jcode-style Centered Background ASCII Art when chat is empty
        if self.messages.len() <= 1 {
            chat_lines.push(Line::from(""));
            for line in ASCII_BANNER {
                chat_lines.push(Line::from(Span::styled(*line, t.header_title())));
            }
            chat_lines.push(Line::from(""));
            chat_lines.push(Line::from(Span::styled("            CYNAPSE LOCAL AGENT SYSTEM", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            chat_lines.push(Line::from(Span::styled("      Pure Rust LLM Engine + Dendrite 4-Tier Memory", Style::default().fg(Color::DarkGray))));
            chat_lines.push(Line::from(""));
        }

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
                        chat_lines.push(Line::from(Span::styled("  [Thinking...]", t.thinking_header())));
                        for t_line in think.lines() {
                            chat_lines.push(Line::from(Span::styled(format!("    {}", t_line), t.thinking_text())));
                        }
                        chat_lines.push(Line::from(""));
                    }
                    chat_lines.push(Line::from(Span::styled("  [Response]:", Style::default().fg(Color::Green))));
                    for r_line in msg.content.lines() {
                        chat_lines.push(Line::from(Span::styled(format!("    {}", r_line), t.assistant_text())));
                    }
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

        // Pulse loading animation frames during stream generation
        let pulse_frames = ["・>・・", "・・>・", "・・・>", "・・・・"];
        let pulse_str = pulse_frames[self.anim_tick % pulse_frames.len()];

        if self.is_generating {
            chat_lines.push(Line::from(Span::styled(format!("Generating {}", pulse_str), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))));
            if !self.current_thinking_buf.is_empty() {
                chat_lines.push(Line::from(Span::styled("  [Thinking...]", t.thinking_header())));
                for l in self.current_thinking_buf.lines() {
                    chat_lines.push(Line::from(Span::styled(format!("    {}", l), t.thinking_text())));
                }
            }
            if !self.current_response_buf.is_empty() {
                chat_lines.push(Line::from(Span::styled("  [Streaming Response]:", Style::default().fg(Color::Green))));
                for l in self.current_response_buf.lines() {
                    chat_lines.push(Line::from(Span::styled(format!("    {}", l), t.assistant_text())));
                }
            }
            chat_lines.push(Line::from(""));
        }

        let viewport = Paragraph::new(chat_lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title(" Conversation Viewport (PgUp/PgDn to scroll) ").border_style(t.border_style()));
        f.render_widget(viewport, viewport_area);

        // 3. Bottom Prompt Input Bar (Clean BorderType::Rounded with '・> ' prefix)
        let input_text = vec![Line::from(vec![
            Span::styled("・> ", t.prompt_prefix()),
            Span::raw(&self.input),
        ])];

        let input_bar = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("").border_style(t.prompt_prefix()));
        f.render_widget(input_bar, main_chunks[2]);

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
            ActiveModal::None => {}
        }
    }

    /// Render 3D Galaxy Memory Atlas Visualizer inside Ratatui modal
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

        // 1. Draw procedural background galaxy stars
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

        // 2. Map Dendrite Nodes into 3D space & project onto terminal screen
        for (idx, node) in nodes.iter().enumerate() {
            let tier = node.node_type.tier();
            let radius = match tier {
                3 => 2.0 + (idx as f32 * 0.5),   // Core
                2 => 8.0 + (idx as f32 * 0.8),   // Inner Disk
                1 => 16.0 + (idx as f32 * 1.2),  // Outer Arms
                _ => 24.0 + (idx as f32 * 1.5),  // Halo
            };

            let angle = (idx as f32 * 1.37) + (tier as f32 * 0.8);
            let raw_x = radius * angle.cos();
            let raw_z = radius * angle.sin();
            let raw_y = ((idx % 5) as f32 - 2.0) * 2.5;

            // 3D rotation transform
            let x1 = raw_x * cos_y - raw_z * sin_y;
            let z1 = raw_x * sin_y + raw_z * cos_y;
            let y1 = raw_y * cos_p - z1 * sin_p;

            let px = (center_x + x1 * 0.7) as i32;
            let py = (center_y + y1 * 0.35) as i32;

            if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                let (ch, style) = match tier {
                    3 => ('*', Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    2 => ('+', Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    1 => ('o', Style::default().fg(Color::Green)),
                    _ => ('.', Style::default().fg(Color::Yellow)),
                };
                grid[py as usize][px as usize] = (ch, style);
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
