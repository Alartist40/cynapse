//! Single Board Computer (SBC) Edge & Cluster Mesh Architecture.
//!
//! Provides hardware detection, thermal inspection, ZRAM memory validation,
//! and tiered cluster mesh delegation between Orange Pi Zero 3W (edge orchestrator)
//! and Orange Pi 5 (RK3588 6-TOPS NPU compute server).

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbcClusterConfig {
    pub role: String, // "edge_orchestrator" or "compute_host"
    pub peer_endpoint: Option<String>,
    pub rknn_npu_enabled: bool,
    pub zram_enabled: bool,
    pub thermal_throttle_limit_c: f32,
}

impl Default for SbcClusterConfig {
    fn default() -> Self {
        Self {
            role: "edge_orchestrator".to_string(),
            peer_endpoint: None,
            rknn_npu_enabled: true,
            zram_enabled: true,
            thermal_throttle_limit_c: 75.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbcEnvironmentReport {
    pub is_arm: bool,
    pub soc_model: String,
    pub npu_detected: bool,
    pub zram_detected: bool,
    pub cpu_temp_c: Option<f32>,
    pub memory_total_mb: u64,
}

/// Probes host hardware for SBC features (Orange Pi 5 RK3588, Zero 3W, etc.)
pub fn probe_sbc_environment() -> SbcEnvironmentReport {
    let is_arm = std::env::consts::ARCH == "aarch64" || std::env::consts::ARCH == "arm";

    let soc_model = if let Ok(info) = fs::read_to_string("/proc/cpuinfo") {
        if info.contains("rk3588") {
            "Rockchip RK3588 (Orange Pi 5 / 5 Plus)".to_string()
        } else if info.contains("h618") || info.contains("h616") {
            "Allwinner H618 (Orange Pi Zero 3W)".to_string()
        } else if info.contains("bcm2835") || info.contains("bcm2711") || info.contains("bcm2712") {
            "Broadcom BCM (Raspberry Pi)".to_string()
        } else {
            "Standard Host Architecture".to_string()
        }
    } else {
        "Unknown".to_string()
    };

    let npu_detected = Path::new("/dev/rknpu").exists()
        || Path::new("/dev/rknpu0").exists()
        || Path::new("/sys/class/rknpu").exists();

    let zram_detected = if let Ok(swaps) = fs::read_to_string("/proc/swaps") {
        swaps.contains("zram")
    } else {
        false
    };

    let cpu_temp_c = read_cpu_thermal_c();

    let memory_total_mb = if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
        meminfo.lines()
            .find(|l| l.starts_with("MemTotal:"))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|kb| kb.parse::<u64>().ok())
            .map(|kb| kb / 1024)
            .unwrap_or(0)
    } else {
        0
    };

    SbcEnvironmentReport {
        is_arm,
        soc_model,
        npu_detected,
        zram_detected,
        cpu_temp_c,
        memory_total_mb,
    }
}

/// Reads Linux thermal zone temp in degrees Celsius
fn read_cpu_thermal_c() -> Option<f32> {
    if let Ok(entries) = fs::read_dir("/sys/class/thermal") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.file_name().map_or(false, |f| f.to_string_lossy().starts_with("thermal_zone")) {
                let temp_file = p.join("temp");
                if let Ok(content) = fs::read_to_string(temp_file) {
                    if let Ok(milli) = content.trim().parse::<f32>() {
                        return Some(milli / 1000.0);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbc_cluster_config_default() {
        let config = SbcClusterConfig::default();
        assert_eq!(config.role, "edge_orchestrator");
        assert!(config.rknn_npu_enabled);
        assert!(config.zram_enabled);
    }

    #[test]
    fn test_probe_sbc_environment_safe() {
        let report = probe_sbc_environment();
        // Should execute cleanly on any Linux host without panicking
        assert!(!report.soc_model.is_empty());
    }
}
