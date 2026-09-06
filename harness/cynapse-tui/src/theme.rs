//! Cynapse TUI Visual Theme Manager — Inspired by jcode.
//!
//! Provides color palettes and styling presets for TUI widgets, headers, user prompt,
//! assistant response, thinking blocks, and modals.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTheme {
    DarkSlate,
    Cyberpunk,
    AmberCRT,
    EmeraldMatrix,
}

impl AppTheme {
    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::DarkSlate => "Dark Slate",
            AppTheme::Cyberpunk => "Cyberpunk Neon",
            AppTheme::AmberCRT => "Amber CRT",
            AppTheme::EmeraldMatrix => "Emerald Matrix",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            AppTheme::DarkSlate => AppTheme::Cyberpunk,
            AppTheme::Cyberpunk => AppTheme::AmberCRT,
            AppTheme::AmberCRT => AppTheme::EmeraldMatrix,
            AppTheme::EmeraldMatrix => AppTheme::DarkSlate,
        }
    }

    // Header Styles
    pub fn header_title(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 191, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        }
    }

    pub fn active_model(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        }
    }

    pub fn border_style(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::DarkGray),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(128, 0, 128)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(180, 100, 0)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(0, 100, 0)),
        }
    }

    pub fn active_border_style(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Cyan),
            AppTheme::Cyberpunk => Style::default().fg(Color::LightMagenta),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 191, 0)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Green),
        }
    }

    // Role Headers & Text
    pub fn user_header(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        }
    }

    pub fn user_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::White),
            AppTheme::Cyberpunk => Style::default().fg(Color::LightCyan),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 235, 175)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(200, 255, 200)),
        }
    }

    pub fn assistant_header(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(50, 205, 50)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 180, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        }
    }

    pub fn assistant_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Reset),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(240, 240, 255)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 220, 150)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(180, 255, 180)),
        }
    }

    pub fn thinking_header(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Magenta).add_modifier(Modifier::ITALIC),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 20, 147)).add_modifier(Modifier::ITALIC),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(200, 140, 0)).add_modifier(Modifier::ITALIC),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(100, 180, 100)).add_modifier(Modifier::ITALIC),
        }
    }

    pub fn thinking_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::LightMagenta),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 105, 180)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(180, 120, 0)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(80, 150, 80)),
        }
    }

    pub fn system_text(&self) -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn error_text(&self) -> Style {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    }

    // Input Bar
    pub fn prompt_prefix(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 0, 128)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 191, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        }
    }

    // Autocomplete Dropdown Selection Highlight
    pub fn highlight_item(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(15, 15, 15)).bg(Color::Cyan).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(15, 15, 15)).bg(Color::Magenta).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(15, 15, 15)).bg(Color::Rgb(255, 191, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(15, 15, 15)).bg(Color::Green).add_modifier(Modifier::BOLD),
        }
    }
}
