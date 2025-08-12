use crate::benchmark::error::BenchmarkError;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageTestResult {
    pub sequential_read: StorageMetrics,
    pub sequential_write: StorageMetrics,
    pub random_read: StorageMetrics,
    pub random_write: StorageMetrics,
    pub test_duration: u64,        // seconds
    pub total_data_processed: u64, // MB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageTestConfig {
    pub file_size: u64,                 // MB
    pub block_size: usize,              // KB
    pub test_duration: u64,             // seconds
    pub test_file_path: Option<String>, // 可选的测试文件路径
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    pub throughput: f64, // MB/s
    pub iops: u64,
    pub latency: f64, // milliseconds
}

pub struct StorageBenchmark {
    config: StorageTestConfig,
}

impl StorageBenchmark {
    pub fn new(config: StorageTestConfig) -> Self {
        Self { config }
    }

    pub fn run_benchmark(&self) -> Result<StorageTestResult, BenchmarkError> {
        self.run_benchmark_with_progress(|_progress, _message| {})
    }

    pub fn run_benchmark_with_progress<F>(&self, progress_callback: F) -> Result<StorageTestResult, BenchmarkError>
    where
        F: Fn(f64, String) + Send + Sync + 'static,
    {
        let start_time = Instant::now();
        
        // 运行顺序写入测试
        progress_callback(0.0, "开始存储顺序写入测试...".to_string());
        let sequential_write = self.test_sequential_write_with_progress(&progress_callback)?;
        
        // 运行顺序读取测试
        progress_callback(25.0, "开始存储顺序读取测试...".to_string());
        let sequential_read = self.test_sequential_read_with_progress(&progress_callback)?;
        
        // 运行随机写入测试
        progress_callback(50.0, "开始存储随机写入测试...".to_string());
        let random_write = self.test_random_write_with_progress(&progress_callback)?;
        
        // 运行随机读取测试
        progress_callback(75.0, "开始存储随机读取测试...".to_string());
        let random_read = self.test_random_read_with_progress(&progress_callback)?;

        let test_duration = std::cmp::max(start_time.elapsed().as_secs(), 1); // 至少1秒
        let total_data_processed = self.config.file_size * 4; // 4个测试，每个处理file_size的数据
        
        progress_callback(100.0, "存储测试完成".to_string());
        
        Ok(StorageTestResult {
            sequential_read,
            sequential_write,
            random_read,
            random_write,
            test_duration,
            total_data_processed,
        })
    }

    fn get_test_file_path(&self) -> PathBuf {
        if let Some(ref path) = self.config.test_file_path {
            PathBuf::from(path)
        } else {
            // 使用临时目录
            let mut temp_dir = env::temp_dir();
            temp_dir.push("tauri_benchmark_test.dat");
            temp_dir
        }
    }

    fn test_sequential_write(&self) -> Result<StorageMetrics, BenchmarkError> {
        self.test_sequential_write_with_progress(&|_progress, _message| {})
    }

    fn test_sequential_write_with_progress<F>(&self, progress_callback: &F) -> Result<StorageMetrics, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let file_path = self.get_test_file_path();
        let file_size_bytes = self.config.file_size * 1024 * 1024; // Convert MB to bytes
        let block_size_bytes = self.config.block_size * 1024; // Convert KB to bytes
        
        // 创建测试数据
        let test_data = vec![0xAA; block_size_bytes];
        
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .map_err(|e| BenchmarkError::StorageTestError(format!("无法创建测试文件: {}", e)))?;

        let start_time = Instant::now();
        let mut total_bytes_written = 0u64;
        let mut operations = 0u64;
        let mut latencies = Vec::new();
        let mut last_progress_update = Instant::now();

        while total_bytes_written < file_size_bytes {
            let op_start = Instant::now();
            
            file.write_all(&test_data)
                .map_err(|e| BenchmarkError::StorageTestError(format!("写入失败: {}", e)))?;
            
            let op_latency = op_start.elapsed().as_millis() as f64;
            latencies.push(op_latency);
            
            total_bytes_written += test_data.len() as u64;
            operations += 1;

            // 更新进度（每200ms更新一次）
            if last_progress_update.elapsed().as_millis() >= 200 {
                let progress = (total_bytes_written as f64 / file_size_bytes as f64) * 100.0;
                progress_callback(progress, format!("顺序写入进行中... ({:.1}%)", progress));
                last_progress_update = Instant::now();
            }
        }

        file.sync_all()
            .map_err(|e| BenchmarkError::StorageTestError(format!("同步失败: {}", e)))?;

