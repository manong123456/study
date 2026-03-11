//! System information and health services.
//!
//! Provides CPU/memory utilization, hardware info, and service health status.

use serde::Serialize;
use sysinfo::System;

/// CPU and memory utilization metrics.
#[derive(Serialize)]
pub struct SystemUtilization {
    /// Global CPU usage percentage (0-100).
    pub cpu_percent: f32,
    /// Total physical memory in bytes.
    pub memory_total: u64,
    /// Used memory in bytes.
    pub memory_used: u64,
    /// Memory usage percentage (0-100).
    pub memory_percent: f64,
}

/// Computes current system utilization from the given `System` instance.
///
/// # Arguments
///
/// * `sys` - A sysinfo `System` (should be refreshed for accurate data).
///
/// # Returns
///
/// `SystemUtilization` with CPU and memory metrics.
pub fn get_utilization(sys: &System) -> SystemUtilization {
    let mem_total = sys.total_memory();
    let mem_used = sys.used_memory();
    let mem_pct = if mem_total > 0 { (mem_used as f64 / mem_total as f64) * 100.0 } else { 0.0 };

    SystemUtilization {
        cpu_percent: sys.global_cpu_usage(),
        memory_total: mem_total,
        memory_used: mem_used,
        memory_percent: mem_pct,
    }
}

/// Hardware and OS information.
#[derive(Serialize)]
pub struct HardwareInfo {
    /// Number of CPU cores.
    pub cpu_count: usize,
    /// CPU brand string (e.g. "Intel Core i7").
    pub cpu_brand: String,
    /// Total physical memory in bytes.
    pub total_memory: u64,
    /// System hostname.
    pub hostname: String,
    /// OS name (e.g. "Linux").
    pub os_name: String,
    /// OS version string.
    pub os_version: String,
    /// Kernel version string.
    pub kernel_version: String,
}

/// Returns hardware and OS information for the current system.
pub fn get_hardware_info() -> HardwareInfo {
    let sys = System::new_all();
    HardwareInfo {
        cpu_count: sys.cpus().len(),
        cpu_brand: sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default(),
        total_memory: sys.total_memory(),
        hostname: System::host_name().unwrap_or_default(),
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        kernel_version: System::kernel_version().unwrap_or_default(),
    }
}

/// Status of CapeOS services (running vs not running).
#[derive(Serialize)]
pub struct HealthServices {
    /// List of service names that are active.
    pub running: Vec<String>,
    /// List of service names that are not active.
    pub not_running: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_utilization() {
        let mut sys = System::new_all();
        sys.refresh_all();
        let util = get_utilization(&sys);
        assert!(util.cpu_percent >= 0.0);
        assert!(util.memory_total > 0);
    }

    #[test]
    fn test_get_hardware_info() {
        let info = get_hardware_info();
        assert!(!info.hostname.is_empty());
        assert!(info.cpu_count > 0);
        assert!(info.total_memory > 0);
    }
}
