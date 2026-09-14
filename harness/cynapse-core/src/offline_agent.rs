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
/// sliding conversation history (with compressed tool results), and the active user instruction.
pub fn compile_zone_b_tail(
    recalled_knowledge: &str,
    conversation_history: &str,
    user_query: &str,
) -> String {
    let mut tail = String::new();

    if !recalled_knowledge.trim().is_empty() {
        tail.push_str("=== RECALLED KNOWLEDGE ===\n");
        tail.push_str(recalled_knowledge.trim());
        tail.push_str("\n\n");
    }

    if !conversation_history.trim().is_empty() {
        tail.push_str("=== CONVERSATION LOG ===\n");
        tail.push_str(conversation_history.trim());
        tail.push_str("\n\n");
    }

    tail.push_str("=== USER INSTRUCTION ===\n");
    tail.push_str(user_query.trim());
    tail
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

    #[test]
    fn test_two_zone_prompt_compilation() {
        let zone_a = compile_zone_a_prefix("Name: Cynapse\nRole: Companion");
        assert!(zone_a.contains("You are Cynapse"));
        assert!(zone_a.contains("Operating Directives:"));
        assert!(zone_a.contains("Available Tools:"));
        assert!(zone_a.contains("read_file"));

        let zone_b = compile_zone_b_tail("## fact 1: Rust is fast", "USER: hello\nASSISTANT: hi", "Can you build it?");
        assert!(zone_b.contains("=== RECALLED KNOWLEDGE ==="));
        assert!(zone_b.contains("## fact 1: Rust is fast"));
        assert!(zone_b.contains("=== CONVERSATION LOG ==="));
        assert!(zone_b.contains("=== USER INSTRUCTION ==="));
        assert!(zone_b.contains("Can you build it?"));

        let conv_prefix = compile_conversational_prefix("Name: Cynapse\nRole: Companion");
        assert!(conv_prefix.contains("You are Cynapse"));
        assert!(!conv_prefix.contains("Available Tools:"));
        assert!(!conv_prefix.contains("read_file"));
    }
}
