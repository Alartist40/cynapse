//! AutoHarness Engine for Synaptic Dendrite Memory.
//!
//! Inspired by the AutoHarness self-learning skill layer:
//! Automatically distills skills from working turns, tracks opportunity-relative usage rates,
//! enforces probation periods, and non-destructively archives unused skills when capacity is exceeded.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle state of a learned skill.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillStatus {
    /// Newly learned skill protected from eviction while building usage evidence.
    Probation,
    /// Established skill subject to capacity contention.
    Mature,
    /// Dormant skill moved out of active prompt recall (non-destructive).
    Archived,
}

/// A self-authored learned skill record within Dendrite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearnedSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub category: String,
    pub created_at_request: u64,
    pub last_loaded_request: u64,
    pub loads_count: u64,
    pub views_count: u64,
    pub is_archived: bool,
    pub absorbed_by: Option<String>,
}

impl LearnedSkill {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        instructions: impl Into<String>,
        category: impl Into<String>,
        current_request: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            instructions: instructions.into(),
            category: category.into(),
            created_at_request: current_request,
            last_loaded_request: current_request,
            loads_count: 0,
            views_count: 0,
            is_archived: false,
            absorbed_by: None,
        }
    }

    /// Evaluates the opportunity-relative usage rate:
    /// loads over requests arrived since skill creation.
    pub fn usage_rate(&self, current_request: u64) -> f64 {
        let opportunities = current_request.saturating_sub(self.created_at_request).max(1);
        self.loads_count as f64 / opportunities as f64
    }

    /// Determines whether the skill is still in probation.
    pub fn status(&self, current_request: u64, probation_threshold: u64) -> SkillStatus {
        if self.is_archived {
            SkillStatus::Archived
        } else if current_request.saturating_sub(self.created_at_request) < probation_threshold {
            SkillStatus::Probation
        } else {
            SkillStatus::Mature
        }
    }
}

/// Configuration options for the AutoHarness memory engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoHarnessOptions {
    /// Tool execution quanta between background reflection runs. Default 50.
    pub reflect_every_n_tools: usize,
    /// Number of requests a skill sits in probation before graduation review. Default 100.
    pub probation_requests: u64,
    /// Maximum number of active mature skills before capacity contention archives lowest rates. Default 30.
    pub mature_capacity_limit: usize,
    /// Maximum characters for the per-skill description in prompt index. Default 80.
    pub index_desc_max_chars: usize,
}

impl Default for AutoHarnessOptions {
    fn default() -> Self {
        Self {
            reflect_every_n_tools: 50,
            probation_requests: 100,
            mature_capacity_limit: 30,
            index_desc_max_chars: 80,
        }
    }
}

/// The AutoHarness Lifecycle & Skill Manager for Dendrite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoHarness {
    pub options: AutoHarnessOptions,
    pub request_counter: u64,
    pub tool_call_counter: usize,
    pub skills: HashMap<String, LearnedSkill>,
}

impl AutoHarness {
    pub fn new(options: AutoHarnessOptions) -> Self {
        Self {
            options,
            request_counter: 0,
            tool_call_counter: 0,
            skills: HashMap::new(),
        }
    }

    /// Advances the global request counter. Called on each user message/turn.
    pub fn record_request(&mut self) {
        self.request_counter += 1;
    }

    /// Records a tool execution. Returns true if reflection threshold is crossed.
    pub fn record_tool_call(&mut self) -> bool {
        self.tool_call_counter += 1;
        if self.tool_call_counter >= self.options.reflect_every_n_tools {
            self.tool_call_counter = 0;
            true
        } else {
            false
        }
    }

