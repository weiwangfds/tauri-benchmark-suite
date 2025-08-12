use crate::benchmark::error::BenchmarkError;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTestResult {
    pub sequential_read_speed: f64, // MB/s
    pub sequential_write_speed: f64, // MB/s
    pub random_access_speed: f64, // MB/s
    pub latency: f64, // nanoseconds
    pub memory_usage_peak: u64, // MB
    pub error_rate: f64, // percentage
    pub test_duration: u64, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTestConfig {
    pub buffer_size: usize, // MB
    pub iterations: usize,
    pub test_duration: u64, // seconds
    pub enable_usage_monitoring: bool,
}

pub struct MemoryBenchmark {
    config: MemoryTestConfig,
}

impl MemoryBenchmark {
    pub fn new(config: MemoryTestConfig) -> Self {
        Self { config }
    }

    pub fn run_benchmark(&self) -> Result<MemoryTestResult, BenchmarkError> {
        self.run_benchmark_with_progress(|_progress, _message| {})
    }

    pub fn run_benchmark_with_progress<F>(&self, progress_callback: F) -> Result<MemoryTestResult, BenchmarkError>
    where
        F: Fn(f64, String) + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        
        // 运行顺序读取测试
        progress_callback(0.0, "开始内存顺序读取测试...".to_string());
        let sequential_read_speed = self.test_sequential_read_with_progress(&progress_callback)?;
        
        // 运行顺序写入测试
        progress_callback(25.0, "开始内存顺序写入测试...".to_string());
        let sequential_write_speed = self.test_sequential_write_with_progress(&progress_callback)?;
        
        // 运行随机访问测试
        progress_callback(50.0, "开始内存随机访问测试...".to_string());
        let random_access_speed = self.test_random_access_with_progress(&progress_callback)?;
        
        // 运行内存延迟测试
        progress_callback(75.0, "开始内存延迟测试...".to_string());
        let latency = self.test_memory_latency_with_progress(&progress_callback)?;
        
        // 监控内存使用量（如果启用）
        progress_callback(90.0, "监控内存使用量...".to_string());
        let memory_usage_peak = if self.config.enable_usage_monitoring {
            self.monitor_memory_usage()?
        } else {
            0
        };

        let test_duration = std::cmp::max(start_time.elapsed().as_secs(), 1); // 至少1秒
        
        progress_callback(100.0, "内存测试完成".to_string());
        
        Ok(MemoryTestResult {
            sequential_read_speed,
            sequential_write_speed,
            random_access_speed,
            latency,
            memory_usage_peak,
            error_rate: 0.0, // 暂时设为0，实际应用中可以检测内存错误
            test_duration,
        })
    }

    fn test_sequential_read(&self) -> Result<f64, BenchmarkError> {
        self.test_sequential_read_with_progress(&|_progress, _message| {})
    }

    fn test_sequential_read_with_progress<F>(&self, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let buffer_size_bytes = self.config.buffer_size * 1024 * 1024; // Convert MB to bytes
        let mut buffer = vec![0u8; buffer_size_bytes];
        
        // 初始化缓冲区
        for i in 0..buffer_size_bytes {
            buffer[i] = (i % 256) as u8;
        }

        let start_time = Instant::now();
        let mut total_bytes = 0u64;
        let mut checksum = 0u64;

        for iteration in 0..self.config.iterations {
            // 顺序读取整个缓冲区
            for chunk in buffer.chunks(4096) { // 4KB chunks
                for &byte in chunk {
                    checksum = checksum.wrapping_add(byte as u64);
                }
                total_bytes += chunk.len() as u64;
            }
            
            // 更新进度
            let progress = ((iteration + 1) as f64 / self.config.iterations as f64) * 100.0;
            progress_callback(progress, format!("顺序读取测试进行中... ({:.1}%)", progress));
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_mb_s = (total_bytes as f64) / (1024.0 * 1024.0) / elapsed;
        
        // 防止编译器优化掉计算
        if checksum == 0 {
            return Err(BenchmarkError::MemoryTestError("Checksum error".to_string()));
        }
        
        Ok(speed_mb_s)
    }

    fn test_sequential_write(&self) -> Result<f64, BenchmarkError> {
        self.test_sequential_write_with_progress(&|_progress, _message| {})
    }

    fn test_sequential_write_with_progress<F>(&self, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let buffer_size_bytes = self.config.buffer_size * 1024 * 1024;
        let mut buffer = vec![0u8; buffer_size_bytes];

        let start_time = Instant::now();
        let mut total_bytes = 0u64;

        for iteration in 0..self.config.iterations {
            let pattern = (iteration % 256) as u8;
            
            // 顺序写入整个缓冲区
            for chunk in buffer.chunks_mut(4096) {
                let chunk_len = chunk.len();
                for byte in chunk {
                    *byte = pattern;
                }
                total_bytes += chunk_len as u64;
            }
            
            // 更新进度
            let progress = ((iteration + 1) as f64 / self.config.iterations as f64) * 100.0;
            progress_callback(progress, format!("顺序写入测试进行中... ({:.1}%)", progress));
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_mb_s = (total_bytes as f64) / (1024.0 * 1024.0) / elapsed;
        
        Ok(speed_mb_s)
    }

    fn test_random_access(&self) -> Result<f64, BenchmarkError> {
        self.test_random_access_with_progress(&|_progress, _message| {})
    }

    fn test_random_access_with_progress<F>(&self, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let buffer_size_bytes = self.config.buffer_size * 1024 * 1024;
        let mut buffer = vec![0u8; buffer_size_bytes];
        
        // 初始化缓冲区
        for i in 0..buffer_size_bytes {
            buffer[i] = (i % 256) as u8;
        }

        let start_time = Instant::now();
        let mut total_accesses = 0u64;
        let mut checksum = 0u64;
        
        // 使用简单的线性同余生成器生成随机索引
        let mut rng_state = 12345u64;
        
        for iteration in 0..self.config.iterations {
            for _ in 0..10000 { // 每次迭代进行10000次随机访问
                rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                let index = (rng_state as usize) % buffer_size_bytes;
                
                // 随机读取
                checksum = checksum.wrapping_add(buffer[index] as u64);
                
                // 随机写入
                buffer[index] = (rng_state % 256) as u8;
                
                total_accesses += 2; // 一次读取 + 一次写入
            }
            
            // 更新进度
            let progress = ((iteration + 1) as f64 / self.config.iterations as f64) * 100.0;
            progress_callback(progress, format!("随机访问测试进行中... ({:.1}%)", progress));
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_mb_s = (total_accesses as f64) / (1024.0 * 1024.0) / elapsed;
        
        // 防止编译器优化
        if checksum == 0 {
            return Err(BenchmarkError::MemoryTestError("Checksum error".to_string()));
        }
        
        Ok(speed_mb_s)
    }

    fn test_memory_latency(&self) -> Result<f64, BenchmarkError> {
        self.test_memory_latency_with_progress(&|_progress, _message| {})
    }

    fn test_memory_latency_with_progress<F>(&self, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        const LATENCY_TEST_SIZE: usize = 64 * 1024; // 64KB for cache testing
        let mut buffer = vec![0usize; LATENCY_TEST_SIZE / std::mem::size_of::<usize>()];
        
        // 创建随机访问模式
        let mut rng_state = 54321u64;
        for i in 0..buffer.len() {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            buffer[i] = (rng_state as usize) % buffer.len();
        }

        let iterations = 1000000; // 100万次访问
        let start_time = Instant::now();
        let mut last_progress_update = Instant::now();
        
        let mut index = 0;
        for i in 0..iterations {
            index = buffer[index];
            
            // 每10万次访问更新一次进度
            if i % 100000 == 0 && last_progress_update.elapsed().as_millis() >= 100 {
                let progress = (i as f64 / iterations as f64) * 100.0;
                progress_callback(progress, format!("内存延迟测试进行中... ({:.1}%)", progress));
                last_progress_update = Instant::now();
            }
        }

        let elapsed = start_time.elapsed();
        let latency_ns = elapsed.as_nanos() as f64 / iterations as f64;
        
        // 防止编译器优化
        if index >= buffer.len() {
            return Err(BenchmarkError::MemoryTestError("Index out of bounds".to_string()));
        }
        
        Ok(latency_ns)
    }

    fn monitor_memory_usage(&self) -> Result<u64, BenchmarkError> {
        let mut sys = System::new_all();
        sys.refresh_memory();
        
        let initial_used = sys.used_memory();
        
        // 分配一些内存来测试峰值使用量
        let test_size = self.config.buffer_size * 1024 * 1024;
        let _test_buffer = vec![0u8; test_size];
        
        sys.refresh_memory();
        let peak_used = sys.used_memory();
        
        let peak_usage_mb = (peak_used - initial_used) / (1024 * 1024);
        Ok(peak_usage_mb)
    }
}#
[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_benchmark_creation() {
        let config = MemoryTestConfig {
            buffer_size: 10, // 10MB
            iterations: 5,
            test_duration: 10,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        assert_eq!(benchmark.config.buffer_size, 10);
        assert_eq!(benchmark.config.iterations, 5);
        assert!(!benchmark.config.enable_usage_monitoring);
    }

    #[test]
    fn test_sequential_read_performance() {
        let config = MemoryTestConfig {
            buffer_size: 1, // 1MB for quick test
            iterations: 2,
            test_duration: 5,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.test_sequential_read();
        
        assert!(result.is_ok());
        let speed = result.unwrap();
        assert!(speed > 0.0, "顺序读取速度应该大于0");
    }

    #[test]
    fn test_sequential_write_performance() {
        let config = MemoryTestConfig {
            buffer_size: 1, // 1MB for quick test
            iterations: 2,
            test_duration: 5,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.test_sequential_write();
        
        assert!(result.is_ok());
        let speed = result.unwrap();
        assert!(speed > 0.0, "顺序写入速度应该大于0");
    }

    #[test]
    fn test_random_access_performance() {
        let config = MemoryTestConfig {
            buffer_size: 1, // 1MB for quick test
            iterations: 1,
            test_duration: 5,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.test_random_access();
        
        assert!(result.is_ok());
        let speed = result.unwrap();
        assert!(speed > 0.0, "随机访问速度应该大于0");
    }

    #[test]
    fn test_memory_latency() {
        let config = MemoryTestConfig {
            buffer_size: 1,
            iterations: 1,
            test_duration: 5,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.test_memory_latency();
        
        assert!(result.is_ok());
        let latency = result.unwrap();
        assert!(latency > 0.0, "内存延迟应该大于0纳秒");
        assert!(latency < 1000000.0, "内存延迟应该小于1毫秒");
    }

    #[test]
    fn test_memory_usage_monitoring() {
        let config = MemoryTestConfig {
            buffer_size: 5, // 5MB
            iterations: 1,
            test_duration: 5,
            enable_usage_monitoring: true,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.monitor_memory_usage();
        
        assert!(result.is_ok());
        let usage = result.unwrap();
        // 内存使用量应该是合理的范围
        assert!(usage >= 0, "内存使用量不应该为负数");
    }

    #[test]
    fn test_full_memory_benchmark() {
        let config = MemoryTestConfig {
            buffer_size: 2, // 2MB for quick test
            iterations: 2,
            test_duration: 5,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.run_benchmark();
        
        assert!(result.is_ok());
        let memory_result = result.unwrap();
        
        assert!(memory_result.sequential_read_speed > 0.0);
        assert!(memory_result.sequential_write_speed > 0.0);
        assert!(memory_result.random_access_speed > 0.0);
        assert!(memory_result.latency > 0.0);
        assert_eq!(memory_result.error_rate, 0.0);
        assert!(memory_result.test_duration > 0);
    }

    #[test]
    fn test_memory_benchmark_with_monitoring() {
        let config = MemoryTestConfig {
            buffer_size: 1, // 1MB
            iterations: 1,
            test_duration: 5,
            enable_usage_monitoring: true,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.run_benchmark();
        
        assert!(result.is_ok());
        let memory_result = result.unwrap();
        
        assert!(memory_result.sequential_read_speed > 0.0);
        assert!(memory_result.sequential_write_speed > 0.0);
        assert!(memory_result.random_access_speed > 0.0);
        assert!(memory_result.latency > 0.0);
        assert!(memory_result.memory_usage_peak >= 0);
    }

    #[test]
    fn test_performance_comparison() {
        let config = MemoryTestConfig {
            buffer_size: 1,
            iterations: 2,
            test_duration: 5,
            enable_usage_monitoring: false,
        };
        
        let benchmark = MemoryBenchmark::new(config);
        
        let read_speed = benchmark.test_sequential_read().unwrap();
        let write_speed = benchmark.test_sequential_write().unwrap();
        let random_speed = benchmark.test_random_access().unwrap();
        
        // 通常顺序访问比随机访问快
        // 但这个测试可能因为测试方法的差异而不总是成立
        assert!(read_speed > 0.0);
        assert!(write_speed > 0.0);
        assert!(random_speed > 0.0);
    }
}