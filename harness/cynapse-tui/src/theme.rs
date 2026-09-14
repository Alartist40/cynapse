//! Cynapse TUI Visual Theme Manager — Inspired by jcode.
//!
//! Provides color palettes and styling presets for TUI widgets, headers, user prompt,
//! assistant response, thinking blocks, and modals.

use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppTheme {
    DarkSlate,
    Cyberpunk,
    AmberCRT,
    EmeraldMatrix,
    VioletSynth,
}

impl AppTheme {
    pub fn name(&self) -> &'static str {
        match self {
            AppTheme::DarkSlate => "Dark Slate (Jcode)",
            AppTheme::Cyberpunk => "Cyberpunk Neon",
            AppTheme::AmberCRT => "Amber CRT",
            AppTheme::EmeraldMatrix => "Emerald Matrix",
            AppTheme::VioletSynth => "Violet Synth (Tokyo Night)",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            AppTheme::DarkSlate => AppTheme::Cyberpunk,
            AppTheme::Cyberpunk => AppTheme::AmberCRT,
            AppTheme::AmberCRT => AppTheme::EmeraldMatrix,
            AppTheme::EmeraldMatrix => AppTheme::VioletSynth,
            AppTheme::VioletSynth => AppTheme::DarkSlate,
        }
    }

    // Header Styles
    pub fn header_title(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(80, 210, 240)).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 60, 180)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 191, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(50, 255, 100)).add_modifier(Modifier::BOLD),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(187, 154, 247)).add_modifier(Modifier::BOLD),
        }
    }

    pub fn active_model(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(255, 215, 100)).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 240, 120)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(140, 255, 160)).add_modifier(Modifier::BOLD),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(224, 175, 104)).add_modifier(Modifier::BOLD),
        }
    }

    pub fn border_style(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(90, 100, 120)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(140, 60, 160)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(160, 110, 30)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(40, 130, 60)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(86, 95, 137)),
        }
    }

    pub fn active_border_style(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(80, 210, 240)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 80, 220)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 191, 0)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(0, 255, 120)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(122, 162, 247)),
        }
    }

    // Role Headers & Text
    pub fn user_header(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(138, 180, 248)).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(0, 240, 255)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 215, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(100, 255, 140)).add_modifier(Modifier::BOLD),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(122, 162, 247)).add_modifier(Modifier::BOLD),
        }
    }

    pub fn user_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(245, 245, 255)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(230, 250, 255)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 240, 200)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(220, 255, 220)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(192, 202, 245)),
        }
    }

    pub fn assistant_header(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(129, 199, 132)).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(80, 255, 130)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 180, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(50, 240, 90)).add_modifier(Modifier::BOLD),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(158, 206, 106)).add_modifier(Modifier::BOLD),
        }
    }

    pub fn assistant_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(225, 228, 238)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(240, 245, 255)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 225, 160)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(200, 255, 200)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(207, 216, 235)),
        }
    }

    pub fn thinking_header(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(186, 139, 255)).add_modifier(Modifier::ITALIC),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 80, 180)).add_modifier(Modifier::ITALIC),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(230, 150, 20)).add_modifier(Modifier::ITALIC),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(80, 200, 120)).add_modifier(Modifier::ITALIC),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(187, 154, 247)).add_modifier(Modifier::ITALIC),
        }
    }

    pub fn thinking_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(175, 160, 210)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 140, 210)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(210, 140, 30)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(100, 190, 110)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(157, 140, 200)),
        }
    }

    pub fn system_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(255, 200, 100)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 220, 80)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 200, 50)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(160, 240, 120)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(224, 175, 104)),
        }
    }

    pub fn error_text(&self) -> Style {
        Style::default().fg(Color::Rgb(255, 100, 100)).add_modifier(Modifier::BOLD)
    }

    // Input Bar
    pub fn prompt_prefix(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(80, 210, 240)).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(255, 60, 180)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(255, 191, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(50, 255, 100)).add_modifier(Modifier::BOLD),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(187, 154, 247)).add_modifier(Modifier::BOLD),
        }
    }

    // High-Contrast Autocomplete & List Selection Highlight
    pub fn highlight_item(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::White).bg(Color::Rgb(40, 75, 140)).add_modifier(Modifier::BOLD),
            AppTheme::Cyberpunk => Style::default().fg(Color::White).bg(Color::Rgb(140, 30, 130)).add_modifier(Modifier::BOLD),
            AppTheme::AmberCRT => Style::default().fg(Color::Black).bg(Color::Rgb(255, 191, 0)).add_modifier(Modifier::BOLD),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Black).bg(Color::Rgb(50, 255, 100)).add_modifier(Modifier::BOLD),
            AppTheme::VioletSynth => Style::default().fg(Color::White).bg(Color::Rgb(104, 80, 170)).add_modifier(Modifier::BOLD),
        }
    }

    // Secondary / Dim Text (Visible, readable slate gray — never black)
    pub fn dim_text(&self) -> Style {
        match self {
            AppTheme::DarkSlate => Style::default().fg(Color::Rgb(145, 150, 170)),
            AppTheme::Cyberpunk => Style::default().fg(Color::Rgb(165, 145, 185)),
            AppTheme::AmberCRT => Style::default().fg(Color::Rgb(185, 145, 60)),
            AppTheme::EmeraldMatrix => Style::default().fg(Color::Rgb(110, 170, 120)),
            AppTheme::VioletSynth => Style::default().fg(Color::Rgb(110, 115, 148)),
        }
    }

    /// Theme-adaptive character styling for ASCII art
    pub fn ascii_char_style(&self, ch: char) -> Style {
        match self {
            AppTheme::DarkSlate => match ch {
                '@' | '%' => Style::default().fg(Color::Rgb(100, 230, 255)).add_modifier(Modifier::BOLD),
                '#' | '*' => Style::default().fg(Color::Rgb(130, 180, 255)),
                '+' | '=' => Style::default().fg(Color::Rgb(80, 190, 210)),
                '-' | ':' => Style::default().fg(Color::Rgb(100, 130, 160)),
                '.' => Style::default().fg(Color::Rgb(70, 90, 115)),
                _ => Style::default().fg(Color::Rgb(60, 75, 95)),
            },
            AppTheme::Cyberpunk => match ch {
                '@' | '%' => Style::default().fg(Color::Rgb(255, 60, 180)).add_modifier(Modifier::BOLD),
                '#' | '*' => Style::default().fg(Color::Rgb(255, 110, 220)),
                '+' | '=' => Style::default().fg(Color::Rgb(0, 240, 255)),
                '-' | ':' => Style::default().fg(Color::Rgb(170, 80, 220)),
                '.' => Style::default().fg(Color::Rgb(100, 50, 130)),
                _ => Style::default().fg(Color::Rgb(70, 35, 90)),
            },
            AppTheme::AmberCRT => match ch {
                '@' | '%' => Style::default().fg(Color::Rgb(255, 220, 60)).add_modifier(Modifier::BOLD),
                '#' | '*' => Style::default().fg(Color::Rgb(255, 191, 0)),
                '+' | '=' => Style::default().fg(Color::Rgb(255, 150, 0)),
                '-' | ':' => Style::default().fg(Color::Rgb(190, 110, 0)),
                '.' => Style::default().fg(Color::Rgb(110, 70, 10)),
                _ => Style::default().fg(Color::Rgb(70, 45, 10)),
            },
            AppTheme::EmeraldMatrix => match ch {
                '@' | '%' => Style::default().fg(Color::Rgb(80, 255, 130)).add_modifier(Modifier::BOLD),
                '#' | '*' => Style::default().fg(Color::Rgb(0, 255, 100)),
                '+' | '=' => Style::default().fg(Color::Rgb(30, 200, 80)),
                '-' | ':' => Style::default().fg(Color::Rgb(20, 130, 50)),
                '.' => Style::default().fg(Color::Rgb(15, 80, 30)),
                _ => Style::default().fg(Color::Rgb(10, 50, 20)),
            },
            AppTheme::VioletSynth => match ch {
                '@' | '%' => Style::default().fg(Color::Rgb(210, 160, 255)).add_modifier(Modifier::BOLD),
                '#' | '*' => Style::default().fg(Color::Rgb(187, 154, 247)),
                '+' | '=' => Style::default().fg(Color::Rgb(122, 162, 247)),
                '-' | ':' => Style::default().fg(Color::Rgb(100, 110, 160)),
                '.' => Style::default().fg(Color::Rgb(65, 75, 110)),
                _ => Style::default().fg(Color::Rgb(45, 50, 75)),
            },
        }
    }
}

pub fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cynapse").join("settings.json"))
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct UserSettings {
    theme: Option<AppTheme>,
}

/// Loads the saved theme from ~/.cynapse/settings.json, defaulting to AmberCRT.
pub fn load_theme() -> AppTheme {
    if let Some(path) = settings_path() {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str::<UserSettings>(&content) {
                    if let Some(t) = settings.theme {
                        return t;
                    }
                }
            }
        }
    }
    AppTheme::AmberCRT
}

/// Persists the selected theme to ~/.cynapse/settings.json.
pub fn save_theme(theme: AppTheme) {
    if let Some(path) = settings_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut settings = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|c| serde_json::from_str::<UserSettings>(&c).ok())
                .unwrap_or_default()
        } else {
            UserSettings::default()
        };
        settings.theme = Some(theme);
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = std::fs::write(&path, json);
        }
    }
}