        let elapsed = start_time.elapsed().as_secs_f64();
        let throughput = (total_bytes_written as f64) / (1024.0 * 1024.0) / elapsed;
        let iops = (operations as f64 / elapsed) as u64;
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;

        Ok(StorageMetrics {
            throughput,
            iops,
            latency: avg_latency,
        })
    }

    fn test_sequential_read(&self) -> Result<StorageMetrics, BenchmarkError> {
        self.test_sequential_read_with_progress(&|_progress, _message| {})
    }

    fn test_sequential_read_with_progress<F>(&self, progress_callback: &F) -> Result<StorageMetrics, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let file_path = self.get_test_file_path();
        let block_size_bytes = self.config.block_size * 1024;
        let file_size_bytes = self.config.file_size * 1024 * 1024;
        
        let mut file = File::open(&file_path)
            .map_err(|e| BenchmarkError::StorageTestError(format!("无法打开测试文件: {}", e)))?;

        let start_time = Instant::now();
        let mut total_bytes_read = 0u64;
        let mut operations = 0u64;
        let mut latencies = Vec::new();
        let mut buffer = vec![0u8; block_size_bytes];
        let mut last_progress_update = Instant::now();

        loop {
            let op_start = Instant::now();
            
            match file.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(bytes_read) => {
                    let op_latency = op_start.elapsed().as_millis() as f64;
                    latencies.push(op_latency);
                    
                    total_bytes_read += bytes_read as u64;
                    operations += 1;

                    // 更新进度（每200ms更新一次）
                    if last_progress_update.elapsed().as_millis() >= 200 {
                        let progress = (total_bytes_read as f64 / file_size_bytes as f64) * 100.0;
                        progress_callback(progress.min(100.0), format!("顺序读取进行中... ({:.1}%)", progress.min(100.0)));
                        last_progress_update = Instant::now();
                    }
                }
                Err(e) => return Err(BenchmarkError::StorageTestError(format!("读取失败: {}", e))),
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let throughput = (total_bytes_read as f64) / (1024.0 * 1024.0) / elapsed;
        let iops = (operations as f64 / elapsed) as u64;
        let avg_latency = if latencies.is_empty() { 0.0 } else { latencies.iter().sum::<f64>() / latencies.len() as f64 };

        Ok(StorageMetrics {
            throughput,
            iops,
            latency: avg_latency,
        })
    }

    fn test_random_write(&self) -> Result<StorageMetrics, BenchmarkError> {
        self.test_random_write_with_progress(&|_progress, _message| {})
    }

    fn test_random_write_with_progress<F>(&self, progress_callback: &F) -> Result<StorageMetrics, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let file_path = self.get_test_file_path();
        let file_size_bytes = self.config.file_size * 1024 * 1024;
        let block_size_bytes = self.config.block_size * 1024;
        
        let test_data = vec![0xBB; block_size_bytes];
        
        let mut file = OpenOptions::new()
            .write(true)
            .open(&file_path)
            .map_err(|e| BenchmarkError::StorageTestError(format!("无法打开测试文件: {}", e)))?;

        let start_time = Instant::now();
        let mut operations = 0u64;
        let mut latencies = Vec::new();
        let max_operations = 1000; // 限制随机操作数量以避免测试时间过长
        
        // 简单的随机数生成器
        let mut rng_state = 12345u64;

        for i in 0..max_operations {
            // 生成随机位置
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let random_pos = (rng_state % (file_size_bytes / block_size_bytes as u64)) * block_size_bytes as u64;
            
            let op_start = Instant::now();
            
            file.seek(SeekFrom::Start(random_pos))
                .map_err(|e| BenchmarkError::StorageTestError(format!("定位失败: {}", e)))?;
            
            file.write_all(&test_data)
                .map_err(|e| BenchmarkError::StorageTestError(format!("随机写入失败: {}", e)))?;
            
            let op_latency = op_start.elapsed().as_millis() as f64;
            latencies.push(op_latency);
            operations += 1;

            // 更新进度（每50次操作更新一次）
            if i % 50 == 0 {
                let progress = (i as f64 / max_operations as f64) * 100.0;
                progress_callback(progress, format!("随机写入进行中... ({:.1}%)", progress));
            }
        }

        file.sync_all()
            .map_err(|e| BenchmarkError::StorageTestError(format!("同步失败: {}", e)))?;

        let elapsed = start_time.elapsed().as_secs_f64();
        let total_bytes = operations * block_size_bytes as u64;
        let throughput = (total_bytes as f64) / (1024.0 * 1024.0) / elapsed;
        let iops = (operations as f64 / elapsed) as u64;
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;

        Ok(StorageMetrics {
            throughput,
            iops,
            latency: avg_latency,
        })
    }

    fn test_random_read(&self) -> Result<StorageMetrics, BenchmarkError> {
        self.test_random_read_with_progress(&|_progress, _message| {})
    }

    fn test_random_read_with_progress<F>(&self, progress_callback: &F) -> Result<StorageMetrics, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let file_path = self.get_test_file_path();
        let file_size_bytes = self.config.file_size * 1024 * 1024;
        let block_size_bytes = self.config.block_size * 1024;
        
        let mut file = File::open(&file_path)
            .map_err(|e| BenchmarkError::StorageTestError(format!("无法打开测试文件: {}", e)))?;

        let start_time = Instant::now();
        let mut operations = 0u64;
        let mut latencies = Vec::new();
        let mut buffer = vec![0u8; block_size_bytes];
        let max_operations = 1000; // 限制随机操作数量
        
        // 简单的随机数生成器
        let mut rng_state = 54321u64;

        for i in 0..max_operations {
            // 生成随机位置
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            let random_pos = (rng_state % (file_size_bytes / block_size_bytes as u64)) * block_size_bytes as u64;
            
            let op_start = Instant::now();
            
            file.seek(SeekFrom::Start(random_pos))
                .map_err(|e| BenchmarkError::StorageTestError(format!("定位失败: {}", e)))?;
            
            match file.read(&mut buffer) {
                Ok(_) => {
                    let op_latency = op_start.elapsed().as_millis() as f64;
                    latencies.push(op_latency);
                    operations += 1;
                }
                Err(e) => return Err(BenchmarkError::StorageTestError(format!("随机读取失败: {}", e))),
            }

            // 更新进度（每50次操作更新一次）
            if i % 50 == 0 {
                let progress = (i as f64 / max_operations as f64) * 100.0;
                progress_callback(progress, format!("随机读取进行中... ({:.1}%)", progress));
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let total_bytes = operations * block_size_bytes as u64;
        let throughput = (total_bytes as f64) / (1024.0 * 1024.0) / elapsed;
        let iops = (operations as f64 / elapsed) as u64;
        let avg_latency = latencies.iter().sum::<f64>() / latencies.len() as f64;

        Ok(StorageMetrics {
            throughput,
            iops,
            latency: avg_latency,
        })
    }
}

