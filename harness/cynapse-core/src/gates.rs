//! Native Deterministic Acceptance Gate Engine.
//!
//! Inspired by the Universal Skill's `unlazy` directive and Atomic Agent verification gates:
//! Parses, executes, and records verifiable machine evidence for observable acceptance gates in `GATES.md`.
//! Eliminates LLM self-evaluation hallucination by grounding task completion in deterministic exit codes.

use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Represents a single observable acceptance gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptanceGate {
    pub id: String,
    pub description: String,
    pub check: String,
    pub expect: Option<String>,
    pub evidence: String,
    pub passed: bool,
}

/// Result of evaluating an acceptance gate suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateSuiteResult {
    pub total: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub gates: Vec<AcceptanceGate>,
    pub summary: String,
}

/// Parses a GATES.md formatted string into a list of AcceptanceGate structures.
pub fn parse_gates(content: &str) -> Vec<AcceptanceGate> {
    let mut gates = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_desc: Option<String> = None;
    let mut current_check: Option<String> = None;
    let mut current_expect: Option<String> = None;
    let mut current_evidence: Option<String> = None;
    let mut current_passed = false;

    let flush_gate = |gates: &mut Vec<AcceptanceGate>,
                      id: &mut Option<String>,
                      desc: &mut Option<String>,
                      check: &mut Option<String>,
                      expect: &mut Option<String>,
                      evidence: &mut Option<String>,
                      passed: &mut bool| {
        if let (Some(gid), Some(gcheck)) = (id.take(), check.take()) {
            gates.push(AcceptanceGate {
                id: gid,
                description: desc.take().unwrap_or_default(),
                check: gcheck,
                expect: expect.take(),
                evidence: evidence.take().unwrap_or_else(|| "pending".to_string()),
                passed: *passed,
            });
            *passed = false;
        }
    };

    for line in content.lines() {
        let trimmed = line.trim();

        // Match gate header: "- [ ] G1: Build passes" or "- [x] G1: Build passes"
        if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            flush_gate(
                &mut gates,
                &mut current_id,
                &mut current_desc,
                &mut current_check,
                &mut current_expect,
                &mut current_evidence,
                &mut current_passed,
            );

            let is_checked = trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]");
            current_passed = is_checked;

            let after_box = if trimmed.len() > 5 { &trimmed[5..].trim() } else { "" };
            if let Some((id_part, desc_part)) = after_box.split_once(':') {
                current_id = Some(id_part.trim().to_string());
                current_desc = Some(desc_part.trim().to_string());
            } else {
                current_id = Some(after_box.to_string());
                current_desc = Some(String::new());
            }
        } else if trimmed.starts_with("CHECK:") {
            current_check = Some(trimmed["CHECK:".len()..].trim().to_string());
        } else if trimmed.starts_with("EXPECT:") {
            current_expect = Some(trimmed["EXPECT:".len()..].trim().to_string());
        } else if trimmed.starts_with("EVIDENCE:") {
            current_evidence = Some(trimmed["EVIDENCE:".len()..].trim().to_string());
        }
    }

    flush_gate(
        &mut gates,
        &mut current_id,
        &mut current_desc,
        &mut current_check,
        &mut current_expect,
        &mut current_evidence,
        &mut current_passed,
    );

    gates
}

/// Executes a single gate command with timeout and validates stdout/stderr.
pub fn execute_gate_check(gate: &mut AcceptanceGate, timeout: Duration) {
    let check_cmd = gate.check.trim();
    if check_cmd.is_empty() {
        gate.passed = false;
        gate.evidence = "failed (empty check command)".to_string();
        return;
    }

    let mut child = match Command::new("bash")
        .arg("-c")
        .arg(check_cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            gate.passed = false;
            gate.evidence = format!("failed to spawn check: {}", e);
            return;
        }
    };

    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if let Ok(Some(status)) = child.try_wait() {
            let output = child.wait_with_output().unwrap_or_else(|_| std::process::Output {
                status,
                stdout: Vec::new(),
                stderr: Vec::new(),
            });
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);

            if status.success() {
                if let Some(ref exp) = gate.expect {
                    if combined.contains(exp) {
                        gate.passed = true;
                        gate.evidence = format!("met (exit 0, found '{}')", exp);
                    } else {
                        gate.passed = false;
                        gate.evidence = format!("failed (exit 0, missing '{}')", exp);
                    }
                } else {
                    gate.passed = true;
                    gate.evidence = "met (exit 0)".to_string();
                }
            } else {
                gate.passed = false;
                gate.evidence = format!("failed (exit code {:?})", status.code());
            }
            return;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = child.kill();
    gate.passed = false;
    gate.evidence = format!("timed out after {}s", timeout.as_secs());
}

