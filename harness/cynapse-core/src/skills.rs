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

/// Formats the operating directive section to be injected into Zone B.
pub fn format_dynamic_directive(user_query: &str) -> String {
    let discipline = select_active_discipline(user_query);
    format!("=== OPERATING DIRECTIVE ===\n{}\n\n", discipline.micro_directive())
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
}
