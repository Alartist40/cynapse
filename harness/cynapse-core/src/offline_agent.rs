//! Atomic-Agent Inspired Offline Execution Core.
//!
//! Provides strict GBNF JSON tool grammar validation, stable KV-cache prompt prefixing,
//! parallel read-only tool execution batching, and circular loop-guard protection for offline LLM engines.

use std::collections::{HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

// ─── Two-Phase Tool Loop Tracker ─────────────────────────────────────────────
// Inspired by Atomic-Agent's `ToolLoopTracker` (loop-detector.ts).
// Two-phase history: check() → record_call() → record_outcome().
// Two counters: args-only repeat (warn), args+result no-progress streak (critical).
// Wandering detector for distinct-args tools. Breaker for consecutive vetoes.

/// Verdict level from the loop tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopVerdict {
    /// No loop detected.
    Ok,
    /// Args-only repeat crossing warning threshold — advisory notice.
    Warn,
    /// No-progress streak (same args + same result) crossing critical threshold.
    Critical,
    /// Distinct-args spread on a wandering-prone tool.
    Wandering,
}

/// Classification of which detector fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopDetector {
    GenericRepeat,
    NoProgress,
    Wandering,
}

/// Result of a loop check.
#[derive(Debug, Clone)]
pub struct LoopCheckResult {
    pub level: LoopVerdict,
    pub detector: LoopDetector,
    /// Repeat count (warn), streak length (critical), or distinct-args spread (wandering).
    pub count: usize,
}

#[derive(Debug, Clone)]
struct HistoryEntry {
    tool: String,
    args_hash: u64,
    result_hash: Option<u64>,
    vetoed: bool,
}

/// Options for the tool loop tracker.
#[derive(Debug, Clone)]
pub struct ToolLoopTrackerOptions {
    /// Args-only repeat count that fires a warn. Min 2. Default 3.
    pub warning_threshold: usize,
    /// No-progress streak (args+result) that fires a critical veto. Default 5.
    pub critical_threshold: usize,
    /// Consecutive vetoes that trip the breaker. Default 3.
    pub breaker_veto_streak: usize,
    /// Sliding window size. Default 30.
    pub history_size: usize,
    /// Distinct-args spread that fires a wandering warn. Default 6.
    pub wandering_threshold: usize,
    /// Distinct-args spread that escalates to breaker. Default 12.
    pub wandering_escalation: usize,
}

impl Default for ToolLoopTrackerOptions {
    fn default() -> Self {
        Self {
            warning_threshold: 3,
            critical_threshold: 5,
            breaker_veto_streak: 3,
            history_size: 30,
            wandering_threshold: 6,
            wandering_escalation: 12,
        }
    }
}

/// Two-phase per-turn tool loop tracker.
///
/// Phase 1: `check(tool, args)` — read-only verdict against history-so-far.
/// Phase 2a: `record_call(tool, args)` — commit the call entry.
/// Phase 2b: `record_outcome(tool, args, result_hash)` — patch the result.
#[derive(Debug, Clone)]
pub struct ToolLoopTracker {
    opts: ToolLoopTrackerOptions,
    history: VecDeque<HistoryEntry>,
    consecutive_veto_tool: Option<String>,
    consecutive_veto_count: usize,
}

impl ToolLoopTracker {
    pub fn new(opts: ToolLoopTrackerOptions) -> Self {
        let history_size = opts.history_size;
        Self {
            opts,
            history: VecDeque::with_capacity(history_size.min(64)),
            consecutive_veto_tool: None,
            consecutive_veto_count: 0,
        }
    }

    /// Check a prospective call against history-so-far. Call BEFORE `record_call`.
    pub fn check(&self, tool: &str, args: &serde_json::Value) -> LoopCheckResult {
        let args_hash = hash_tool_call(tool, args);

        // 1. No-progress streak (args+result identical)
        let no_progress = self.get_no_progress_streak(tool, args_hash);
        if no_progress + 1 >= self.opts.critical_threshold {
            return LoopCheckResult {
                level: LoopVerdict::Critical,
                detector: LoopDetector::NoProgress,
                count: no_progress,
            };
        }

        // 2. Wandering detector (distinct args on same tool)
        if is_wandering_prone(tool) {
            let spread = self.effective_spread(tool, args_hash);
            let already_repeats = self.get_repeat_count(tool, args_hash) > 0;
            if spread >= self.opts.wandering_threshold && !already_repeats {
                return LoopCheckResult {
                    level: LoopVerdict::Wandering,
                    detector: LoopDetector::Wandering,
                    count: spread,
                };
            }
        }

        // 3. Args-only repeat
        let repeat_count = self.get_repeat_count(tool, args_hash);
        if repeat_count + 1 >= self.opts.warning_threshold {
            return LoopCheckResult {
                level: LoopVerdict::Warn,
                detector: LoopDetector::GenericRepeat,
                count: repeat_count,
            };
        }

        LoopCheckResult {
            level: LoopVerdict::Ok,
            detector: LoopDetector::GenericRepeat,
            count: 0,
        }
    }

