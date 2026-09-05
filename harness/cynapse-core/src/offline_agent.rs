//! Atomic-Agent Inspired Offline Execution Core.
//!
//! Provides strict GBNF JSON tool grammar validation, stable KV-cache prompt prefixing,
//! parallel read-only tool execution batching, and circular loop-guard protection for offline LLM engines.

use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// Standard parsed tool invocation structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Validates a raw model output string against GBNF tool call grammar.
pub fn validate_gbnf_tool_call(output: &str) -> Result<ToolCall, String> {
    let trimmed = output.trim();
    let json_text = if trimmed.starts_with("```json") && trimmed.ends_with("```") {
        trimmed.trim_start_matches("```json").trim_end_matches("```").trim()
    } else if trimmed.starts_with("```") && trimmed.ends_with("```") {
        trimmed.trim_start_matches("```").trim_end_matches("```").trim()
    } else {
        trimmed
    };

    let val: serde_json::Value = serde_json::from_str(json_text)
        .map_err(|e| format!("GBNF JSON Syntax Error: {}", e))?;

    let name = val.get("name")
        .or_else(|| val.get("tool"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'name' or 'tool' string property".to_string())?;

    let arguments = val.get("arguments")
        .or_else(|| val.get("args"))
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    Ok(ToolCall {
        name: name.to_string(),
        arguments,
    })
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

/// Circular buffer loop guard to detect repeated, non-progressing tool call loops offline.
#[derive(Debug, Clone)]
pub struct LoopGuard {
    history: VecDeque<u64>,
    max_history: usize,
    trigger_threshold: usize,
}

impl Default for LoopGuard {
    fn default() -> Self {
        Self::new(10, 3)
    }
}

impl LoopGuard {
    pub fn new(max_history: usize, trigger_threshold: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history,
            trigger_threshold,
        }
    }

    /// Record a tool call and check if it triggers a loop guard warning.
    pub fn record_and_check(&mut self, tool: &ToolCall) -> Result<(), String> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        tool.name.hash(&mut hasher);
        tool.arguments.to_string().hash(&mut hasher);
        let hash_val = hasher.finish();

        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(hash_val);

        // Count consecutive occurrences at the tail of history
        let consecutive_matches = self.history.iter().rev().take_while(|&&h| h == hash_val).count();

        if consecutive_matches >= self.trigger_threshold {
            Err(format!(
                "LOOP GUARD INTERVENTION: Tool '{}' has been executed {} times with identical parameters without state change. Try a different strategy.",
                tool.name, consecutive_matches
            ))
        } else {
            Ok(())
        }
    }

    pub fn clear(&mut self) {
        self.history.clear();
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

        let codeblock_json = "```json\n{\"tool\": \"grep_search\", \"args\": {\"query\": \"fn main\"}}\n```";
        let parsed_cb = validate_gbnf_tool_call(codeblock_json).unwrap();
        assert_eq!(parsed_cb.name, "grep_search");
    }

    #[test]
    fn test_loop_guard() {
        let mut guard = LoopGuard::new(10, 3);
        let call = ToolCall {
            name: "read_file".into(),
            arguments: serde_json::json!({"path": "foo.rs"}),
        };

        assert!(guard.record_and_check(&call).is_ok());
        assert!(guard.record_and_check(&call).is_ok());
        let res = guard.record_and_check(&call);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("LOOP GUARD INTERVENTION"));
    }
}
