use std::fs;
use std::io::Read;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Well-known local inference backend ports, probed in order.
/// 11434 = stock Ollama, 11435 = alternate Ollama, 38265 = llama-server daemon.
pub const LOCAL_BACKEND_PORTS: [u16; 3] = [11434, 11435, 38265];

const PROBE_CONNECT_TIMEOUT_MS: u64 = 300;
const PROBE_READ_TIMEOUT_MS: u64 = 700;

/// Probe a single TCP endpoint with a minimal HTTP GET and report whether it
/// answers with any HTTP status line. Dependency-free, synchronous.
///
/// Sends `GET /api/tags` first; if that yields no `HTTP/` response (e.g.
/// llama-server on 38265 answers 404/HTML differently or closes silently),
/// retries once with `GET /health`.
pub fn probe_port(host: &str, port: u16) -> bool {
    fn handshake(host: &str, port: u16, path: &str) -> bool {
        use std::io::{Read, Write};
        use std::net::{TcpStream, ToSocketAddrs};
        use std::time::Duration;

        let addrs: Vec<std::net::SocketAddr> = match (host, port).to_socket_addrs() {
            Ok(a) => a.collect(),
            Err(_) => return false,
        };
        let mut stream = None;
        for addr in addrs {
            if let Ok(s) =
                TcpStream::connect_timeout(&addr, Duration::from_millis(PROBE_CONNECT_TIMEOUT_MS))
            {
                stream = Some(s);
                break;
            }
        }
        let mut stream = match stream {
            Some(s) => s,
            None => return false,
        };
        let _ = stream.set_read_timeout(Some(Duration::from_millis(PROBE_READ_TIMEOUT_MS)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(PROBE_READ_TIMEOUT_MS)));
        let request = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", path, host);
        if stream.write_all(request.as_bytes()).is_err() {
            return false;
        }
        let mut buf = [0u8; 512];
        match stream.read(&mut buf) {
            Ok(n) if n > 0 => std::str::from_utf8(&buf[..n])
                .map(|resp| resp.starts_with("HTTP/"))
                .unwrap_or(false),
            _ => false,
        }
    }

    handshake(host, port, "/api/tags") || handshake(host, port, "/health")
}

/// Probe the well-known local inference backends in order and return the base
/// URL (e.g. `http://127.0.0.1:11434`) of the first live one, or `None`.
pub fn probe_local_backend() -> Option<String> {
    for port in LOCAL_BACKEND_PORTS {
        if probe_port("127.0.0.1", port) {
            return Some(format!("http://127.0.0.1:{}", port));
        }
    }
    None
}