    /// Push a pending history entry. Call after `check()`, before tool execution.
    pub fn record_call(&mut self, tool: &str, args: &serde_json::Value) {
        let args_hash = hash_tool_call(tool, args);
        self.history.push_back(HistoryEntry {
            tool: tool.to_string(),
            args_hash,
            result_hash: None,
            vetoed: false,
        });
        self.trim_history();
    }

    /// Patch the latest pending entry with a result hash. Call after tool execution.
    pub fn record_outcome(&mut self, tool: &str, args: &serde_json::Value, result_hash: Option<u64>) {
        let args_hash = hash_tool_call(tool, args);
        // Find latest pending entry for (tool, args)
        for entry in self.history.iter_mut().rev() {
            if entry.tool == tool && entry.args_hash == args_hash && entry.result_hash.is_none() && !entry.vetoed {
                entry.result_hash = result_hash;
                return;
            }
        }
        // No pending entry: append a finished entry
        self.history.push_back(HistoryEntry {
            tool: tool.to_string(),
            args_hash,
            result_hash,
            vetoed: false,
        });
        self.trim_history();
    }

    /// Record a veto (no-progress loop). The vetoed entry is excluded from the streak.
    pub fn record_veto(&mut self, tool: &str, args: &serde_json::Value) {
        let args_hash = hash_tool_call(tool, args);
        // Patch latest pending as vetoed
        for entry in self.history.iter_mut().rev() {
            if entry.tool == tool && entry.args_hash == args_hash && entry.result_hash.is_none() && !entry.vetoed {
                entry.vetoed = true;
                break;
            }
        }
        // Track consecutive vetoes
        if self.consecutive_veto_tool.as_deref() == Some(tool) {
            self.consecutive_veto_count += 1;
        } else {
            self.consecutive_veto_tool = Some(tool.to_string());
            self.consecutive_veto_count = 1;
        }
    }

    /// Whether the breaker has tripped (too many consecutive vetoes on same tool).
    pub fn is_breaker_tripped(&self, tool: &str) -> bool {
        self.consecutive_veto_tool.as_deref() == Some(tool)
            && self.consecutive_veto_count >= self.opts.breaker_veto_streak
    }

    /// Whether the wandering spread has crossed the escalation threshold.
    pub fn is_wandering_escalated(&self, tool: &str, args: &serde_json::Value) -> bool {
        if !is_wandering_prone(tool) {
            return false;
        }
        let args_hash = hash_tool_call(tool, args);
        self.effective_spread(tool, args_hash) >= self.opts.wandering_escalation
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.consecutive_veto_tool = None;
        self.consecutive_veto_count = 0;
    }

    // ── Internals ──

    fn trim_history(&mut self) {
        while self.history.len() > self.opts.history_size {
            self.history.pop_front();
        }
    }

    /// Count matching (tool, argsHash) entries in window.
    fn get_repeat_count(&self, tool: &str, args_hash: u64) -> usize {
        self.history.iter()
            .filter(|e| e.tool == tool && e.args_hash == args_hash)
            .count()
    }

    /// Walk backwards counting consecutive (tool, argsHash) entries with same resultHash.
    fn get_no_progress_streak(&self, tool: &str, args_hash: u64) -> usize {
        let mut streak = 0;
        let mut latest_result: Option<u64> = None;
        for entry in self.history.iter().rev() {
            if entry.vetoed || entry.result_hash.is_none() {
                continue;
            }
            if entry.tool != tool || entry.args_hash != args_hash {
                break;
            }
            let rh = entry.result_hash.unwrap();
            if latest_result.is_none() {
                latest_result = Some(rh);
                streak = 1;
                continue;
            }
            if rh != latest_result.unwrap() {
                break;
            }
            streak += 1;
        }
        streak
    }

