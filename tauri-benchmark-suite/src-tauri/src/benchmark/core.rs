use crate::benchmark::error::BenchmarkError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub cpu_test: CpuTestConfig,
    pub memory_test: MemoryTestConfig,
    pub storage_test: StorageTestConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTestConfig {
    pub enabled: bool,
    pub duration: u64, // seconds
    pub thread_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTestConfig {
    pub enabled: bool,
    pub buffer_size: usize, // MB
    pub iterations: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageTestConfig {
    pub enabled: bool,
    pub file_size: u64,    // MB
    pub block_size: usize, // KB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub timestamp: String,
    pub system_info: crate::benchmark::system_info::SystemInfo,
    pub cpu_results: Option<crate::benchmark::cpu::CpuTestResult>,
    pub memory_results: Option<crate::benchmark::memory::MemoryTestResult>,
    pub storage_results: Option<crate::benchmark::storage::StorageTestResult>,
    pub overall_score: f64,
}

pub struct BenchmarkCore {
    config: BenchmarkConfig,
    results: Vec<TestResult>,
}

impl BenchmarkCore {
    pub fn new() -> Self {
        Self {
            config: BenchmarkConfig {
                cpu_test: CpuTestConfig {
                    enabled: true,
                    duration: 60,
                    thread_count: 0, // 0 means use all available threads
                },
                memory_test: MemoryTestConfig {
                    enabled: true,
                    buffer_size: 1024, // 1GB
                    iterations: 100,
                },
                storage_test: StorageTestConfig {
                    enabled: true,
                    file_size: 1024, // 1GB
                    block_size: 4,   // 4KB
                },
            },
            results: Vec::new(),
        }
    }

    pub fn set_config(&mut self, config: BenchmarkConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> &BenchmarkConfig {
        &self.config
    }

    pub fn run_cpu_benchmark(
        &mut self,
    ) -> Result<crate::benchmark::cpu::CpuTestResult, BenchmarkError> {
        // TODO: 实现CPU基准测试
        Err(BenchmarkError::CpuTestError(
            "Not implemented yet".to_string(),
        ))
    }

    pub fn run_memory_benchmark(
        &mut self,
    ) -> Result<crate::benchmark::memory::MemoryTestResult, BenchmarkError> {
        // TODO: 实现内存基准测试
        Err(BenchmarkError::MemoryTestError(
            "Not implemented yet".to_string(),
        ))
    }

    pub fn run_storage_benchmark(
        &mut self,
    ) -> Result<crate::benchmark::storage::StorageTestResult, BenchmarkError> {
        // TODO: 实现存储基准测试
        Err(BenchmarkError::StorageTestError(
            "Not implemented yet".to_string(),
        ))
    }

    pub fn get_system_info() -> Result<crate::benchmark::system_info::SystemInfo, BenchmarkError> {
        crate::benchmark::system_info::collect_system_info()
    }
}
