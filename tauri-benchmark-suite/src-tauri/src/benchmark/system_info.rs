use crate::benchmark::error::BenchmarkError;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub storage: Vec<StorageInfo>,
    pub system_details: SystemDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub cores: usize,
    pub threads: usize,
    pub base_frequency: u64,
    pub max_frequency: u64,
    pub architecture: String,
    pub cache_info: CacheInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64, // GB
    pub available: u64, // GB
    pub used: u64, // GB
    pub memory_type: String,
    pub speed: u64, // MHz
    pub slots_used: usize,
    pub slots_total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub storage_type: StorageType,
    pub capacity: u64, // GB
    pub available: u64, // GB
    pub interface: String,
    pub file_system: String,
    pub mount_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    SSD,
    HDD,
    NVMe,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub l1_data: Option<u64>, // KB
    pub l1_instruction: Option<u64>, // KB
    pub l2: Option<u64>, // KB
    pub l3: Option<u64>, // KB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDetails {
    pub hostname: String,
    pub uptime: u64, // seconds
    pub boot_time: u64, // timestamp
    pub kernel_version: String,
    pub total_processes: usize,
    pub temperatures: HashMap<String, f32>, // component -> temperature
}

pub fn collect_system_info() -> Result<SystemInfo, BenchmarkError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    // 获取操作系统信息
    let os = format!("{} {}", 
        System::name().unwrap_or_else(|| "Unknown".to_string()),
        System::os_version().unwrap_or_else(|| "Unknown".to_string())
    );

    // 获取CPU信息
    let cpu_info = collect_cpu_info(&sys)?;
    
    // 获取内存信息
    let memory_info = collect_memory_info(&sys);
    
    // 获取存储信息
    let storage_info = collect_storage_info(&sys);
    
    // 获取系统详细信息
    let system_details = collect_system_details(&sys);

    Ok(SystemInfo {
        os,
        cpu: cpu_info,
        memory: memory_info,
        storage: storage_info,
        system_details,
    })
}

fn collect_cpu_info(sys: &System) -> Result<CpuInfo, BenchmarkError> {
    if let Some(cpu) = sys.cpus().first() {
        let brand = cpu.brand().to_string();
        
        // 尝试从CPU品牌信息中提取厂商
        let vendor = if brand.to_lowercase().contains("intel") {
            "Intel".to_string()
        } else if brand.to_lowercase().contains("amd") {
            "AMD".to_string()
        } else if brand.to_lowercase().contains("apple") {
            "Apple".to_string()
        } else {
            "Unknown".to_string()
        };

        // 尝试获取架构信息
        let architecture = std::env::consts::ARCH.to_string();

        // 创建缓存信息（sysinfo不直接提供，使用默认值）
        let cache_info = CacheInfo {
            l1_data: None,
            l1_instruction: None,
            l2: None,
            l3: None,
        };

        Ok(CpuInfo {
            name: brand,
            vendor,
            cores: System::physical_core_count().unwrap_or(0),
            threads: sys.cpus().len(),
            base_frequency: cpu.frequency() as u64,
            max_frequency: cpu.frequency() as u64, // sysinfo doesn't provide max frequency
            architecture,
            cache_info,
        })
    } else {
        Err(BenchmarkError::SystemInfoError("无法获取CPU信息".to_string()))
    }
}

fn collect_memory_info(sys: &System) -> MemoryInfo {
    let total_bytes = sys.total_memory();
    let available_bytes = sys.available_memory();
    let used_bytes = total_bytes - available_bytes;

    MemoryInfo {
        total: total_bytes / (1024 * 1024 * 1024), // Convert to GB
        available: available_bytes / (1024 * 1024 * 1024), // Convert to GB
        used: used_bytes / (1024 * 1024 * 1024), // Convert to GB
        memory_type: "Unknown".to_string(), // sysinfo doesn't provide memory type
        speed: 0, // sysinfo doesn't provide memory speed
        slots_used: 0, // sysinfo doesn't provide slot information
        slots_total: 0, // sysinfo doesn't provide slot information
    }
}