    /// Distinct completed argsHashes for tool in the window, plus one if current is new.
    fn effective_spread(&self, tool: &str, current_args_hash: u64) -> usize {
        let mut seen = HashSet::new();
        for entry in &self.history {
            if entry.tool != tool {
                continue;
            }
            if entry.result_hash.is_none() || entry.vetoed {
                continue;
            }
            seen.insert(entry.args_hash);
        }
        if seen.contains(&current_args_hash) {
            seen.len()
        } else {
            seen.len() + 1
        }
    }
}

/// Tools whose repeated invocation with changing args is a "wandering" loop.
pub fn is_wandering_prone(tool: &str) -> bool {
    tool == "execute_command"
}

/// Stable signature for a (tool, args) pair.
pub fn hash_tool_call(tool: &str, args: &serde_json::Value) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tool.hash(&mut hasher);
    args.to_string().hash(&mut hasher);
    hasher.finish()
}

/// Hash a tool result for no-progress comparison.
pub fn hash_tool_result(tool: &str, args: &serde_json::Value, result: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    tool.hash(&mut hasher);
    args.to_string().hash(&mut hasher);
    result.hash(&mut hasher);
    hasher.finish()
}

/// Standard parsed tool invocation structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Helper to parse a single JSON value into a ToolCall
fn parse_single_tool_call(v: &serde_json::Value) -> Option<ToolCall> {
    let name = v.get("name")
        .or_else(|| v.get("tool"))
        .or_else(|| v.get("function").and_then(|f| f.get("name")))
        .and_then(|s| s.as_str())?;

    let arguments = v.get("arguments")
        .or_else(|| v.get("args"))
        .or_else(|| v.get("parameters"))
        .or_else(|| v.get("function").and_then(|f| f.get("arguments")))
        .cloned()
        .unwrap_or_else(|| {
            // Flattened arguments fallback: gather non-metadata keys
            let mut map = serde_json::Map::new();
            if let Some(obj) = v.as_object() {
                for (k, val) in obj {
                    if k != "name" && k != "tool" && k != "type" && k != "function" {
                        map.insert(k.clone(), val.clone());
                    }
                }
            }
            serde_json::Value::Object(map)
        });

    let parsed_args = if let Some(s) = arguments.as_str() {
        serde_json::from_str(s).unwrap_or(arguments)
    } else {
        arguments
    };

    Some(ToolCall {
        name: name.to_string(),
        arguments: parsed_args,
    })
}

/// Validates and extracts GBNF tool calls from raw model output, supporting code blocks, XML tags, and raw JSON objects.
pub fn validate_gbnf_tool_calls(output: &str) -> Result<Vec<ToolCall>, String> {
    let trimmed = output.trim();

    // 1. Direct parse of whole trimmed text (if it's already a valid JSON object or array)
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(arr) = val.as_array() {
            let calls: Vec<ToolCall> = arr.iter().filter_map(parse_single_tool_call).collect();
            if !calls.is_empty() {
                return Ok(calls);
            }
        } else if let Some(call) = parse_single_tool_call(&val) {
            return Ok(vec![call]);
        }
    }

    // 2. Scan for Markdown ```json ... ``` or ``` ... ``` code fences
    let mut search_pos = 0;
    while let Some(start_fence) = output[search_pos..].find("```") {
        let actual_start = search_pos + start_fence;
        let content_start = if output[actual_start..].starts_with("```json") {
            actual_start + 7
        } else {
            actual_start + 3
        };

        if let Some(end_fence) = output[content_start..].find("```") {
            let chunk = output[content_start..content_start + end_fence].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(chunk) {
                if let Some(arr) = val.as_array() {
                    let calls: Vec<ToolCall> = arr.iter().filter_map(parse_single_tool_call).collect();
                    if !calls.is_empty() {
                        return Ok(calls);
                    }
                } else if let Some(call) = parse_single_tool_call(&val) {
                    return Ok(vec![call]);
                }
            }
            search_pos = content_start + end_fence + 3;
        } else {
            break;
        }
    }

    // 3. Scan for <tool_call>...</tool_call> tags
    if let Some(start_tag) = output.find("<tool_call>") {
        let content_start = start_tag + 11;
        if let Some(end_tag) = output[content_start..].find("</tool_call>") {
            let chunk = output[content_start..content_start + end_tag].trim();
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(chunk) {
                if let Some(arr) = val.as_array() {
                    let calls: Vec<ToolCall> = arr.iter().filter_map(parse_single_tool_call).collect();
                    if !calls.is_empty() {
                        return Ok(calls);
                    }
                } else if let Some(call) = parse_single_tool_call(&val) {
                    return Ok(vec![call]);
                }
            }
        }
    }

    // 4. Scan for standalone raw JSON objects containing "name" or "tool"
    let mut search_idx = 0;
    while let Some(open_brace) = output[search_idx..].find('{') {
        let abs_start = search_idx + open_brace;
        if let Some(close_brace) = output[abs_start..].rfind('}') {
            let candidate = &output[abs_start..=abs_start + close_brace];
            if candidate.contains("\"name\"") || candidate.contains("\"tool\"") || candidate.contains("\"function\"") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                    if let Some(arr) = val.as_array() {
                        let calls: Vec<ToolCall> = arr.iter().filter_map(parse_single_tool_call).collect();
                        if !calls.is_empty() {
                            return Ok(calls);
                        }
                    } else if let Some(call) = parse_single_tool_call(&val) {
                        return Ok(vec![call]);
                    }
                }
            }
        }
        search_idx = abs_start + 1;
        if search_idx >= output.len() {
            break;
        }
    }

    Err("No valid GBNF tool call detected".to_string())
}