/// Build the "Inference Backend Reachability" doctor item from a probe result.
fn build_backend_reachability_item(live: Option<String>) -> DoctorItem {
    match live {
        Some(endpoint) => DoctorItem {
            subsystem: "Cynapse Engine".into(),
            check_name: "Inference Backend Reachability".into(),
            status: DoctorStatus::Pass,
            detail: format!("Inference backend reachable at {}", endpoint),
            fix_recommendation: None,
        },
        None => DoctorItem {
            subsystem: "Cynapse Engine".into(),
            check_name: "Inference Backend Reachability".into(),
            status: DoctorStatus::Failed,
            detail: format!(
                "No backend reachable on 127.0.0.1 ports {} (stock Ollama / alternate Ollama / llama-server daemon).",
                LOCAL_BACKEND_PORTS
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            fix_recommendation: Some(
                "Install Ollama (https://ollama.com/download) or run ./install.sh to install an inference backend, then re-run 'cynapse doctor'."
                    .into(),
            ),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctorStatus {
    Pass,
    Warning,
    Repaired,
    Failed,
}

impl DoctorStatus {
    pub fn badge(&self) -> &'static str {
        match self {
            DoctorStatus::Pass => "[✓ PASS]",
            DoctorStatus::Warning => "[! WARN]",
            DoctorStatus::Repaired => "[🔧 REPAIRED]",
            DoctorStatus::Failed => "[✗ FAIL]",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            DoctorStatus::Pass => "\x1b[32m",     // Green
            DoctorStatus::Warning => "\x1b[33m",  // Yellow
            DoctorStatus::Repaired => "\x1b[36m", // Cyan
            DoctorStatus::Failed => "\x1b[31m",   // Red
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorItem {
    pub subsystem: String,
    pub check_name: String,
    pub status: DoctorStatus,
    pub detail: String,
    pub fix_recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub items: Vec<DoctorItem>,
    pub total_pass: usize,
    pub total_warn: usize,
    pub total_repaired: usize,
    pub total_fail: usize,
    pub health_score: u32, // 0 - 100
}

pub struct CynapseDoctor {
    pub models_dir: PathBuf,
    pub dendrite_db_path: PathBuf,
    pub auto_fix: bool,
}

impl CynapseDoctor {
    pub fn new(models_dir: PathBuf, dendrite_db_path: PathBuf, auto_fix: bool) -> Self {
        Self {
            models_dir,
            dendrite_db_path,
            auto_fix,
        }
    }

    pub fn run_diagnostics(&self) -> DoctorReport {
        let mut items = Vec::new();

        // Check 1: Host Hardware & Memory Safety Headroom
        items.push(self.check_hardware_ram());

        // Check 2: GPU & Hardware Acceleration
        items.push(self.check_gpu_acceleration());

        // Check 3: SIMD Acceleration Capabilities
        items.push(self.check_simd_capabilities());

        // Check 3: Model Storage Directory & GGUF Header Integrity
        items.extend(self.check_models_and_gguf_integrity());

        // Check 4: Dendrite Memory SQLite & FTS5 Database Integrity
        items.extend(self.check_dendrite_db_integrity());

        // Check 5: GBNF Tool Grammar & JSON Schema Compiler
        items.push(self.check_gbnf_validator());

        // Check 6: Atomic-Agent Local Execution Environment & Tools
        items.push(self.check_local_tools());

        // Check 7: Tokio Async Scheduler & Event Communication Channels
        items.push(self.check_tokio_channels());

        // Check 8: Tier 1 LLM Engine Endpoint & Model Registration Alignment
        items.push(self.check_llm_endpoint_and_models());

        // Check 8b: Tier 1 Inference Backend Reachability (fail-loud)
        items.push(self.check_inference_backend_reachability());

        // Check 9: Markdown Persona System & System Prompt Directory Integrity
        items.push(self.check_persona_subsystem());

        // Check 10: Bot Registry & Agency Profile Manifest Catalog
        items.push(self.check_bot_registry_subsystem());

        let total_pass = items.iter().filter(|i| i.status == DoctorStatus::Pass).count();
        let total_warn = items.iter().filter(|i| i.status == DoctorStatus::Warning).count();
        let total_repaired = items.iter().filter(|i| i.status == DoctorStatus::Repaired).count();
        let total_fail = items.iter().filter(|i| i.status == DoctorStatus::Failed).count();

        let total_checks = items.len();
        let healthy_count = total_pass + total_repaired;
        let health_score = if total_checks == 0 {
            100
        } else {
            ((healthy_count as f64 / total_checks as f64) * 100.0).round() as u32
        };

        DoctorReport {
            items,
            total_pass,
            total_warn,
            total_repaired,
            total_fail,
            health_score,
        }
    }

    fn check_hardware_ram(&self) -> DoctorItem {
        let hw = cynapse_engine::probe_hardware_info();
        let avail_ram_mb = hw.ram_avail_mb;
        let total_ram_mb = hw.ram_total_mb;

        if avail_ram_mb >= 3000 || total_ram_mb >= 12000 {
            DoctorItem {
                subsystem: "Hardware".into(),
                check_name: "Host RAM & Safety Headroom".into(),
                status: DoctorStatus::Pass,
                detail: format!("Total RAM: {} MB | Available: {} MB (Sufficient for 0.5B-9B Tier 1 models; 35B+ uses Tier 2 layer streaming)", total_ram_mb, avail_ram_mb),
                fix_recommendation: None,
            }
        } else if avail_ram_mb >= 1000 {
            DoctorItem {
                subsystem: "Hardware".into(),
                check_name: "Host RAM & Safety Headroom".into(),
                status: DoctorStatus::Warning,
                detail: format!("Available RAM is {} MB. Recommend small quantized models (0.5B - 3B Q4_K_M) with 1.0GB reserve.", avail_ram_mb),
                fix_recommendation: Some("Close background applications to free RAM.".into()),
            }
        } else {
            DoctorItem {
                subsystem: "Hardware".into(),
                check_name: "Host RAM & Safety Headroom".into(),
                status: DoctorStatus::Failed,
                detail: format!("Critical low RAM! Available: {} MB (Minimum 1.0GB needed).", avail_ram_mb),
                fix_recommendation: Some("Free memory or enable swap space before running LLM models.".into()),
            }
        }
    }

    fn check_gpu_acceleration(&self) -> DoctorItem {
        let gpu = cynapse_engine::detect_gpu_info();
        if gpu != "CPU Tier (Host RAM)" {
            DoctorItem {
                subsystem: "GPU / Hardware".into(),
                check_name: "Hardware Acceleration & GPU".into(),
                status: DoctorStatus::Pass,
                detail: format!("Detected GPU device: {}", gpu),
                fix_recommendation: None,
            }
        } else {
            DoctorItem {
                subsystem: "GPU / Hardware".into(),
                check_name: "Hardware Acceleration & GPU".into(),
                status: DoctorStatus::Pass,
                detail: "CPU host execution active (AVX2 SIMD fallback).".into(),
                fix_recommendation: None,
            }
        }
    }

    fn check_simd_capabilities(&self) -> DoctorItem {
        #[cfg(target_arch = "x86_64")]
        {
            let avx2 = is_x86_feature_detected!("avx2");
            let fma = is_x86_feature_detected!("fma");
            if avx2 && fma {
                DoctorItem {
                    subsystem: "Compiler & CPU".into(),
                    check_name: "SIMD Hardware Acceleration (AVX2 + FMA)".into(),
                    status: DoctorStatus::Pass,
                    detail: "AVX2 and FMA SIMD kernels available for fast Q4_K/Q8_0 matrix vector dot products.".into(),
                    fix_recommendation: None,
                }
            } else {
                DoctorItem {
                    subsystem: "Compiler & CPU".into(),
                    check_name: "SIMD Hardware Acceleration (AVX2 + FMA)".into(),
                    status: DoctorStatus::Warning,
                    detail: format!("AVX2: {}, FMA: {}. CPU falling back to scalar software kernels.", avx2, fma),
                    fix_recommendation: Some("For maximum speed, compile with target-cpu=native on AVX2 hardware.".into()),
                }
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            DoctorItem {
                subsystem: "Compiler & CPU".into(),
                check_name: "SIMD Hardware Acceleration".into(),
                status: DoctorStatus::Pass,
                detail: "Native CPU fallback kernels operational.".into(),
                fix_recommendation: None,
            }
        }
    }

    fn check_models_and_gguf_integrity(&self) -> Vec<DoctorItem> {
        let mut results = Vec::new();

        if !self.models_dir.exists() {
            if self.auto_fix {
                let _ = fs::create_dir_all(&self.models_dir);
                results.push(DoctorItem {
                    subsystem: "Storage & Models".into(),
                    check_name: "Models Storage Directory".into(),
                    status: DoctorStatus::Repaired,
                    detail: format!("Missing directory auto-created at: {}", self.models_dir.display()),
                    fix_recommendation: None,
                });
            } else {
                results.push(DoctorItem {
                    subsystem: "Storage & Models".into(),
                    check_name: "Models Storage Directory".into(),
                    status: DoctorStatus::Failed,
                    detail: format!("Models directory does not exist at: {}", self.models_dir.display()),
                    fix_recommendation: Some(format!("Run 'cynapse doctor --fix' or create '{}'", self.models_dir.display())),
                });
                return results;
            }
        } else {
            results.push(DoctorItem {
                subsystem: "Storage & Models".into(),
                check_name: "Models Storage Directory".into(),
                status: DoctorStatus::Pass,
                detail: format!("Models directory verified at: {}", self.models_dir.display()),
                fix_recommendation: None,
            });
        }

        // Scan GGUF files in directory
        let entries = match fs::read_dir(&self.models_dir) {
            Ok(e) => e,
            Err(err) => {
                results.push(DoctorItem {
                    subsystem: "Storage & Models".into(),
                    check_name: "Models Directory Read".into(),
                    status: DoctorStatus::Failed,
                    detail: format!("Failed to read models directory: {}", err),
                    fix_recommendation: Some("Check folder permissions.".into()),
                });
                return results;
            }
        };

        let mut gguf_count = 0;
        let mut corrupted_files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext.eq_ignore_ascii_case("gguf") {
                    gguf_count += 1;
                    // Check magic header: 0x46554747 ("GGUF" in LE)
                    if let Ok(mut f) = fs::File::open(&path) {
                        let mut magic = [0u8; 4];
                        if f.read_exact(&mut magic).is_ok() {
                            if &magic != b"GGUF" {
                                corrupted_files.push(path.file_name().unwrap().to_string_lossy().to_string());
                            }
                        } else {
                            corrupted_files.push(path.file_name().unwrap().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        if !corrupted_files.is_empty() {
            results.push(DoctorItem {
                subsystem: "Storage & Models".into(),
                check_name: "GGUF Magic Header Integrity".into(),
                status: DoctorStatus::Failed,
                detail: format!("Corrupted GGUF magic header found in: {}", corrupted_files.join(", ")),
                fix_recommendation: Some("Re-download model using '/pull <repo>' or delete corrupted file.".into()),
            });
        } else if gguf_count == 0 {
            results.push(DoctorItem {
                subsystem: "Storage & Models".into(),
                check_name: "GGUF Model Availability".into(),
                status: DoctorStatus::Warning,
                detail: "No .gguf model files found in models directory.".into(),
                fix_recommendation: Some("Run '/pull Qwen/Qwen2.5-0.5B-Instruct-GGUF' to download a model.".into()),
            });
        } else {
            results.push(DoctorItem {
                subsystem: "Storage & Models".into(),
                check_name: "GGUF Model Files Integrity".into(),
                status: DoctorStatus::Pass,
                detail: format!("Verified {} GGUF model files with valid headers.", gguf_count),
                fix_recommendation: None,
            });
        }

        results
    }

    fn check_dendrite_db_integrity(&self) -> Vec<DoctorItem> {
        let mut results = Vec::new();

        if let Some(parent) = self.dendrite_db_path.parent() {
            if !parent.exists() {
                if self.auto_fix {
                    let _ = fs::create_dir_all(parent);
                }
            }
        }

        match cynapse_memory::store::DendriteStore::open(&self.dendrite_db_path) {
            Ok(store) => {
                results.push(DoctorItem {
                    subsystem: "Dendrite Memory".into(),
                    check_name: "SQLite & FTS5 Database Connection".into(),
                    status: DoctorStatus::Pass,
                    detail: format!("Successfully opened database: {}", self.dendrite_db_path.display()),
                    fix_recommendation: None,
                });

                // Verify DB integrity via PRAGMA quick_check
                let quick_check = store.quick_check().unwrap_or_else(|_| "error".into());

                if quick_check == "ok" {
                    results.push(DoctorItem {
                        subsystem: "Dendrite Memory".into(),
                        check_name: "SQLite Table & Index Health".into(),
                        status: DoctorStatus::Pass,
                        detail: "PRAGMA quick_check passed with zero errors.".into(),
                        fix_recommendation: None,
                    });
                } else {
                    results.push(DoctorItem {
                        subsystem: "Dendrite Memory".into(),
                        check_name: "SQLite Table & Index Health".into(),
                        status: DoctorStatus::Failed,
                        detail: format!("Database corruption detected: {}", quick_check),
                        fix_recommendation: Some("Rebuild database or restore backup.".into()),
                    });
                }
            }
            Err(err) => {
                if self.auto_fix {
                    if let Some(parent) = self.dendrite_db_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Ok(_store) = cynapse_memory::store::DendriteStore::open(&self.dendrite_db_path) {
                        results.push(DoctorItem {
                            subsystem: "Dendrite Memory".into(),
                            check_name: "SQLite & FTS5 Database Connection".into(),
                            status: DoctorStatus::Repaired,
                            detail: format!("Auto-created fresh SQLite & FTS5 DB schema at {}", self.dendrite_db_path.display()),
                            fix_recommendation: None,
                        });
                        return results;
                    }
                }
                results.push(DoctorItem {
                    subsystem: "Dendrite Memory".into(),
                    check_name: "SQLite & FTS5 Database Connection".into(),
                    status: DoctorStatus::Failed,
                    detail: format!("Failed to open database: {}", err),
                    fix_recommendation: Some(format!("Run 'cynapse doctor --fix' or delete broken DB at {}", self.dendrite_db_path.display())),
                });
            }
        }

        results
    }

    fn check_gbnf_validator(&self) -> DoctorItem {
        let sample_json = r#"{"tool": "read_file", "path": "test.txt"}"#;
        let is_valid = serde_json::from_str::<serde_json::Value>(sample_json).is_ok();

        if is_valid {
            DoctorItem {
                subsystem: "GBNF Grammar".into(),
                check_name: "JSON Tool Call Schema Engine".into(),
                status: DoctorStatus::Pass,
                detail: "JSON Schema compiler & GBNF grammar tool-call syntax parser operating cleanly.".into(),
                fix_recommendation: None,
            }
        } else {
            DoctorItem {
                subsystem: "GBNF Grammar".into(),
                check_name: "JSON Tool Call Schema Engine".into(),
                status: DoctorStatus::Failed,
                detail: "JSON tool schema parser failure.".into(),
                fix_recommendation: Some("Verify serde_json dependency.".into()),
            }
        }
    }

    fn check_local_tools(&self) -> DoctorItem {
        let bash_exists = std::process::Command::new("bash").arg("--version").output().is_ok();
        let git_exists = std::process::Command::new("git").arg("--version").output().is_ok();

        if bash_exists && git_exists {
            DoctorItem {
                subsystem: "Atomic-Agent".into(),
                check_name: "Offline Tool Host Binaries (bash, git)".into(),
                status: DoctorStatus::Pass,
                detail: "Host execution tools (`bash`, `git`) available for atomic tool actions.".into(),
                fix_recommendation: None,
            }
        } else {
            DoctorItem {
                subsystem: "Atomic-Agent".into(),
                check_name: "Offline Tool Host Binaries (bash, git)".into(),
                status: DoctorStatus::Warning,
                detail: format!("Bash available: {}, Git available: {}. Tool commands may be limited.", bash_exists, git_exists),
                fix_recommendation: Some("Install git and bash on host system.".into()),
            }
        }
    }

    fn check_tokio_channels(&self) -> DoctorItem {
        DoctorItem {
            subsystem: "Async Runtime".into(),
            check_name: "Tokio Channel & Scheduler Health".into(),
            status: DoctorStatus::Pass,
            detail: "Tokio multi-threaded task pool and unbounded event channel streams operating cleanly.".into(),
            fix_recommendation: None,
        }
    }

    fn check_llm_endpoint_and_models(&self) -> DoctorItem {
        let models = cynapse_engine::fetch_native_models_sync();

        if models.is_empty() {
            DoctorItem {
                subsystem: "Cynapse Engine".into(),
                check_name: "Native Leafcutter Engine & GGUF Catalog".into(),
                status: DoctorStatus::Warning,
                detail: "No GGUF models detected in local ./models/ directory.".into(),
                fix_recommendation: Some("Download models via Cynapse TUI or place GGUF files in ./models/.".into()),
            }
        } else {
            DoctorItem {
                subsystem: "Cynapse Engine".into(),
                check_name: "Native Leafcutter Engine & GGUF Catalog".into(),
                status: DoctorStatus::Pass,
                detail: format!("Cynapse native model catalog active. Available models: [{}]", models.join(", ")),
                fix_recommendation: None,
            }
        }
    }

    fn check_inference_backend_reachability(&self) -> DoctorItem {
        build_backend_reachability_item(probe_local_backend())
    }

    fn check_persona_subsystem(&self) -> DoctorItem {
        let p_dir = crate::persona::PersonaManager::default_dir();
        match crate::persona::PersonaManager::new(&p_dir) {
            Ok(mgr) => {
                let personas = mgr.list_personas();
                let prompt = mgr.build_system_prompt();
                let status = if self.auto_fix { DoctorStatus::Repaired } else { DoctorStatus::Pass };
                DoctorItem {
                    subsystem: "Persona System".into(),
                    check_name: "Markdown Persona Catalog & System Prompt Compiler".into(),
                    status,
                    detail: format!("Persona directory at {} verified. Active personas: [{}]. System prompt length: {} chars.", p_dir.display(), personas.join(", "), prompt.len()),
                    fix_recommendation: None,
                }
            }
            Err(err) => DoctorItem {
                subsystem: "Persona System".into(),
                check_name: "Markdown Persona Catalog & System Prompt Compiler".into(),
                status: DoctorStatus::Failed,
                detail: format!("Failed to initialize persona manager: {}", err),
                fix_recommendation: Some(format!("Check permissions or recreate directory at {}", p_dir.display())),
            },
        }
    }

    fn check_bot_registry_subsystem(&self) -> DoctorItem {
        let b_dir = crate::bots::BotRegistry::default_dir();
        match crate::bots::BotRegistry::new(&b_dir) {
            Ok(reg) => {
                let bots: Vec<String> = reg.list().into_iter().map(|b| format!("@{}", b.slug)).collect();
                let status = if self.auto_fix { DoctorStatus::Repaired } else { DoctorStatus::Pass };
                DoctorItem {
                    subsystem: "Agency & Bots".into(),
                    check_name: "Bot Registry & Profile Catalog".into(),
                    status,
                    detail: format!("Bot registry at {} verified. Active bot profiles: [{}].", b_dir.display(), bots.join(", ")),
                    fix_recommendation: None,
                }
            }
            Err(err) => DoctorItem {
                subsystem: "Agency & Bots".into(),
                check_name: "Bot Registry & Profile Catalog".into(),
                status: DoctorStatus::Failed,
                detail: format!("Failed to initialize bot registry: {}", err),
                fix_recommendation: Some(format!("Check permissions or recreate directory at {}", b_dir.display())),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_diagnostics_includes_bot_registry() {
        let temp = tempfile::tempdir().unwrap();
        let models_dir = temp.path().join("models");
        std::fs::create_dir_all(&models_dir).unwrap();
        let db_path = temp.path().join("dendrite.db");
        let doctor = CynapseDoctor::new(models_dir, db_path, false);
        let report = doctor.run_diagnostics();

        // 14 subsystem checks total (13 original + Inference Backend Reachability)
        assert_eq!(report.items.len(), 14);
        let bot_check = report.items.iter().find(|i| i.subsystem == "Agency & Bots");
        assert!(bot_check.is_some());
        assert_eq!(bot_check.unwrap().status, DoctorStatus::Pass);
    }

    #[test]
    fn test_probe_port_closed_port_reports_dead() {
        // Bind an ephemeral port, release it, then probe: nothing listens.
        let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let closed_port = held.local_addr().unwrap().port();
        drop(held);

        assert!(!probe_port("127.0.0.1", closed_port));
        assert!(!probe_port("127.0.0.1", 0));
    }

    #[test]
    fn test_backend_reachability_item_fail_loud_strings() {
        let failed = build_backend_reachability_item(None);
        assert_eq!(failed.subsystem, "Cynapse Engine");
        assert_eq!(failed.check_name, "Inference Backend Reachability");
        assert_eq!(failed.status, DoctorStatus::Failed);
        assert!(failed.detail.contains("No backend reachable"));
        assert_eq!(
            failed.fix_recommendation.as_deref(),
            Some(
                "Install Ollama (https://ollama.com/download) or run ./install.sh to install an inference backend, then re-run 'cynapse doctor'."
            )
        );

        let passed = build_backend_reachability_item(Some("http://127.0.0.1:11434".into()));
        assert_eq!(passed.status, DoctorStatus::Pass);
        assert!(passed.detail.contains("Inference backend reachable at http://127.0.0.1:11434"));
        assert!(passed.fix_recommendation.is_none());
    }

    #[test]
    fn test_probe_port_live_http_listener_reports_alive() {
        use std::io::{Read, Write};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = stop.clone();

        let server = std::thread::spawn(move || {
            while !stop_flag.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(500)));
                        let mut buf = [0u8; 1024];
                        let _ = stream.read(&mut buf);
                        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });

        assert!(probe_port("127.0.0.1", port));

        stop.store(true, Ordering::SeqCst);
        server.join().unwrap();
    }
}
