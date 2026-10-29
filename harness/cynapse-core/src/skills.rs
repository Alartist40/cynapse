//! In-Memory Universal Skill Engine & Dynamic Micro-Directive Slicer.
//!
//! Synthesizes the 30KB `UNIVERSAL-SKILL.md` into zero-token in-memory disciplines:
//! 1. `OutputDiscipline` (i-have-adhd): 4-bullet reader format, max 5 items, lead with action.
//! 2. `SurgicalEngineering` (ponytail): minimal code blast radius, exact patching via patch_file.
//! 3. `CompletionGates` (unlazy): observable gates (CHECK, EXPECT, EVIDENCE), no premature stop.
//! 4. `TokenCompression` (caveman): ASD-STE100 concise brevity, no pleasantries.
//! 5. `SecurityAudit` (strix): pre-flight vulnerability validation, proof-of-concept gating.
//! 6. `HarnessEngineering`: persistent PLAN.md / GATES.md companion artifacts.
//!
//! Micro-directives are dynamically sliced into Zone B based on active user query intent,
//! ensuring total prompt context stays strictly <= 650 tokens on 0.5B-3B models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillDiscipline {
    OutputDiscipline,
    SurgicalEngineering,
    CompletionGates,
    TokenCompression,
    SecurityAudit,
    HarnessEngineering,
}

impl SkillDiscipline {
    /// Returns the ultra-compact micro-directive for this discipline (≤ 50 tokens).
    pub fn micro_directive(&self) -> &'static str {
        match self {
            Self::OutputDiscipline => {
                "DIRECTIVE (Output): Lead with direct answer/command on line 1. Cap lists at 5 items. End with one concrete next action (<2 min). Zero conversational filler."
            }
            Self::SurgicalEngineering => {
                "DIRECTIVE (Code): Smallest viable change only. Never rewrite intact files — use patch_file. Match existing style. Run pre-flight check before declaring done."
            }
            Self::CompletionGates => {
                "DIRECTIVE (Gates): Maintain GATES.md with CHECK, EXPECT, EVIDENCE. Run run_gates to verify machine execution — never assume success. Do not stop until all pass."
            }
            Self::TokenCompression => {
                "DIRECTIVE (Anti-Slop): High-density imperative terseness. Eliminate AI boilerplate ('delve', 'testament', 'revolutionize', 'not just X but Y', empty contrasts). State facts and technical actions directly."
            }
            Self::SecurityAudit => {
                "DIRECTIVE (Security): White-box vulnerability check. Flag destructive commands, path escapes, and injection vectors. Require PoC verification."
            }
            Self::HarnessEngineering => {
                "DIRECTIVE (Harness): Maintain persistent PLAN.md and GATES.md. Scaffold clean interfaces and deterministic verification commands."
            }
        }
    }
}

/// Dynamically classifies the user query intent to select the highest-signal micro-discipline.
pub fn select_active_discipline(user_query: &str) -> SkillDiscipline {
    let q = user_query.to_lowercase();

    if q.contains("security") || q.contains("audit") || q.contains("vuln") || q.contains("owasp") || q.contains("cve") || q.contains("injection") {
        SkillDiscipline::SecurityAudit
    } else if q.contains("patch") || q.contains("edit") || q.contains("fix") || q.contains("refactor") || q.contains("bug") || q.contains("write_file") || q.contains("code") {
        SkillDiscipline::SurgicalEngineering
    } else if q.contains("gate") || q.contains("test") || q.contains("verify") || q.contains("check") || q.contains("unlazy") || q.contains("plan") {
        SkillDiscipline::CompletionGates
    } else if q.contains("compress") || q.contains("short") || q.contains("caveman") || q.contains("terse") || q.contains("slop") || q.contains("humanize") || q.contains("cleanse") {
        SkillDiscipline::TokenCompression
    } else if q.contains("harness") || q.contains("scaffold") || q.contains("pipeline") {
        SkillDiscipline::HarnessEngineering
    } else {
        SkillDiscipline::OutputDiscipline
    }
}

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// A dynamic, learned procedural skill stored in Dendrite memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedSkill {
    pub id: String,
    pub name: String,
    pub trigger_keywords: Vec<String>,
    pub micro_directive: String,
    pub use_count: usize,
    pub last_used_epoch_secs: u64,
    pub created_at_epoch_secs: u64,
    pub is_pinned: bool,
}

impl LearnedSkill {
    pub fn new(id: impl Into<String>, name: impl Into<String>, triggers: Vec<String>, directive: impl Into<String>) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        Self {
            id: id.into(),
            name: name.into(),
            trigger_keywords: triggers,
            micro_directive: directive.into(),
            use_count: 0,
            last_used_epoch_secs: now,
            created_at_epoch_secs: now,
            is_pinned: false,
        }
    }

    pub fn matches(&self, query: &str) -> bool {
        let lower = query.to_lowercase();
        self.trigger_keywords.iter().any(|k| lower.contains(&k.to_lowercase()))
    }
}