/// Validates a raw model output string against GBNF tool call grammar and returns the first tool call.
pub fn validate_gbnf_tool_call(output: &str) -> Result<ToolCall, String> {
    let calls = validate_gbnf_tool_calls(output)?;
    calls.into_iter().next().ok_or_else(|| "No tool calls found".to_string())
}

/// Formats a context prompt with a stable invariant prefix to maximize KV-cache hits in offline engines.
pub fn format_stable_kv_prompt(system_prompt: &str, tools_schema: &str, conversation_history: &str) -> String {
    format!(
        "=== CYNAPSE SYSTEM PRESET ===\n\
        {}\n\n\
        === AVAILABLE TOOL SCHEMAS ===\n\
        {}\n\n\
        === CONVERSATION LOG ===\n\
        {}",
        system_prompt.trim(),
        tools_schema.trim(),
        conversation_history.trim()
    )
}

/// Builds a clean conversational system prompt without tool schemas or JSON invocation grammar
/// for casual greetings and general dialogue.
pub fn compile_conversational_prefix(_system_persona: &str) -> String {
    "You are Cynapse, an intelligent, concise, and friendly local companion. Respond naturally and directly to the user.".to_string()
}

/// Zone A (100% byte-stable prefix): Invariant persona, behavior directives, and tool definitions.
/// Pinned in KV cache to enable 0 ms prefill latency across multi-turn sessions.
pub fn compile_zone_a_prefix(system_persona: &str) -> String {
    let tools = crate::list_tools();
    let mut tools_desc = String::new();
    for t in tools {
        tools_desc.push_str(&format!("- `{}`: {}\n", t.name, t.description));
    }

    format!(
        "You are Cynapse, an autonomous local intelligent assistant.\n\
        {}\n\n\
        Operating Directives:\n\
        1. Lead with the direct answer, command, or code snippet on line 1.\n\
        2. Number multi-step tasks clearly. Cap lists at maximum 5 items.\n\
        3. End with exactly one concrete next action doable immediately.\n\
        4. State cause and fix directly for errors. Be concise and eliminate conversational filler.\n\
        5. When asked to write, create, edit, or read a file, search code, or run a shell command:\n\
           Immediately emit the tool call in a ```json code block. Do not just talk about doing it — execute it:\n\
           ```json\n\
           {{\"name\": \"<tool_name>\", \"arguments\": {{\"path\": \"...\", \"content\": \"...\"}}}}\n\
           ```\n\
           For general conversational questions or explanations, respond directly in plain text without tools.\n\n\
        Available Tools:\n\
        {}",
        system_persona.trim(),
        tools_desc.trim()
    )
}