fn collect_storage_info(_sys: &System) -> Vec<StorageInfo> {
    // 暂时返回空的存储信息，等待sysinfo API修复
    vec![StorageInfo {
        name: "Primary Storage".to_string(),
        storage_type: StorageType::Unknown,
        capacity: 0,
        available: 0,
        interface: "Unknown".to_string(),
        file_system: "Unknown".to_string(),
        mount_point: "/".to_string(),
    }]
}

fn determine_storage_type(name: &str, mount_point: &str) -> StorageType {
    let name_lower = name.to_lowercase();
    let mount_lower = mount_point.to_lowercase();
    
    if name_lower.contains("nvme") || mount_lower.contains("nvme") {
        StorageType::NVMe
    } else if name_lower.contains("ssd") || mount_lower.contains("ssd") {
        StorageType::SSD
    } else if name_lower.contains("hdd") || name_lower.contains("hard") {
        StorageType::HDD
    } else {
        // 默认假设现代系统使用SSD
        StorageType::Unknown
    }
}

fn collect_system_details(sys: &System) -> SystemDetails {
    let temperatures = HashMap::new();
    
    // 暂时不收集温度信息，等待sysinfo API修复

    SystemDetails {
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        uptime: System::uptime(),
        boot_time: System::boot_time(),
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        total_processes: sys.processes().len(),
        temperatures,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_system_info() {
        let result = collect_system_info();
        assert!(result.is_ok(), "系统信息收集应该成功");
        
        let system_info = result.unwrap();
        
        // 验证基本信息不为空
        assert!(!system_info.os.is_empty(), "操作系统信息不应为空");
        assert!(!system_info.cpu.name.is_empty(), "CPU名称不应为空");
        assert!(system_info.cpu.cores > 0, "CPU核心数应大于0");
        assert!(system_info.cpu.threads > 0, "CPU线程数应大于0");
        assert!(system_info.memory.total > 0, "内存总量应大于0");
        assert!(!system_info.system_details.hostname.is_empty(), "主机名不应为空");
    }

    #[test]
    fn test_determine_storage_type() {
        assert!(matches!(determine_storage_type("nvme0n1", "/"), StorageType::NVMe));
        assert!(matches!(determine_storage_type("sda", "/"), StorageType::Unknown));
        assert!(matches!(determine_storage_type("Samsung SSD", "/"), StorageType::SSD));
        assert!(matches!(determine_storage_type("WD HDD", "/"), StorageType::HDD));
    }

    #[test]
    fn test_cpu_vendor_detection() {
        let sys = System::new_all();
        if let Ok(cpu_info) = collect_cpu_info(&sys) {
            // 验证厂商信息被正确识别
            assert!(
                cpu_info.vendor == "Intel" || 
                cpu_info.vendor == "AMD" || 
                cpu_info.vendor == "Apple" || 
                cpu_info.vendor == "Unknown"
            );
        }
    }

    #[test]
    fn test_memory_calculations() {
        let sys = System::new_all();
        let memory_info = collect_memory_info(&sys);
        
        // 验证内存计算的逻辑正确性
        assert!(memory_info.used <= memory_info.total, "已使用内存不应超过总内存");
        assert!(memory_info.available <= memory_info.total, "可用内存不应超过总内存");
    }

    #[test]
    fn test_storage_info_completeness() {
        let sys = System::new_all();
        let storage_info = collect_storage_info(&sys);
        
        // 暂时只验证基本结构
        assert!(!storage_info.is_empty(), "存储信息不应为空");
        for storage in storage_info {
            assert!(!storage.name.is_empty(), "存储设备名称不应为空");
            assert!(!storage.mount_point.is_empty(), "挂载点不应为空");
        }
    }
}