/// In-memory dynamic skill registry that grows skills and prunes obsolete skills to prevent prompt bloat.
#[derive(Debug, Default)]
pub struct DynamicSkillRegistry {
    skills: HashMap<String, LearnedSkill>,
}

impl DynamicSkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: LearnedSkill) {
        self.skills.insert(skill.id.clone(), skill);
    }

    pub fn get_skill_mut(&mut self, id: &str) -> Option<&mut LearnedSkill> {
        self.skills.get_mut(id)
    }

    pub fn record_usage(&mut self, id: &str) {
        if let Some(skill) = self.skills.get_mut(id) {
            skill.use_count += 1;
            skill.last_used_epoch_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }
    }

    /// Prunes obsolete learned skills that have not been used within the threshold, unless pinned.
    /// Returns the number of pruned skills.
    pub fn prune_obsolete(&mut self, max_idle_secs: u64) -> usize {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let before = self.skills.len();
        self.skills.retain(|_, skill| {
            if skill.is_pinned {
                return true;
            }
            let idle = now.saturating_sub(skill.last_used_epoch_secs);
            idle <= max_idle_secs
        });
        before - self.skills.len()
    }

    /// Finds matching dynamic skills for a query, updates usage, and returns formatted micro-directives.
    pub fn match_and_format(&mut self, query: &str) -> Option<String> {
        let mut matched_directives = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        for skill in self.skills.values_mut() {
            if skill.matches(query) {
                skill.use_count += 1;
                skill.last_used_epoch_secs = now;
                matched_directives.push(format!("DIRECTIVE ({}): {}", skill.name, skill.micro_directive));
            }
        }

        if matched_directives.is_empty() {
            None
        } else {
            Some(matched_directives.join("\n"))
        }
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

static GLOBAL_SKILL_REGISTRY: OnceLock<Mutex<DynamicSkillRegistry>> = OnceLock::new();

pub fn global_skills() -> &'static Mutex<DynamicSkillRegistry> {
    GLOBAL_SKILL_REGISTRY.get_or_init(|| Mutex::new(DynamicSkillRegistry::new()))
}

/// Formats the operating directive section to be injected into Zone B.
pub fn format_dynamic_directive(user_query: &str) -> String {
    let discipline = select_active_discipline(user_query);
    let mut out = format!("=== OPERATING DIRECTIVE ===\n{}", discipline.micro_directive());

    if let Ok(mut reg) = global_skills().lock() {
        if let Some(dynamic) = reg.match_and_format(user_query) {
            out.push('\n');
            out.push_str(&dynamic);
        }
    }
    out.push_str("\n\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discipline_intent_routing() {
        assert_eq!(
            select_active_discipline("Fix the bug in src/lib.rs"),
            SkillDiscipline::SurgicalEngineering
        );
        assert_eq!(
            select_active_discipline("Run unit tests and verify the gates"),
            SkillDiscipline::CompletionGates
        );
        assert_eq!(
            select_active_discipline("Audit this endpoint for command injection"),
            SkillDiscipline::SecurityAudit
        );
        assert_eq!(
            select_active_discipline("What is the status of the server?"),
            SkillDiscipline::OutputDiscipline
        );
    }

    #[test]
    fn test_format_dynamic_directive() {
        let directive = format_dynamic_directive("Please patch the authentication token");
        assert!(directive.contains("=== OPERATING DIRECTIVE ==="));
        assert!(directive.contains("Smallest viable change only"));
        assert!(directive.contains("patch_file"));
    }

    #[test]
    fn test_dynamic_skill_growth_and_obsolescence_pruning() {
        let mut reg = DynamicSkillRegistry::new();
        let mut s1 = LearnedSkill::new("s1", "DockerDeploy", vec!["docker".into(), "container".into()], "Use multi-stage builds.");
        s1.last_used_epoch_secs = 100; // old
        let mut s2 = LearnedSkill::new("s2", "SqliteWal", vec!["sqlite".into()], "Enable WAL mode.");
        s2.last_used_epoch_secs = 200;
        s2.is_pinned = true; // pinned: immune to pruning

        reg.register(s1);
        reg.register(s2);
        assert_eq!(reg.len(), 2);

        // Match query
        let matched = reg.match_and_format("How to run sqlite?").unwrap();
        assert!(matched.contains("DIRECTIVE (SqliteWal)"));

        // Pruning with max_idle = 0 should prune unpinned s1
        let pruned = reg.prune_obsolete(0);
        assert_eq!(pruned, 1);
        assert_eq!(reg.len(), 1);
        assert!(reg.get_skill_mut("s2").is_some());
        assert!(reg.get_skill_mut("s1").is_none());
    }
}