/// Zone B (Variable tail): Recalled Dendrite knowledge nodes (deterministically sorted),
/// sliding conversation history (with compressed tool results), optional operator/loop notice, and active user instruction.
pub fn compile_zone_b_tail(
    recalled_knowledge: &str,
    conversation_history: &str,
    user_query: &str,
    notice: Option<&str>,
) -> String {
    let mut tail = String::new();

    if !recalled_knowledge.trim().is_empty() {
        if recalled_knowledge.trim().starts_with("===") {
            tail.push_str(recalled_knowledge.trim());
        } else {
            tail.push_str("=== RECALLED KNOWLEDGE ===\n");
            tail.push_str(recalled_knowledge.trim());
        }
        tail.push_str("\n\n");
    }

    if !conversation_history.trim().is_empty() {
        tail.push_str("=== CONVERSATION LOG ===\n");
        tail.push_str(conversation_history.trim());
        tail.push_str("\n\n");
    }

    if let Some(n) = notice {
        if !n.trim().is_empty() {
            tail.push_str("=== NOTICE ===\n");
            tail.push_str(n.trim());
            tail.push_str("\n\n");
        }
    }

    tail.push_str("=== USER INSTRUCTION ===\n");
    tail.push_str(user_query.trim());
    tail
}

/// Two-phase tool loop guard wrapping ToolLoopTracker.
/// Detects repeating invocations, identical result no-progress streaks, and wandering loops.
#[derive(Debug, Clone)]
pub struct LoopGuard {
    tracker: ToolLoopTracker,
}

impl Default for LoopGuard {
    fn default() -> Self {
        Self::new(ToolLoopTrackerOptions::default())
    }
}

impl LoopGuard {
    pub fn new(opts: ToolLoopTrackerOptions) -> Self {
        Self {
            tracker: ToolLoopTracker::new(opts),
        }
    }

    /// Check if a prospective tool call triggers a loop intervention.
    pub fn check(&self, tool: &ToolCall) -> Result<(), String> {
        if self.tracker.is_breaker_tripped(&tool.name) {
            return Err(format!(
                "LOOP BREAKER TRIPPED: Tool '{}' has been repeatedly vetoed without progress. Circuit broken — please report outcome directly to the user.",
                tool.name
            ));
        }

        let res = self.tracker.check(&tool.name, &tool.arguments);
        match res.level {
            LoopVerdict::Critical => Err(format!(
                "LOOP GUARD CRITICAL (No-Progress): Tool '{}' executed {} consecutive times with identical output. Please change approach or state result.",
                tool.name, res.count
            )),
            LoopVerdict::Wandering => {
                if self.tracker.is_wandering_escalated(&tool.name, &tool.arguments) {
                    Err(format!(
                        "LOOP GUARD BREAKER (Wandering): Tool '{}' executed {} distinct exploratory steps. Please summarize findings and reply.",
                        tool.name, res.count
                    ))
                } else {
                    Ok(())
                }
            }
            LoopVerdict::Warn => {
                // Advisory notice: do not veto execution (only Critical vetoes)
                Ok(())
            }
            LoopVerdict::Ok => Ok(()),
        }
    }

    /// Record prospective tool call invocation before execution.
    pub fn record_call(&mut self, tool: &ToolCall) {
        self.tracker.record_call(&tool.name, &tool.arguments);
    }

    /// Record tool execution outcome.
    pub fn record_outcome(&mut self, tool: &ToolCall, output: &str) {
        let res_hash = hash_tool_result(&tool.name, &tool.arguments, output);
        self.tracker.record_outcome(&tool.name, &tool.arguments, Some(res_hash));
    }

    /// Record veto when loop guard trips.
    pub fn record_veto(&mut self, tool: &ToolCall) {
        self.tracker.record_veto(&tool.name, &tool.arguments);
    }

