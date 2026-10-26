use std::fs;
use std::path::PathBuf;
use anyhow::Result;

/// Monitors battery state and dynamically determines optimal thread allocation
/// to prevent thermal throttling on edge devices and laptops.
pub struct PowerThrottler {
    power_supply_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSource {
    AcPower,
    Battery,
    Unknown,
}

impl PowerThrottler {
    pub fn new() -> Self {
        Self {
            power_supply_path: PathBuf::from("/sys/class/power_supply"),
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            power_supply_path: path,
        }
    }

    /// Detects current power source from Linux /sys/class/power_supply/
    pub fn detect_power_source(&self) -> PowerSource {
        if !self.power_supply_path.exists() {
            return PowerSource::Unknown;
        }

        if let Ok(entries) = fs::read_dir(&self.power_supply_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                
                // Check AC adapter online status (e.g. ACAD, AC, ADP1)
                if file_name.starts_with("AC") || file_name.starts_with("ADP") {
                    let online_file = path.join("online");
                    if let Ok(val) = fs::read_to_string(online_file) {
                        if val.trim() == "1" {
                            return PowerSource::AcPower;
                        }
                    }
                }

                // Check battery status
                if file_name.starts_with("BAT") {
                    let status_file = path.join("status");
                    if let Ok(val) = fs::read_to_string(status_file) {
                        let status = val.trim().to_lowercase();
                        if status == "discharging" {
                            return PowerSource::Battery;
                        } else if status == "charging" || status == "full" {
                            return PowerSource::AcPower;
                        }
                    }
                }
            }
        }

        PowerSource::Unknown
    }

    /// Computes optimal worker thread count based on physical cores and power status.
    /// On battery: throttles to 50% or physical_cores - 2 to eliminate thermal throttling.
    /// On AC: utilizes maximum physical compute capacity.
    pub fn recommended_threads(&self, max_threads: usize) -> usize {
        let base_threads = if max_threads == 0 { 1 } else { max_threads };
        match self.detect_power_source() {
            PowerSource::Battery => {
                // Throttled: scale down by 40% or cap at max(2, base - 2)
                let throttled = (base_threads as f32 * 0.6).round() as usize;
                throttled.clamp(1, base_threads)
            }
            PowerSource::AcPower | PowerSource::Unknown => base_threads,
        }
    }
}

/// Plays a subtle terminal audio cue (terminal bell or ASCII beep) upon task completion.
pub fn play_completion_sound() {
    print!("\x07");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

/// Automatically distills a minimal bash reproduction script to ~/.cynapse/repro/<id>.sh
/// when a tool fails twice or encounters a test failure.
pub fn generate_reproduction_script(repro_id: &str, command: &str, expected: &str, actual: &str) -> Result<PathBuf> {
    let repro_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cynapse")
        .join("repro");
    fs::create_dir_all(&repro_dir)?;

    let script_path = repro_dir.join(format!("{}.sh", repro_id));
    let script_content = format!(
        r#"#!/usr/bin/env bash
# Cynapse Automated Failure Reproduction Script
# Repro ID: {}
# Generated automatically to enable 1-step reproduction

set -euo pipefail

echo "=== Running Failing Command ==="
echo "Command: {}"
echo ""

# Expected Output:
# {}

# Actual Failure Recorded:
# {}

{}
"#,
        repro_id,
        command,
        expected.replace('\n', "\n# "),
        actual.replace('\n', "\n# "),
        command
    );

    fs::write(&script_path, script_content)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    Ok(script_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_power_throttler_battery_detection() {
        let temp = tempdir().unwrap();
        let bat_dir = temp.path().join("BAT0");
        fs::create_dir_all(&bat_dir).unwrap();
        fs::write(bat_dir.join("status"), "Discharging\n").unwrap();

        let throttler = PowerThrottler::with_path(temp.path().to_path_buf());
        assert_eq!(throttler.detect_power_source(), PowerSource::Battery);
        assert_eq!(throttler.recommended_threads(8), 5); // 8 * 0.6 = 4.8 -> 5
    }

    #[test]
    fn test_power_throttler_ac_detection() {
        let temp = tempdir().unwrap();
        let ac_dir = temp.path().join("ACAD");
        fs::create_dir_all(&ac_dir).unwrap();
        fs::write(ac_dir.join("online"), "1\n").unwrap();

        let throttler = PowerThrottler::with_path(temp.path().to_path_buf());
        assert_eq!(throttler.detect_power_source(), PowerSource::AcPower);
        assert_eq!(throttler.recommended_threads(8), 8);
    }

    #[test]
    fn test_generate_reproduction_script() {
        let path = generate_reproduction_script("test_repro_1", "cargo test test_fails", "ok", "error: test failed").unwrap();
        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("cargo test test_fails"));
        assert!(content.contains("Actual Failure Recorded:"));
        let _ = fs::remove_file(path);
    }
}
