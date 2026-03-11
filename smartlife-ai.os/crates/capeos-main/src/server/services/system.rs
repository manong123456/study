use serde::Serialize;
use sysinfo::System;

#[derive(Serialize)]
pub struct SystemUtilization {
    pub cpu_percent: f32,
    pub memory_total: u64,
    pub memory_used: u64,
    pub memory_percent: f64,
}

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

#[derive(Serialize)]
pub struct HardwareInfo {
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub total_memory: u64,
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
}

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

#[derive(Serialize)]
pub struct HealthServices {
    pub running: Vec<String>,
    pub not_running: Vec<String>,
}
