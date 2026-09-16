//! Tool Output Compressor for Cynapse Agent System.
//!
//! Inspired by Atomic Agent's `result-compressor.ts`:
//! Shrinks verbose tool output (cargo build logs, test runs, grep results, file reads)
//! into a compact summary that preserves error signatures and the trailing execution tail
//! without exhausting small model context windows.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressedResult {
    pub summary: String,
    pub signature: Option<String>,
    pub truncated: bool,
    pub original_len: usize,
}

pub struct CompressorOptions {
    pub max_summary_length: usize,
    pub max_tail_lines: usize,
}

impl Default for CompressorOptions {
    fn default() -> Self {
        Self {
            max_summary_length: 1200,
            max_tail_lines: 20,
        }
    }
}

const ERROR_MARKERS: &[&str] = &[
    "error:",
    "error[",
    "failed:",
    "panic:",
    "fatal:",
    "exception:",
    "traceback",
    "assertionerror",
    "cannot find",
    "undefined",
    "syntax error",
];

/// Extracts key diagnostic signature lines if output indicates error or failure.
fn extract_signature(text: &str, is_error: bool) -> Option<String> {
    if !is_error {
        return None;
    }
    let mut sigs = Vec::new();
    for line in text.lines() {
        let lower = line.to_lowercase();
        for marker in ERROR_MARKERS {
            if lower.contains(marker) {
                let trimmed = line.trim();
                let sig = if trimmed.chars().count() > 250 {
                    trimmed.chars().take(250).collect::<String>() + "…"
                } else {
                    trimmed.to_string()
                };
                if !sigs.contains(&sig) {
                    sigs.push(sig);
                }
                break;
            }
        }
        if sigs.len() >= 3 {
            break;
        }
    }
    if sigs.is_empty() {
        None
    } else {
        Some(sigs.iter().map(|s| format!("key: {}", s)).collect::<Vec<_>>().join("\n"))
    }
}

/// Extracts trailing lines while skipping massive intermediate logs.
fn extract_tail(text: &str, max_lines: usize) -> (String, bool) {
    if text.is_empty() {
        return (String::new(), false);
    }
    let non_empty_lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if non_empty_lines.len() <= max_lines {
        return (non_empty_lines.join("\n"), false);
    }
    let omitted = non_empty_lines.len() - max_lines;
    let tail_slice = &non_empty_lines[omitted..];
    let result = format!("… [omitted {} lines]\n{}", omitted, tail_slice.join("\n"));
    (result, true)
}

/// Compresses a raw tool output string into a compact representation.
pub fn compress_tool_result(raw: &str, is_error: bool, options: Option<CompressorOptions>) -> CompressedResult {
    let opts = options.unwrap_or_default();
    let original_len = raw.len();
    let normalised = raw.replace("\r\n", "\n").trim_end().to_string();

    let (tail, tail_truncated) = extract_tail(&normalised, opts.max_tail_lines);
    let signature = extract_signature(&normalised, is_error);

    let mut parts = Vec::new();
    if let Some(ref sig) = signature {
        parts.push(sig.clone());
    }
    if !tail.is_empty() {
        parts.push(tail);
    }

    let joined = parts.join("\n");
    let over_length = joined.chars().count() > opts.max_summary_length;

    let summary = if over_length {
        let mut truncated_text: String = joined.chars().take(opts.max_summary_length.saturating_sub(15)).collect();
        truncated_text.push_str("\n… [truncated]");
        truncated_text
    } else {
        joined
    };

    CompressedResult {
        summary,
        signature,
        truncated: tail_truncated || over_length,
        original_len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_short_output() {
        let text = "Hello world\nSecond line";
        let res = compress_tool_result(text, false, None);
        assert_eq!(res.summary, text);
        assert!(!res.truncated);
        assert_eq!(res.signature, None);
    }

    #[test]
    fn test_compress_error_signature_extracted() {
        let text = "Compiling cynapse...\nerror: unresolved import `super::foo`\n   --> src/lib.rs:10\nBuild failed.";
        let res = compress_tool_result(text, true, None);
        assert!(res.signature.is_some());
        assert!(res.signature.as_ref().unwrap().contains("key: error: unresolved import"));
    }

    #[test]
    fn test_compress_multi_error_signatures() {
        let text = "Compiling...\nerror[E0425]: cannot find value `foo`\nerror[E0425]: cannot find value `bar`\nfatal: aborting";
        let res = compress_tool_result(text, true, None);
        assert!(res.signature.is_some());
        let sig = res.signature.unwrap();
        assert!(sig.contains("key: error[E0425]: cannot find value `foo`"));
        assert!(sig.contains("key: error[E0425]: cannot find value `bar`"));
    }

    #[test]
    fn test_compress_tail_omission() {
        let mut lines = Vec::new();
        for i in 1..=50 {
            lines.push(format!("Log line {}", i));
        }
        let text = lines.join("\n");
        let res = compress_tool_result(&text, false, Some(CompressorOptions { max_summary_length: 1000, max_tail_lines: 5 }));
        assert!(res.truncated);
        assert!(res.summary.contains("… [omitted 45 lines]"));
        assert!(res.summary.contains("Log line 50"));
    }
}
