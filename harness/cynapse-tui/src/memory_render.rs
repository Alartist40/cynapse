//! Cynapse Visual Memory Renderer — jcode-inspired TUI visualizer.
//!
//! Provides categorized memory box cards, 4-tier graph topology ASCII trees,
//! color-coded node age badges, and real-time step execution graphics.

use std::collections::HashMap;
use colored::*;
use cynapse_memory::graph::{Dendrite, Node};

/// Pipeline execution step status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Done,
    Error,
}

pub struct PipelineState {
    pub search_detail: String,
    pub search_status: StepStatus,
    pub verify_detail: String,
    pub verify_status: StepStatus,
    pub inject_detail: String,
    pub inject_status: StepStatus,
    pub update_detail: String,
    pub update_status: StepStatus,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            search_detail: "waiting".into(),
            search_status: StepStatus::Pending,
            verify_detail: "waiting".into(),
            verify_status: StepStatus::Pending,
            inject_detail: "waiting".into(),
            inject_status: StepStatus::Pending,
            update_detail: "waiting".into(),
            update_status: StepStatus::Pending,
        }
    }
}

/// Formats relative time since timestamp in seconds.
fn format_age_seconds(ts: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff = now.saturating_sub(ts);

    if diff < 5 {
        "now".to_string()
    } else if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Truncate text cleanly with ellipsis
pub fn truncate_smart(s: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max_len.saturating_sub(1)).collect::<String>())
    }
}

/// Render jcode-inspired memory execution pipeline steps.
pub fn render_memory_pipeline(pipeline: &PipelineState) {
    println!("{}", "╭─ MEMORY PIPELINE ───────────────────────────────────────────────────".cyan());
    render_step_line("╭ ", "Find matches    ", &pipeline.search_status, &pipeline.search_detail);
    render_step_line("├ ", "Check relevance ", &pipeline.verify_status, &pipeline.verify_detail);
    render_step_line("├ ", "Inject context  ", &pipeline.inject_status, &pipeline.inject_detail);
    render_step_line("╰ ", "Update memory   ", &pipeline.update_status, &pipeline.update_detail);
    println!("{}", "╰─────────────────────────────────────────────────────────────────────".cyan());
}

fn render_step_line(prefix: &str, label: &str, status: &StepStatus, detail: &str) {
    let (marker, marker_color) = match status {
        StepStatus::Pending => ("·", "gray"),
        StepStatus::Running => ("•", "yellow"),
        StepStatus::Done => ("✓", "green"),
        StepStatus::Error => ("!", "red"),
    };

    let (p_str, m_str) = match marker_color {
        "green" => (prefix.green(), marker.bold().green()),
        "yellow" => (prefix.yellow(), marker.bold().yellow()),
        "red" => (prefix.red(), marker.bold().red()),
        _ => (prefix.dimmed(), marker.dimmed()),
    };

    println!("{} {} {}   {}", p_str, m_str, label.bright_white(), detail.dimmed());
}

