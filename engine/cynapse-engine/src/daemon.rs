//! Managed llama-server child daemon lifecycle.
//!
//! Supervises an in-process child process running `llama-server` on `127.0.0.1:38265`.
//! Automatically locates `llama-server` binary, tunes hardware flags (`--parallel 2`,
//! `--flash-attn auto`, `--cache-type-k q8_0`, `-ngl -1`), monitors health probes,
//! and ensures clean process termination on drop.

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use anyhow::{bail, Result};

pub const DEFAULT_DAEMON_PORT: u16 = 38265;

pub struct LlamaServerDaemon {
    child: Option<Child>,
    pub port: u16,
    pub model_path: PathBuf,
}

impl LlamaServerDaemon {
    /// Discovers `llama-server` executable in system PATH or ~/.cynapse/bin/
    pub fn find_binary() -> Option<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let user_bin = PathBuf::from(home).join(".cynapse").join("bin").join("llama-server");
            if user_bin.is_file() {
                return Some(user_bin);
            }
        }

        if let Ok(path_env) = std::env::var("PATH") {
            for entry in std::env::split_paths(&path_env) {
                let candidate = entry.join("llama-server");
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        let fallback_paths = [
            "/usr/local/lib/ollama/llama-server",
            "/usr/local/bin/llama-server",
            "/usr/bin/llama-server",
            "/opt/homebrew/bin/llama-server",
        ];
        for path in fallback_paths {
            let p = PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }

    /// Checks if a daemon is actively responding on the given port.
    pub fn is_healthy(port: u16) -> bool {
        let addr_str = format!("127.0.0.1:{}", port);
        if let Ok(addr) = addr_str.parse() {
            if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(200)) {
                drop(stream);
                return true;
            }
        }
        false
    }

    /// Spawns a supervised `llama-server` child process.
    pub fn spawn(model_path: &Path, port: u16) -> Result<Self> {
        let bin = match Self::find_binary() {
            Some(b) => b,
            None => bail!("`llama-server` binary not found in ~/.cynapse/bin/ or PATH"),
        };

        if !model_path.exists() {
            bail!("Model file not found: {}", model_path.display());
        }

        let log_dir = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".cynapse").join("logs")
        } else {
            PathBuf::from("./logs")
        };
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join("llama-server.log");
        let (out_stdio, err_stdio) = if let Ok(f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let err_f = f.try_clone().map(Stdio::from).unwrap_or_else(|_| Stdio::null());
            (Stdio::from(f), err_f)
        } else {
            (Stdio::null(), Stdio::null())
        };

        let child = Command::new(&bin)
            .arg("-m")
            .arg(model_path)
            .arg("--port")
            .arg(port.to_string())
            .arg("--host")
            .arg("127.0.0.1")
            .arg("-ngl")
            .arg("-1")
            .arg("-t")
            .arg("8")
            .arg("--flash-attn")
            .arg("auto")
            .arg("--cache-type-k")
            .arg("q8_0")
            .arg("--cache-type-v")
            .arg("q8_0")
            .arg("--parallel")
            .arg("1")
            .arg("--ctx-size")
            .arg("4096")
            .arg("--no-webui")
            .arg("--jinja")
            .stdout(out_stdio)
            .stderr(err_stdio)
            .spawn()?;

        let mut daemon = Self {
            child: Some(child),
            port,
            model_path: model_path.to_path_buf(),
        };

        // Poll health for up to 10 seconds
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(10) {
            if Self::is_healthy(port) {
                return Ok(daemon);
            }
            std::thread::sleep(Duration::from_millis(250));
        }

        daemon.shutdown();
        bail!("`llama-server` did not become healthy within 10 seconds. Check logs at {}", log_path.display());
    }

    /// Singleton daemon spawner: checks health and spawns if binary and model exist.
    pub fn get_or_spawn_daemon(model_path: &Path, port: u16) -> bool {
        static ACTIVE_DAEMON: std::sync::OnceLock<std::sync::Mutex<Option<LlamaServerDaemon>>> = std::sync::OnceLock::new();
        if Self::is_healthy(port) {
            return true;
        }

        let mutex = ACTIVE_DAEMON.get_or_init(|| std::sync::Mutex::new(None));
        let mut guard = match mutex.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        if Self::is_healthy(port) {
            return true;
        }

        match Self::spawn(model_path, port) {
            Ok(d) => {
                *guard = Some(d);
                true
            }
            Err(_) => false,
        }
    }

    /// Graceful shutdown of the child process.
    pub fn shutdown(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for LlamaServerDaemon {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_health_check_offline() {
        // High random unassigned port should report not healthy
        assert!(!LlamaServerDaemon::is_healthy(59991));
    }
}