    /// Records when a skill was loaded / adhered to by an agent.
    pub fn record_skill_load(&mut self, skill_id: &str) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.loads_count += 1;
            skill.last_loaded_request = self.request_counter;
        }
    }

    /// Records when a skill was viewed in context index.
    pub fn record_skill_view(&mut self, skill_id: &str) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.views_count += 1;
        }
    }

    /// Registers a newly learned skill into Dendrite.
    pub fn register_skill(&mut self, skill: LearnedSkill) {
        self.skills.insert(skill.id.clone(), skill);
        self.evaluate_lifecycle();
    }

    /// Evaluates the active library against probation and capacity rules.
    ///
    /// Non-destructive archiving: mature skills exceeding capacity are moved out of active recall,
    /// preserving their content so they can be revived if needed.
    pub fn evaluate_lifecycle(&mut self) {
        let current_req = self.request_counter;
        let probation = self.options.probation_requests;
        let cap = self.options.mature_capacity_limit;

        // Gather mature active skill IDs and their usage rates
        let mut mature_active: Vec<(String, f64)> = self
            .skills
            .values()
            .filter(|s| !s.is_archived && s.status(current_req, probation) == SkillStatus::Mature)
            .map(|s| (s.id.clone(), s.usage_rate(current_req)))
            .collect();

        if mature_active.len() > cap {
            // Sort by usage rate ascending (lowest usage rate first)
            mature_active.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let to_archive = mature_active.len() - cap;

            for (id, _) in mature_active.into_iter().take(to_archive) {
                if let Some(skill) = self.skills.get_mut(&id) {
                    skill.is_archived = true;
                }
            }
        }
    }

    /// Folds near-duplicate skills under a unified umbrella skill.
    pub fn consolidate_umbrella(&mut self, umbrella_id: &str, absorbed_ids: &[&str]) {
        for &absorbed in absorbed_ids {
            if absorbed != umbrella_id {
                if let Some(skill) = self.skills.get_mut(absorbed) {
                    skill.is_archived = true;
                    skill.absorbed_by = Some(umbrella_id.to_string());
                }
            }
        }
    }

    /// Revives an archived skill back into active duty.
    pub fn revive_skill(&mut self, skill_id: &str) {
        if let Some(skill) = self.skills.get_mut(skill_id) {
            skill.is_archived = false;
            skill.absorbed_by = None;
            skill.created_at_request = self.request_counter; // fresh probation window
        }
    }

    /// Builds the compact session-start prompt index of active skills.
    pub fn build_prompt_index(&self) -> String {
        let active: Vec<&LearnedSkill> = self.skills.values().filter(|s| !s.is_archived).collect();
        if active.is_empty() {
            return String::new();
        }

        let mut out = String::from("=== LEARNED SKILLS INDEX ===\n");
        for s in active {
            let desc = if s.description.chars().count() > self.options.index_desc_max_chars {
                s.description.chars().take(self.options.index_desc_max_chars).collect::<String>() + "…"
            } else {
                s.description.clone()
            };
            out.push_str(&format!("- [{}] {}: {}\n", s.category, s.name, desc));
        }
        out.push('\n');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autoharness_lifecycle_probation_and_capacity_archiving() {
        let opts = AutoHarnessOptions {
            reflect_every_n_tools: 5,
            probation_requests: 10,
            mature_capacity_limit: 2,
            index_desc_max_chars: 50,
        };
        let mut harness = AutoHarness::new(opts);

        // 1. Add skill in probation
        let s1 = LearnedSkill::new("s1", "Skill 1", "Git patching", "Use patch_file", "git", 0);
        harness.register_skill(s1);
        assert_eq!(harness.skills["s1"].status(0, 10), SkillStatus::Probation);

        // 2. Advance requests past probation
        for _ in 0..15 {
            harness.record_request();
        }
        assert_eq!(harness.skills["s1"].status(harness.request_counter, 10), SkillStatus::Mature);

        // 3. Add s2 and s3 directly at request 0 so they are mature
        let mut s2 = LearnedSkill::new("s2", "Skill 2", "Rust check", "Run cargo check", "rust", 0);
        s2.loads_count = 5; // high usage rate
        harness.register_skill(s2);

        let s3 = LearnedSkill::new("s3", "Skill 3", "SQL query", "Inspect tables", "db", 0);
        harness.register_skill(s3);

        // Mature pool is now 3 skills: s1 (0 loads), s2 (5 loads), s3 (0 loads)
        // Capacity is 2 -> s1 is archived non-destructively
        assert_eq!(harness.skills["s2"].is_archived, false);
        let archived_count = harness.skills.values().filter(|s| s.is_archived).count();
        assert_eq!(archived_count, 1);

        // 4. Revive s1
        let s1_archived_id = if harness.skills["s1"].is_archived { "s1" } else { "s3" };
        harness.revive_skill(s1_archived_id);
        assert_eq!(harness.skills[s1_archived_id].is_archived, false);
    }

    #[test]
    fn test_consolidation_umbrella() {
        let mut harness = AutoHarness::new(AutoHarnessOptions::default());
        let umbrella = LearnedSkill::new("git_workflow", "Git Workflow", "Full git ops", "", "git", 0);
        let part1 = LearnedSkill::new("git_commit", "Git Commit", "Commit files", "", "git", 0);
        let part2 = LearnedSkill::new("git_push", "Git Push", "Push branches", "", "git", 0);

        harness.register_skill(umbrella);
        harness.register_skill(part1);
        harness.register_skill(part2);

        harness.consolidate_umbrella("git_workflow", &["git_commit", "git_push"]);

        assert_eq!(harness.skills["git_commit"].is_archived, true);
        assert_eq!(harness.skills["git_commit"].absorbed_by.as_deref(), Some("git_workflow"));
        assert_eq!(harness.skills["git_push"].is_archived, true);
        assert_eq!(harness.skills["git_workflow"].is_archived, false);
    }

    #[test]
    fn test_opportunity_relative_usage_rate() {
        let mut s = LearnedSkill::new("test", "Test", "Desc", "Body", "cat", 100);
        s.loads_count = 10;
        // At request 200, 100 requests have passed since creation
        let rate = s.usage_rate(200);
        assert!((rate - 0.1).abs() < 1e-6);
    }
}