    /// Convenience single-step record and check for compatibility.
    pub fn record_and_check(&mut self, tool: &ToolCall) -> Result<(), String> {
        self.check(tool)?;
        self.record_call(tool);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.tracker.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gbnf_tool_validation() {
        let valid_json = r#"{"name": "read_file", "arguments": {"path": "src/main.rs"}}"#;
        let parsed = validate_gbnf_tool_call(valid_json).unwrap();
        assert_eq!(parsed.name, "read_file");
        assert_eq!(parsed.arguments["path"], "src/main.rs");

        // Codeblock embedded within surrounding conversational text
        let codeblock_json = "I'll read that file for you:\n```json\n{\"tool\": \"grep_search\", \"args\": {\"query\": \"fn main\"}}\n```\nLet me know if you need more.";
        let parsed_cb = validate_gbnf_tool_call(codeblock_json).unwrap();
        assert_eq!(parsed_cb.name, "grep_search");
        assert_eq!(parsed_cb.arguments["query"], "fn main");

        // XML-style tool_call tag with flattened arguments
        let tag_json = "Sure, creating the file now:\n<tool_call>\n{\"name\": \"write_file\", \"path\": \"about.md\", \"content\": \"# About Me\"}\n</tool_call>";
        let parsed_tag = validate_gbnf_tool_call(tag_json).unwrap();
        assert_eq!(parsed_tag.name, "write_file");
        assert_eq!(parsed_tag.arguments["path"], "about.md");
        assert_eq!(parsed_tag.arguments["content"], "# About Me");

        let batch_json = r#"[{"tool": "read_file", "args": {"path": "a.txt"}}, {"tool": "grep_search", "args": {"query": "foo"}}]"#;
        let batch_parsed = validate_gbnf_tool_calls(batch_json).unwrap();
        assert_eq!(batch_parsed.len(), 2);
        assert_eq!(batch_parsed[0].name, "read_file");
        assert_eq!(batch_parsed[1].name, "grep_search");
    }

    #[test]
    fn test_batch_tool_calls_in_codeblocks() {
        let text = "First, let's read the file:\n```json\n[{\"name\": \"read_file\", \"path\": \"Cargo.toml\"}, {\"name\": \"list_dir\", \"path\": \".\"}]\n```\nDone.";
        let calls = validate_gbnf_tool_calls(text).unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].arguments["path"], "Cargo.toml");
        assert_eq!(calls[1].name, "list_dir");
        assert_eq!(calls[1].arguments["path"], ".");
    }

    #[test]
    fn test_loop_guard() {
        let mut guard = LoopGuard::new(ToolLoopTrackerOptions {
            warning_threshold: 3,
            critical_threshold: 3,
            ..Default::default()
        });
        let call = ToolCall {
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "foo.rs"}),
        };

        for _ in 0..2 {
            assert!(guard.check(&call).is_ok());
            guard.record_call(&call);
            guard.record_outcome(&call, "content");
        }
        let res = guard.check(&call);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("LOOP GUARD CRITICAL"));
    }

    #[test]
    fn test_tool_loop_tracker_args_only_repeat() {
        let mut tracker = ToolLoopTracker::new(ToolLoopTrackerOptions {
            warning_threshold: 3,
            ..Default::default()
        });
        let args = serde_json::json!({"path": "foo.rs"});

        // First two calls are ok
        let v = tracker.check("read_file", &args);
        assert_eq!(v.level, LoopVerdict::Ok);
        tracker.record_call("read_file", &args);
        tracker.record_outcome("read_file", &args, Some(100));

        let v = tracker.check("read_file", &args);
        assert_eq!(v.level, LoopVerdict::Ok);
        tracker.record_call("read_file", &args);
        tracker.record_outcome("read_file", &args, Some(200));

        // Third call triggers warn
        let v = tracker.check("read_file", &args);
        assert_eq!(v.level, LoopVerdict::Warn);
        assert_eq!(v.count, 2);
    }

    #[test]
    fn test_tool_loop_tracker_no_progress_streak() {
        let mut tracker = ToolLoopTracker::new(ToolLoopTrackerOptions {
            warning_threshold: 10,
            critical_threshold: 3,
            ..Default::default()
        });
        let args = serde_json::json!({"cmd": "cargo test"});

        // Same args + same result = no progress
        for _ in 0..2 {
            let v = tracker.check("execute_command", &args);
            assert_eq!(v.level, LoopVerdict::Ok);
            tracker.record_call("execute_command", &args);
            tracker.record_outcome("execute_command", &args, Some(42));
        }

        // Third triggers critical
        let v = tracker.check("execute_command", &args);
        assert_eq!(v.level, LoopVerdict::Critical);
        assert_eq!(v.detector, LoopDetector::NoProgress);
        assert_eq!(v.count, 2);
    }

    #[test]
    fn test_tool_loop_tracker_breaker() {
        let mut tracker = ToolLoopTracker::new(ToolLoopTrackerOptions {
            breaker_veto_streak: 2,
            critical_threshold: 2,
            ..Default::default()
        });
        let args = serde_json::json!({"cmd": "cargo build"});

        // First veto
        tracker.record_call("execute_command", &args);
        tracker.record_veto("execute_command", &args);
        assert!(!tracker.is_breaker_tripped("execute_command"));

        // Second veto trips breaker
        tracker.record_call("execute_command", &args);
        tracker.record_veto("execute_command", &args);
        assert!(tracker.is_breaker_tripped("execute_command"));
    }

    #[test]
    fn test_tool_loop_tracker_wandering() {
        let mut tracker = ToolLoopTracker::new(ToolLoopTrackerOptions {
            wandering_threshold: 3,
            ..Default::default()
        });

        // 3 distinct args on same wandering-prone tool
        for i in 0..3 {
            let args = serde_json::json!({"cmd": format!("cargo run {}", i)});
            let v = tracker.check("execute_command", &args);
            if i < 2 {
                assert_eq!(v.level, LoopVerdict::Ok);
            } else {
                assert_eq!(v.level, LoopVerdict::Wandering);
                assert_eq!(v.count, 3);
            }
            tracker.record_call("execute_command", &args);
            tracker.record_outcome("execute_command", &args, Some(i));
        }
    }

    #[test]
    fn test_tool_loop_tracker_different_args_different_result_breaks_streak() {
        let mut tracker = ToolLoopTracker::new(ToolLoopTrackerOptions {
            warning_threshold: 10,
            critical_threshold: 3,
            ..Default::default()
        });

        let args1 = serde_json::json!({"cmd": "cargo test"});
        let args2 = serde_json::json!({"cmd": "cargo build"});

        // Same args, same result x2
        tracker.record_call("execute_command", &args1);
        tracker.record_outcome("execute_command", &args1, Some(1));
        tracker.record_call("execute_command", &args1);
        tracker.record_outcome("execute_command", &args1, Some(1));

        // Different args (different result) — streak resets
        tracker.record_call("execute_command", &args2);
        tracker.record_outcome("execute_command", &args2, Some(99));

        // No-progress streak should be 0 (broken by different args/result)
        let v = tracker.check("execute_command", &args1);
        assert_eq!(v.level, LoopVerdict::Ok);
    }

    #[test]
    fn test_two_zone_prompt_compilation() {
        let zone_a = compile_zone_a_prefix("Name: Cynapse\nRole: Companion");
        assert!(zone_a.contains("You are Cynapse"));
        assert!(zone_a.contains("Operating Directives:"));
        assert!(zone_a.contains("Available Tools:"));
        assert!(zone_a.contains("read_file"));

        let zone_b = compile_zone_b_tail("## fact 1: Rust is fast", "USER: hello\nASSISTANT: hi", "Can you build it?", None);
        assert!(zone_b.contains("=== RECALLED KNOWLEDGE ==="));
        assert!(zone_b.contains("## fact 1: Rust is fast"));
        assert!(zone_b.contains("=== CONVERSATION LOG ==="));
        assert!(zone_b.contains("=== USER INSTRUCTION ==="));
        assert!(zone_b.contains("Can you build it?"));

        let zone_b_with_notice = compile_zone_b_tail(
            "## fact 1: Rust is fast",
            "USER: hello\nASSISTANT: hi",
            "Can you build it?",
            Some("Loop detector: tool repeated. Change strategy."),
        );
        assert!(zone_b_with_notice.contains("=== NOTICE ==="));
        assert!(zone_b_with_notice.contains("Loop detector: tool repeated. Change strategy."));
        // Notice must appear before USER INSTRUCTION
        let notice_idx = zone_b_with_notice.find("=== NOTICE ===").unwrap();
        let instruction_idx = zone_b_with_notice.find("=== USER INSTRUCTION ===").unwrap();
        assert!(notice_idx < instruction_idx);

        let conv_prefix = compile_conversational_prefix("Name: Cynapse\nRole: Companion");
        assert!(conv_prefix.contains("You are Cynapse"));
        assert!(!conv_prefix.contains("Available Tools:"));
        assert!(!conv_prefix.contains("read_file"));
    }

    #[test]
    fn test_loop_guard_warn_is_advisory() {
        let mut guard = LoopGuard::default();
        let tool = ToolCall {
            name: "read_file".to_string(),
            arguments: serde_json::json!({"path": "main.rs"}),
        };

        // Repeated invocations trigger Warn verdict, which is advisory (Ok(()))
        for _ in 0..4 {
            assert!(guard.check(&tool).is_ok());
            guard.record_call(&tool);
            guard.record_outcome(&tool, "fn main() {}");
        }
    }
}