/// Formats gates back into canonical Markdown format.
pub fn format_gates_markdown(gates: &[AcceptanceGate]) -> String {
    let mut out = String::from("# Acceptance Gates\n\n");
    for g in gates {
        let mark = if g.passed { "x" } else { " " };
        out.push_str(&format!("- [{}] {}: {}\n", mark, g.id, g.description));
        out.push_str(&format!("  CHECK: {}\n", g.check));
        if let Some(ref exp) = g.expect {
            out.push_str(&format!("  EXPECT: {}\n", exp));
        }
        out.push_str(&format!("  EVIDENCE: {}\n\n", g.evidence));
    }
    out
}

/// Executes all gates in the specified file (or default `GATES.md`), updates the file with verifiable evidence,
/// and returns a structured summary string.
pub fn run_gates(path_opt: Option<&str>) -> Result<String> {
    let path_str = path_opt.unwrap_or("GATES.md");
    let path = PathBuf::from(path_str);

    if !path.exists() {
        bail!("Acceptance gates file not found: {}", path.display());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read gates file {}", path.display()))?;

    let mut gates = parse_gates(&content);
    if gates.is_empty() {
        bail!("No acceptance gates found in {}", path.display());
    }

    let mut passed_count = 0;
    let timeout = Duration::from_secs(15);

    for gate in &mut gates {
        execute_gate_check(gate, timeout);
        if gate.passed {
            passed_count += 1;
        }
    }

    let total = gates.len();
    let failed_count = total - passed_count;

    // Update markdown file on disk with verified evidence
    let updated_md = format_gates_markdown(&gates);
    let _ = fs::write(&path, updated_md);

    let mut summary = format!(
        "=== ACCEPTANCE GATES EVALUATION ===\nTotal: {} | Passed: {} | Failed: {}\n\n",
        total, passed_count, failed_count
    );

    for g in &gates {
        let mark = if g.passed { "[x]" } else { "[ ]" };
        summary.push_str(&format!("{} {}: {} — {}\n", mark, g.id, g.description, g.evidence));
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gates_and_formatting() {
        let doc = r#"
# Project Gates

- [ ] G1: Short build
  CHECK: echo "building..."
  EXPECT: building
  EVIDENCE: pending

- [x] G2: Unit tests
  CHECK: echo "test result: ok"
  EXPECT: ok
  EVIDENCE: met (exit 0)
"#;
        let gates = parse_gates(doc);
        assert_eq!(gates.len(), 2);
        assert_eq!(gates[0].id, "G1");
        assert_eq!(gates[0].description, "Short build");
        assert_eq!(gates[0].check, "echo \"building...\"");
        assert_eq!(gates[0].expect.as_deref(), Some("building"));
        assert!(!gates[0].passed);

        assert_eq!(gates[1].id, "G2");
        assert!(gates[1].passed);

        let formatted = format_gates_markdown(&gates);
        assert!(formatted.contains("- [ ] G1: Short build"));
        assert!(formatted.contains("- [x] G2: Unit tests"));
    }

    #[test]
    fn test_execute_gate_check_success_and_failure() {
        let mut g1 = AcceptanceGate {
            id: "G1".into(),
            description: "Success echo".into(),
            check: "echo 'hello world'".into(),
            expect: Some("hello".into()),
            evidence: "pending".into(),
            passed: false,
        };
        execute_gate_check(&mut g1, Duration::from_secs(3));
        assert!(g1.passed);
        assert!(g1.evidence.contains("met (exit 0"));

        let mut g2 = AcceptanceGate {
            id: "G2".into(),
            description: "Missing expect token".into(),
            check: "echo 'goodbye'".into(),
            expect: Some("nonexistent_token".into()),
            evidence: "pending".into(),
            passed: false,
        };
        execute_gate_check(&mut g2, Duration::from_secs(3));
        assert!(!g2.passed);
        assert!(g2.evidence.contains("missing 'nonexistent_token'"));

        let mut g3 = AcceptanceGate {
            id: "G3".into(),
            description: "Failing exit code".into(),
            check: "exit 42".into(),
            expect: None,
            evidence: "pending".into(),
            passed: false,
        };
        execute_gate_check(&mut g3, Duration::from_secs(3));
        assert!(!g3.passed);
        assert!(g3.evidence.contains("failed (exit code Some(42))"));
    }
}
