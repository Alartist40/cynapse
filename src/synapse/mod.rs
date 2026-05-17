//! Synapse discovery and plugin system.
//!
//! Synapses are discovered as executables in a directory that respond to `--meta`
//! with JSON metadata. This is the same protocol used by the full Cynapse.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{CynapseError, Result};

/// Metadata emitted by a synapse when called with `--meta`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Registry of discovered synapses.
pub struct SynapseRegistry {
    synapses: HashMap<String, SynapseMeta>,
}

impl SynapseRegistry {
    pub fn new() -> Self {
        Self {
            synapses: HashMap::new(),
        }
    }

    /// Scan a directory for synapse binaries.
    pub fn discover(&mut self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            // Skip non-executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let meta = entry.metadata()?;
                if meta.permissions().mode() & 0o111 == 0 {
                    continue;
                }
            }

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }

                if let Ok(meta) = load_synapse_meta(&path) {
                    tracing::info!("Loaded synapse: {} v{}", meta.name, meta.version);
                    self.synapses.insert(meta.name.clone(), meta);
                }
            }
        }

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&SynapseMeta> {
        self.synapses.get(name)
    }

    pub fn list(&self) -> Vec<&SynapseMeta> {
        self.synapses.values().collect()
    }

    pub fn len(&self) -> usize {
        self.synapses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.synapses.is_empty()
    }
}

fn load_synapse_meta(path: &Path) -> Result<SynapseMeta> {
    let output = Command::new(path)
        .arg("--meta")
        .output()
        .map_err(|e| CynapseError::ToolError(format!("Failed to query synapse: {}", e)))?;

    if !output.status.success() {
        return Err(CynapseError::ToolError(
            format!("Synapse --meta failed: {}", String::from_utf8_lossy(&output.stderr))
        ));
    }

    let meta: SynapseMeta = serde_json::from_slice(&output.stdout)
        .map_err(|e| CynapseError::ToolError(format!("Invalid synapse metadata: {}", e)))?;

    Ok(meta)
}

/// Install a synapse from a local binary path.
pub fn install_from_path(
    name: &str,
    synapse_dir: impl AsRef<Path>,
    source: impl AsRef<Path>,
) -> Result<PathBuf> {
    let synapse_dir = synapse_dir.as_ref();
    std::fs::create_dir_all(synapse_dir)?;

    let dest = synapse_dir.join(name);
    std::fs::copy(&source, &dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dest)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dest, perms)?;
    }

    // Verify it responds to --meta
    let _meta = load_synapse_meta(&dest)?;
    tracing::info!("Installed synapse: {} -> {:?}", name, dest);

    Ok(dest)
}