impl Drop for StorageBenchmark {
    fn drop(&mut self) {
        // 清理测试文件
        let file_path = self.get_test_file_path();
        if file_path.exists() {
            let _ = std::fs::remove_file(file_path);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_storage_benchmark_creation() {
        let config = StorageTestConfig {
            file_size: 1, // 1MB for quick test
            block_size: 4, // 4KB
            test_duration: 5,
            test_file_path: None,
        };
        
        let benchmark = StorageBenchmark::new(config);
        assert_eq!(benchmark.config.file_size, 1);
        assert_eq!(benchmark.config.block_size, 4);
    }

    #[test]
    fn test_sequential_write_performance() {
        let config = StorageTestConfig {
            file_size: 1, // 1MB for quick test
            block_size: 4, // 4KB
            test_duration: 5,
            test_file_path: Some("test_seq_write.dat".to_string()),
        };
        
        let benchmark = StorageBenchmark::new(config);
        let result = benchmark.test_sequential_write();
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.throughput > 0.0, "顺序写入吞吐量应该大于0");
        assert!(metrics.iops > 0, "IOPS应该大于0");
        assert!(metrics.latency >= 0.0, "延迟应该大于等于0");
        
        // 清理测试文件
        let _ = fs::remove_file("test_seq_write.dat");
    }

    #[test]
    fn test_sequential_read_performance() {
        let config = StorageTestConfig {
            file_size: 1, // 1MB
            block_size: 4, // 4KB
            test_duration: 5,
            test_file_path: Some("test_seq_read.dat".to_string()),
        };
        
        let benchmark = StorageBenchmark::new(config);
        
        // 先写入数据
        let _ = benchmark.test_sequential_write();
        
        // 然后测试读取
        let result = benchmark.test_sequential_read();
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.throughput > 0.0, "顺序读取吞吐量应该大于0");
        assert!(metrics.iops > 0, "IOPS应该大于0");
        assert!(metrics.latency >= 0.0, "延迟应该大于等于0");
        
        // 清理测试文件
        let _ = fs::remove_file("test_seq_read.dat");
    }