/// Render full visual Dendrite memory overview (box cards + ASCII tree topology).
pub fn render_dendrite_visualizer(graph: &Dendrite) {
    let (nodes, edges) = graph.topology();

    println!("{}", "======================================================================".cyan().bold());
    println!("{}", "             🧠 CYNAPSE DENDRITE MEMORY CORE (4-TIER GRAPH)           ".yellow().bold());
    println!("{}", "======================================================================".cyan().bold());
    println!(
        "Total Nodes: {} | Edge Links: {} | FTS5 Index: Active | Ranker: BM25",
        nodes.len().to_string().green().bold(),
        edges.len().to_string().cyan().bold()
    );
    println!("{}", "----------------------------------------------------------------------".cyan());

    if nodes.is_empty() {
        println!(" (Memory graph is currently empty. Run conversation turns to build graph.)");
        println!("{}", "======================================================================".cyan().bold());
        return;
    }

    // Group nodes by memory tier
    let mut tier_groups: HashMap<u8, Vec<Node>> = HashMap::new();
    for node in &nodes {
        tier_groups.entry(node.node_type.tier()).or_default().push(node.clone());
    }

    let tier_labels = [
        (3, "L3 CONSOLIDATED CORE (#summary / identity)", "magenta"),
        (2, "L2 PROCEDURES & CONCEPTS (#procedure / how-to)", "cyan"),
        (1, "L1 ATOMIC FACTS & EVENTS (#fact / preference)", "green"),
        (0, "L0 EPHEMERAL TURN LOGS (#transcript)", "yellow"),
    ];

    println!("\n{}", "📋 CATEGORIZED MEMORY TILES & CARDS".yellow().bold());

    for (tier_num, title, color_name) in tier_labels {
        let group_nodes = tier_groups.get(&tier_num).cloned().unwrap_or_default();
        if group_nodes.is_empty() {
            continue;
        }

        render_memory_box_card(title, &group_nodes, color_name);
        println!();
    }

    // Render 4-tier Graph Topology ASCII Tree
    println!("{}", "🌐 DENDRITE GRAPH TOPOLOGY (CONNECTIVITY & EDGES)".cyan().bold());
    println!("{}", "----------------------------------------------------------------------".cyan());

    for (idx, node) in nodes.iter().take(15).enumerate() {
        let is_last = idx == nodes.len().min(15) - 1;
        let connector = if is_last { "└─" } else { "├─" };

        let tier_badge = match node.node_type.tier() {
            3 => "[L3 Summary]".magenta().bold(),
            2 => "[L2 Procedure]".cyan().bold(),
            1 => "[L1 Fact]".green().bold(),
            _ => "[L0 TurnLog]".yellow().dimmed(),
        };

        let age_str = format_age_seconds(node.updated_at);
        println!(
            " {} {} {:<28} {} {}",
            connector.bright_black(),
            tier_badge,
            truncate_smart(&node.title, 28).bright_white(),
            format!("(updated {})", age_str).dimmed(),
            if !node.tags.is_empty() {
                format!("[#{}]", node.tags.join(" #")).blue()
            } else {
                "".blue()
            }
        );

        // Render outgoing links
        for (l_idx, link) in node.links.iter().enumerate() {
            let sub_conn = if l_idx == node.links.len() - 1 { "    └─" } else { "    ├─" };
            println!(" {} {} {}", sub_conn.bright_black(), "──>".cyan(), link.dimmed());
        }
    }

    println!("{}", "======================================================================".cyan().bold());
}

fn render_memory_box_card(title: &str, nodes: &[Node], color_name: &str) {
    let box_width: usize = 68;
    let header_text = format!(" {} ({}) ", title, nodes.len());
    let title_len = header_text.chars().count();
    let border_chars = box_width.saturating_sub(title_len + 2);
    let left_border = "─".repeat(border_chars / 2);
    let right_border = "─".repeat(border_chars - border_chars / 2);

    let top_border = format!("╭{}{}{}╮", left_border, header_text, right_border);
    let bottom_border = format!("╰{}╯", "─".repeat(box_width.saturating_sub(2)));

    match color_name {
        "magenta" => println!("{}", top_border.magenta().bold()),
        "cyan" => println!("{}", top_border.cyan().bold()),
        "green" => println!("{}", top_border.green().bold()),
        "yellow" => println!("{}", top_border.yellow().bold()),
        _ => println!("{}", top_border.white().bold()),
    }

    for node in nodes.iter().take(5) {
        let age_str = format_age_seconds(node.updated_at);
        let content_snippet = truncate_smart(&node.title, 38);
        let line_content = format!("│  · {:<38} {:>18}  │", content_snippet, format!("({})", age_str));
        
        match color_name {
            "magenta" => println!("{}", line_content.bright_white()),
            "cyan" => println!("{}", line_content.cyan()),
            "green" => println!("{}", line_content.green()),
            "yellow" => println!("{}", line_content.yellow()),
            _ => println!("{}", line_content.white()),
        }
    }

    match color_name {
        "magenta" => println!("{}", bottom_border.magenta().bold()),
        "cyan" => println!("{}", bottom_border.cyan().bold()),
        "green" => println!("{}", bottom_border.green().bold()),
        "yellow" => println!("{}", bottom_border.yellow().bold()),
        _ => println!("{}", bottom_border.white().bold()),
    }
}
