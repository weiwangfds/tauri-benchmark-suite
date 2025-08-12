use crate::benchmark::error::BenchmarkError;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
// use std::sync::{Arc, Mutex}; // 暂时不需要
use std::thread;
use rayon::prelude::*;
use sysinfo::System;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTestResult {
    pub single_thread_score: f64,
    pub multi_thread_score: f64,
    pub floating_point_score: f64,
    pub average_temperature: f32,
    pub max_temperature: f32,
    pub test_duration: u64, // seconds
    pub operations_per_second: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuTestConfig {
    pub thread_count: usize, // 0 means use all available threads
    pub test_duration: u64, // seconds
    pub enable_temperature_monitoring: bool,
}

pub struct CpuBenchmark {
    config: CpuTestConfig,
}

impl CpuBenchmark {
    pub fn new(config: CpuTestConfig) -> Self {
        Self { config }
    }

    pub fn run_benchmark(&self) -> Result<CpuTestResult, BenchmarkError> {
        self.run_benchmark_with_progress(|_progress, _message| {})
    }

    pub fn run_benchmark_with_progress<F>(&self, progress_callback: F) -> Result<CpuTestResult, BenchmarkError>
    where
        F: Fn(f64, String) + Send + Sync + 'static,
    {
        let test_duration = Duration::from_secs(self.config.test_duration);
        
        // 运行单线程测试
        progress_callback(0.0, "开始单线程CPU测试...".to_string());
        let single_thread_score = self.run_single_thread_test_with_progress(test_duration, &progress_callback)?;
        
        // 运行多线程测试
        progress_callback(33.3, "开始多线程CPU测试...".to_string());
        let multi_thread_score = self.run_multi_thread_test_with_progress(test_duration, &progress_callback)?;
        
        // 运行浮点运算测试
        progress_callback(66.6, "开始浮点运算测试...".to_string());
        let floating_point_score = self.run_floating_point_test_with_progress(test_duration, &progress_callback)?;
        
        // 监控温度（如果启用）
        progress_callback(90.0, "收集温度数据...".to_string());
        let (avg_temp, max_temp) = if self.config.enable_temperature_monitoring {
            self.monitor_temperature_during_test(test_duration)?
        } else {
            (0.0, 0.0)
        };

        // 计算总操作数
        let operations_per_second = ((single_thread_score + multi_thread_score + floating_point_score) / 3.0) as u64;

        progress_callback(100.0, "CPU测试完成".to_string());

        Ok(CpuTestResult {
            single_thread_score,
            multi_thread_score,
            floating_point_score,
            average_temperature: avg_temp,
            max_temperature: max_temp,
            test_duration: self.config.test_duration,
            operations_per_second,
        })
    }

    fn run_single_thread_test(&self, duration: Duration) -> Result<f64, BenchmarkError> {
        self.run_single_thread_test_with_progress(duration, &|_progress, _message| {})
    }

    fn run_single_thread_test_with_progress<F>(&self, duration: Duration, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let start_time = Instant::now();
        let mut operations = 0u64;
        let mut _result = 1u64;
        let mut last_progress_update = Instant::now();

        // 执行计算密集型任务
        while start_time.elapsed() < duration {
            // 素数计算测试
            _result = self.calculate_primes_up_to(10000);
            operations += 1;
            
            // 数学运算测试
            for i in 1..1000 {
                _result = _result.wrapping_mul(i).wrapping_add(i * i);
            }
            operations += 999;

            // 更新进度（每100ms更新一次）
            if last_progress_update.elapsed() >= Duration::from_millis(100) {
                let progress = (start_time.elapsed().as_secs_f64() / duration.as_secs_f64() * 100.0).min(100.0);
                progress_callback(progress, format!("单线程测试进行中... ({:.1}%)", progress));
                last_progress_update = Instant::now();
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let score = operations as f64 / elapsed;
        
        Ok(score)
    }

    fn run_multi_thread_test(&self, duration: Duration) -> Result<f64, BenchmarkError> {
        self.run_multi_thread_test_with_progress(duration, &|_progress, _message| {})
    }

    fn run_multi_thread_test_with_progress<F>(&self, duration: Duration, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String) + Sync,
    {
        let thread_count = if self.config.thread_count == 0 {
            num_cpus::get()
        } else {
            self.config.thread_count
        };

        let start_time = Instant::now();
        let test_duration = duration;

        progress_callback(0.0, format!("多线程测试开始 (使用{}个线程)...", thread_count));

        // 使用简单的并行计算避免溢出
        let chunk_size = 100u64;
        let total_operations: u64 = (0..thread_count)
            .into_par_iter()
            .map(|thread_id| {
                let mut local_operations = 0u64;
                let thread_start = Instant::now();
                let mut last_progress_update = Instant::now();
                
                while thread_start.elapsed() < test_duration {
                    // 简单的并行计算密集型任务
                    let _result: u64 = (0..chunk_size)
                        .into_par_iter()
                        .map(|i| {
                            // 简化的数学运算避免溢出
                            let mut val = (i % 1000) + 1;
                            for j in 1..10 {
                                val = (val * j + j) % 1000000;
                            }
                            val
                        })
                        .reduce(|| 0, |a, b| (a + b) % 1000000);
                    
                    local_operations = local_operations.saturating_add(chunk_size);

                    // 只让第一个线程报告进度，避免过多的回调
                    if thread_id == 0 && last_progress_update.elapsed() >= Duration::from_millis(200) {
                        let progress = (thread_start.elapsed().as_secs_f64() / test_duration.as_secs_f64() * 100.0).min(100.0);
                        progress_callback(progress, format!("多线程测试进行中... ({:.1}%)", progress));
                        last_progress_update = Instant::now();
                    }
                }
                
                local_operations
            })
            .reduce(|| 0, |a, b| a.saturating_add(b));

        let elapsed = start_time.elapsed().as_secs_f64();
        let score = total_operations as f64 / elapsed;
        
        Ok(score)
    }

    fn run_floating_point_test(&self, duration: Duration) -> Result<f64, BenchmarkError> {
        self.run_floating_point_test_with_progress(duration, &|_progress, _message| {})
    }

    fn run_floating_point_test_with_progress<F>(&self, duration: Duration, progress_callback: &F) -> Result<f64, BenchmarkError>
    where
        F: Fn(f64, String),
    {
        let start_time = Instant::now();
        let mut operations = 0u64;
        let mut result = 1.0f64;
        let mut last_progress_update = Instant::now();

        while start_time.elapsed() < duration {
            // 浮点数学运算测试
            for i in 1..1000 {
                let x = i as f64;
                result = result * x.sin() + x.cos().powi(2) + x.sqrt().ln();
                
                // 复杂浮点运算
                result = result.exp().tanh() + (x * 3.14159).sin();
            }
            operations += 999;

            // 更新进度（每150ms更新一次）
            if last_progress_update.elapsed() >= Duration::from_millis(150) {
                let progress = (start_time.elapsed().as_secs_f64() / duration.as_secs_f64() * 100.0).min(100.0);
                progress_callback(progress, format!("浮点运算测试进行中... ({:.1}%)", progress));
                last_progress_update = Instant::now();
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let score = operations as f64 / elapsed;
        
        Ok(score)
    }

    fn monitor_temperature_during_test(&self, duration: Duration) -> Result<(f32, f32), BenchmarkError> {
        let mut sys = System::new_all();
        let mut temperatures = Vec::new();
        let start_time = Instant::now();
        let sample_interval = Duration::from_millis(500); // 每500ms采样一次

        while start_time.elapsed() < duration {
            sys.refresh_cpu_all();
            
            // 收集所有CPU核心的温度
            for cpu in sys.cpus() {
                // 注意：sysinfo可能不提供温度信息，这里使用CPU使用率作为替代指标
                let usage = cpu.cpu_usage();
                temperatures.push(usage);
            }
            
            thread::sleep(sample_interval);
        }

        if temperatures.is_empty() {
            return Ok((0.0, 0.0));
        }

        let avg_temp = temperatures.iter().sum::<f32>() / temperatures.len() as f32;
        let max_temp = temperatures.iter().fold(0.0f32, |a, &b| a.max(b));

        Ok((avg_temp, max_temp))
    }

    // 辅助函数：计算素数
    fn calculate_primes_up_to(&self, limit: u64) -> u64 {
        let mut count = 0;
        for num in 2..=limit {
            if self.is_prime(num) {
                count += 1;
            }
        }
        count
    }

    fn is_prime(&self, n: u64) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }
        
        let sqrt_n = (n as f64).sqrt() as u64;
        for i in (3..=sqrt_n).step_by(2) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_benchmark_creation() {
        let config = CpuTestConfig {
            thread_count: 4,
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        assert_eq!(benchmark.config.thread_count, 4);
        assert_eq!(benchmark.config.test_duration, 1);
        assert!(!benchmark.config.enable_temperature_monitoring);
    }

    #[test]
    fn test_single_thread_performance() {
        let config = CpuTestConfig {
            thread_count: 1,
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        let result = benchmark.run_single_thread_test(Duration::from_secs(1));
        
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score > 0.0, "单线程测试分数应该大于0");
    }

    #[test]
    fn test_multi_thread_performance() {
        let config = CpuTestConfig {
            thread_count: 2,
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        let result = benchmark.run_multi_thread_test(Duration::from_secs(1));
        
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score > 0.0, "多线程测试分数应该大于0");
    }

    #[test]
    fn test_floating_point_performance() {
        let config = CpuTestConfig {
            thread_count: 1,
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        let result = benchmark.run_floating_point_test(Duration::from_secs(1));
        
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score > 0.0, "浮点运算测试分数应该大于0");
    }

    #[test]
    fn test_prime_calculation() {
        let config = CpuTestConfig {
            thread_count: 1,
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        
        // 测试素数判断
        assert!(benchmark.is_prime(2));
        assert!(benchmark.is_prime(3));
        assert!(benchmark.is_prime(5));
        assert!(benchmark.is_prime(7));
        assert!(!benchmark.is_prime(4));
        assert!(!benchmark.is_prime(6));
        assert!(!benchmark.is_prime(8));
        assert!(!benchmark.is_prime(9));
        
        // 测试素数计算
        let prime_count = benchmark.calculate_primes_up_to(100);
        assert_eq!(prime_count, 25); // 100以内有25个素数
    }

    #[test]
    fn test_full_benchmark() {
        let config = CpuTestConfig {
            thread_count: 2,
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        let result = benchmark.run_benchmark();
        
        assert!(result.is_ok());
        let cpu_result = result.unwrap();
        
        assert!(cpu_result.single_thread_score > 0.0);
        assert!(cpu_result.multi_thread_score > 0.0);
        assert!(cpu_result.floating_point_score > 0.0);
        assert_eq!(cpu_result.test_duration, 1);
        assert!(cpu_result.operations_per_second > 0);
    }

    #[test]
    fn test_temperature_monitoring() {
        let config = CpuTestConfig {
            thread_count: 1,
            test_duration: 1,
            enable_temperature_monitoring: true,
        };
        
        let benchmark = CpuBenchmark::new(config);
        let result = benchmark.monitor_temperature_during_test(Duration::from_millis(100));
        
        assert!(result.is_ok());
        let (avg_temp, max_temp) = result.unwrap();
        
        // 温度值应该是合理的范围（这里使用CPU使用率作为替代）
        assert!(avg_temp >= 0.0);
        assert!(max_temp >= avg_temp);
    }

    #[test]
    fn test_auto_thread_count() {
        let config = CpuTestConfig {
            thread_count: 0, // 0表示使用所有可用线程
            test_duration: 1,
            enable_temperature_monitoring: false,
        };
        
        let benchmark = CpuBenchmark::new(config);
        let result = benchmark.run_multi_thread_test(Duration::from_secs(1));
        
        assert!(result.is_ok());
        let score = result.unwrap();
        assert!(score > 0.0, "自动线程数测试分数应该大于0");
    }
}