    #[test]
    fn test_random_write_performance() {
        let config = StorageTestConfig {
            file_size: 1, // 1MB
            block_size: 4, // 4KB
            test_duration: 5,
            test_file_path: Some("test_rand_write.dat".to_string()),
        };
        
        let benchmark = StorageBenchmark::new(config);
        
        // 先创建文件
        let _ = benchmark.test_sequential_write();
        
        // 然后测试随机写入
        let result = benchmark.test_random_write();
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.throughput > 0.0, "随机写入吞吐量应该大于0");
        assert!(metrics.iops > 0, "IOPS应该大于0");
        assert!(metrics.latency >= 0.0, "延迟应该大于等于0");
        
        // 清理测试文件
        let _ = fs::remove_file("test_rand_write.dat");
    }

    #[test]
    fn test_random_read_performance() {
        let config = StorageTestConfig {
            file_size: 1, // 1MB
            block_size: 4, // 4KB
            test_duration: 5,
            test_file_path: Some("test_rand_read.dat".to_string()),
        };
        
        let benchmark = StorageBenchmark::new(config);
        
        // 先创建文件
        let _ = benchmark.test_sequential_write();
        
        // 然后测试随机读取
        let result = benchmark.test_random_read();
        
        assert!(result.is_ok());
        let metrics = result.unwrap();
        assert!(metrics.throughput > 0.0, "随机读取吞吐量应该大于0");
        assert!(metrics.iops > 0, "IOPS应该大于0");
        assert!(metrics.latency >= 0.0, "延迟应该大于等于0");
        
        // 清理测试文件
        let _ = fs::remove_file("test_rand_read.dat");
    }

    #[test]
    fn test_full_storage_benchmark() {
        let config = StorageTestConfig {
            file_size: 1, // 1MB for quick test
            block_size: 4, // 4KB
            test_duration: 5,
            test_file_path: Some("test_full_benchmark.dat".to_string()),
        };
        
        let benchmark = StorageBenchmark::new(config);
        let result = benchmark.run_benchmark();
        
        assert!(result.is_ok());
        let storage_result = result.unwrap();
        
        assert!(storage_result.sequential_read.throughput > 0.0);
        assert!(storage_result.sequential_write.throughput > 0.0);
        assert!(storage_result.random_read.throughput > 0.0);
        assert!(storage_result.random_write.throughput > 0.0);
        
        assert!(storage_result.sequential_read.iops > 0);
        assert!(storage_result.sequential_write.iops > 0);
        assert!(storage_result.random_read.iops > 0);
        assert!(storage_result.random_write.iops > 0);
        
        assert!(storage_result.test_duration > 0);
        assert!(storage_result.total_data_processed > 0);
        
        // 清理测试文件
        let _ = fs::remove_file("test_full_benchmark.dat");
    }

    #[test]
    fn test_file_path_generation() {
        let config_with_path = StorageTestConfig {
            file_size: 1,
            block_size: 4,
            test_duration: 5,
            test_file_path: Some("custom_test.dat".to_string()),
        };
        
        let benchmark_with_path = StorageBenchmark::new(config_with_path);
        let path_with_custom = benchmark_with_path.get_test_file_path();
        assert_eq!(path_with_custom, PathBuf::from("custom_test.dat"));
        
        let config_without_path = StorageTestConfig {
            file_size: 1,
            block_size: 4,
            test_duration: 5,
            test_file_path: None,
        };
        
        let benchmark_without_path = StorageBenchmark::new(config_without_path);
        let path_without_custom = benchmark_without_path.get_test_file_path();
        assert!(path_without_custom.to_string_lossy().contains("tauri_benchmark_test.dat"));
    }

    #[test]
    fn test_performance_metrics_validity() {
        let config = StorageTestConfig {
            file_size: 1,
            block_size: 4,
            test_duration: 5,
            test_file_path: Some("test_metrics.dat".to_string()),
        };
        
        let benchmark = StorageBenchmark::new(config);
        
        // 测试顺序写入
        let write_result = benchmark.test_sequential_write().unwrap();
        assert!(write_result.throughput > 0.0);
        assert!(write_result.iops > 0);
        assert!(write_result.latency >= 0.0);
        
        // 测试顺序读取
        let read_result = benchmark.test_sequential_read().unwrap();
        assert!(read_result.throughput > 0.0);
        assert!(read_result.iops > 0);
        assert!(read_result.latency >= 0.0);
        
        // 清理测试文件
        let _ = fs::remove_file("test_metrics.dat");
    }
}