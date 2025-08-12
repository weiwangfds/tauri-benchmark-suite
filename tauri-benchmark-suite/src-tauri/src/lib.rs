mod benchmark;
mod ipc;

use benchmark::system_info::{collect_system_info, SystemInfo};
use benchmark::cpu::{CpuBenchmark, CpuTestConfig, CpuTestResult};
use benchmark::memory::{MemoryBenchmark, MemoryTestConfig, MemoryTestResult};
use benchmark::storage::{StorageBenchmark, StorageTestConfig, StorageTestResult};
use benchmark::error::BenchmarkError;
use benchmark::core::{BenchmarkConfig, TestResult};
use benchmark::cpu::CpuTestConfig as CpuConfig;
use benchmark::memory::MemoryTestConfig as MemoryConfig;
use benchmark::storage::StorageTestConfig as StorageConfig;
use ipc::{BenchmarkProgress, TestStatus, ProgressUpdate, TestSession, SystemMonitoringData, RealTimePerformanceData, TestWarningEvent, WarningSeverity};
use tauri::{AppHandle, Emitter};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

// 全局测试状态管理
type TestSessions = Arc<Mutex<HashMap<String, TestStatus>>>;

// Tauri命令：获取系统信息
#[tauri::command]
async fn get_system_info() -> Result<SystemInfo, String> {
    collect_system_info().map_err(|e| e.to_string())
}

// Tauri命令：启动完整基准测试套件
#[tauri::command]
async fn start_benchmark_suite(
    app: AppHandle,
    config: BenchmarkConfig,
    sessions: tauri::State<'_, TestSessions>,
) -> Result<String, String> {
    let session_id = Uuid::new_v4().to_string();
    
    // 初始化测试会话
    {
        let mut sessions_guard = sessions.lock().unwrap();
        sessions_guard.insert(session_id.clone(), TestStatus::Running);
    }
    
    // 在后台线程中运行测试
    let app_clone = app.clone();
    let sessions_clone = sessions.inner().clone();
    let session_id_clone = session_id.clone();
    
    tokio::spawn(async move {
        if let Err(e) = run_full_benchmark_suite(app_clone.clone(), session_id_clone.clone(), config, sessions_clone.clone()).await {
            // 发送错误事件
            let _ = app_clone.emit("benchmark-error", format!("测试失败: {}", e));
            
            // 更新会话状态
            let mut sessions_guard = sessions_clone.lock().unwrap();
            sessions_guard.insert(session_id_clone, TestStatus::Failed);
        }
    });
    
    Ok(session_id)
}

// Tauri命令：取消测试
#[tauri::command]
async fn cancel_benchmark(
    session_id: String,
    sessions: tauri::State<'_, TestSessions>,
) -> Result<(), String> {
    let mut sessions_guard = sessions.lock().unwrap();
    if let Some(status) = sessions_guard.get_mut(&session_id) {
        *status = TestStatus::Cancelled;
        Ok(())
    } else {
        Err("测试会话不存在".to_string())
    }
}

// Tauri命令：获取所有测试会话
#[tauri::command]
async fn get_all_test_sessions(
    sessions: tauri::State<'_, TestSessions>,
) -> Result<Vec<TestSession>, String> {
    let sessions_guard = sessions.lock().unwrap();
    let test_sessions: Vec<TestSession> = sessions_guard
        .iter()
        .map(|(session_id, status)| TestSession {
            session_id: session_id.clone(),
            status: status.clone(),
            start_time: chrono::Utc::now().to_rfc3339(), // 实际应用中应该存储真实的开始时间
            end_time: match status {
                TestStatus::Completed | TestStatus::Failed | TestStatus::Cancelled => {
                    Some(chrono::Utc::now().to_rfc3339())
                }
                _ => None,
            },
            config: None, // 实际应用中应该存储配置信息
        })
        .collect();
    Ok(test_sessions)
}

// Tauri命令：获取系统监控数据
#[tauri::command]
async fn get_system_monitoring_data() -> Result<SystemMonitoringData, String> {
    // 这里应该实现真实的系统监控数据获取
    // 目前返回模拟数据
    Ok(SystemMonitoringData {
        cpu_usage: 45.2,
        memory_usage: 62.8,
        temperature: Some(55.0),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// Tauri命令：清理已完成的测试会话
#[tauri::command]
async fn cleanup_completed_sessions(
    sessions: tauri::State<'_, TestSessions>,
) -> Result<usize, String> {
    let mut sessions_guard = sessions.lock().unwrap();
    let initial_count = sessions_guard.len();
    
    sessions_guard.retain(|_, status| {
        !matches!(status, TestStatus::Completed | TestStatus::Failed | TestStatus::Cancelled)
    });
    
    let cleaned_count = initial_count - sessions_guard.len();
    Ok(cleaned_count)
}

// Tauri命令：暂停测试（如果支持）
#[tauri::command]
async fn pause_benchmark(
    session_id: String,
    sessions: tauri::State<'_, TestSessions>,
) -> Result<(), String> {
    let mut sessions_guard = sessions.lock().unwrap();
    if let Some(status) = sessions_guard.get_mut(&session_id) {
        match status {
            TestStatus::Running => {
                // 注意：实际的暂停功能需要在测试执行逻辑中实现
                // 这里只是更新状态，实际的暂停需要通过其他机制实现
                Err("暂停功能尚未完全实现".to_string())
            }
            _ => Err("只能暂停正在运行的测试".to_string()),
        }
    } else {
        Err("测试会话不存在".to_string())
    }
}

// Tauri命令：恢复测试（如果支持）
#[tauri::command]
async fn resume_benchmark(
    session_id: String,
    sessions: tauri::State<'_, TestSessions>,
) -> Result<(), String> {
    let sessions_guard = sessions.lock().unwrap();
    if sessions_guard.contains_key(&session_id) {
        // 注意：实际的恢复功能需要在测试执行逻辑中实现
        Err("恢复功能尚未完全实现".to_string())
    } else {
        Err("测试会话不存在".to_string())
    }
}

// Tauri命令：获取测试状态
#[tauri::command]
async fn get_test_status(
    session_id: String,
    sessions: tauri::State<'_, TestSessions>,
) -> Result<TestStatus, String> {
    let sessions_guard = sessions.lock().unwrap();
    sessions_guard.get(&session_id)
        .cloned()
        .ok_or_else(|| "测试会话不存在".to_string())
}

// Tauri命令：运行单个CPU基准测试
#[tauri::command]
async fn run_cpu_benchmark(
    app: AppHandle,
    config: CpuTestConfig,
) -> Result<CpuTestResult, String> {
    let benchmark = CpuBenchmark::new(config);
    
    // 创建进度回调
    let progress_callback = move |progress: f64, message: String| {
        let _ = app.emit("cpu-test-progress", ProgressUpdate {
            progress,
            message,
            test_type: "cpu".to_string(),
        });
    };
    
    benchmark.run_benchmark_with_progress(progress_callback).map_err(|e| e.to_string())
}

// Tauri命令：运行单个内存基准测试
#[tauri::command]
async fn run_memory_benchmark(
    app: AppHandle,
    config: MemoryTestConfig,
) -> Result<MemoryTestResult, String> {
    let benchmark = MemoryBenchmark::new(config);
    
    // 创建进度回调
    let progress_callback = move |progress: f64, message: String| {
        let _ = app.emit("memory-test-progress", ProgressUpdate {
            progress,
            message,
            test_type: "memory".to_string(),
        });
    };
    
    benchmark.run_benchmark_with_progress(progress_callback).map_err(|e| e.to_string())
}

// Tauri命令：运行单个存储基准测试
#[tauri::command]
async fn run_storage_benchmark(
    app: AppHandle,
    config: StorageTestConfig,
) -> Result<StorageTestResult, String> {
    let benchmark = StorageBenchmark::new(config);
    
    // 创建进度回调
    let progress_callback = move |progress: f64, message: String| {
        let _ = app.emit("storage-test-progress", ProgressUpdate {
            progress,
            message,
            test_type: "storage".to_string(),
        });
    };
    
    benchmark.run_benchmark_with_progress(progress_callback).map_err(|e| e.to_string())
}

// 运行完整基准测试套件的内部函数
async fn run_full_benchmark_suite(
    app: AppHandle,
    session_id: String,
    config: BenchmarkConfig,
    sessions: TestSessions,
) -> Result<(), BenchmarkError> {
    use std::sync::Arc;
    
    let overall_progress = Arc::new(std::sync::Mutex::new(0.0f64));
    let total_tests = [config.cpu_test.enabled, config.memory_test.enabled, config.storage_test.enabled]
        .iter()
        .filter(|&&enabled| enabled)
        .count() as f64;
    
    let mut test_result = TestResult {
        timestamp: chrono::Utc::now().to_rfc3339(),
        system_info: collect_system_info()?,
        cpu_results: None,
        memory_results: None,
        storage_results: None,
        overall_score: 0.0,
    };
    
    // 检查是否被取消
    let check_cancelled = || {
        let sessions_guard = sessions.lock().unwrap();
        matches!(sessions_guard.get(&session_id), Some(TestStatus::Cancelled))
    };
    
    // 发送系统监控数据
    let send_monitoring_data = |test_type: &str| {
        let _ = app.emit("system-monitoring", SystemMonitoringData {
            cpu_usage: 45.0, // 实际应用中应该获取真实数据
            memory_usage: 60.0,
            temperature: Some(55.0),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    };
    
    // 运行CPU测试
    if config.cpu_test.enabled && !check_cancelled() {
        send_monitoring_data("cpu");
        
        let _ = app.emit("benchmark-progress", BenchmarkProgress {
            session_id: session_id.clone(),
            current_test: "CPU基准测试".to_string(),
            // 锁定互斥锁以获取当前进度值
            overall_progress: *overall_progress.lock().unwrap(),
            test_progress: 0.0,
            message: "开始CPU性能测试...".to_string(),
            estimated_time_remaining: Some(config.cpu_test.duration),
        });
        
        let cpu_config = CpuConfig {
            thread_count: config.cpu_test.thread_count,
            test_duration: config.cpu_test.duration,
            enable_temperature_monitoring: true,
        };
        let benchmark = CpuBenchmark::new(cpu_config);
        // 克隆需要在闭包中使用的变量
        let app_clone = app.clone();
        let session_id_clone = session_id.clone();
        let overall_progress_clone = overall_progress.clone();
        let progress_callback = move |progress: f64, message: String| {
            // 发送实时性能数据
            let mut metrics = std::collections::HashMap::new();
            metrics.insert("progress".to_string(), progress);
            metrics.insert("cpu_usage".to_string(), 75.0); // 模拟数据
            
            let _ = app_clone.emit("real-time-performance", RealTimePerformanceData {
                session_id: session_id_clone.clone(),
                test_type: "cpu".to_string(),
                metrics,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
            
            let _ = app_clone.emit("benchmark-progress", BenchmarkProgress {
                session_id: session_id_clone.clone(),
                current_test: "CPU基准测试".to_string(),
                // 锁定互斥锁并添加进度值
                overall_progress: *overall_progress_clone.lock().unwrap() + (progress / total_tests),
                test_progress: progress,
                message,
                estimated_time_remaining: None,
            });
        };
        
        match benchmark.run_benchmark_with_progress(progress_callback) {
            Ok(result) => {
                test_result.cpu_results = Some(result);
                // 锁定互斥锁以安全修改共享变量
                *overall_progress.lock().unwrap() += 1.0 / total_tests;
            }
            Err(e) => {
                let _ = app.emit("test-error", ipc::TestCompleteEvent {
                    session_id: session_id.clone(),
                    test_type: "cpu".to_string(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                });
                
                // 发送警告事件
                let _ = app.emit("test-warning", TestWarningEvent {
                    session_id: session_id.clone(),
                    test_type: "cpu".to_string(),
                    warning_type: "test_failure".to_string(),
                    message: format!("CPU测试失败: {}", e),
                    severity: WarningSeverity::High,
                });
            }
        }
    }
    
    // 运行内存测试
    if config.memory_test.enabled && !check_cancelled() {
        let _ = app.emit("benchmark-progress", BenchmarkProgress {
            session_id: session_id.clone(),
            current_test: "内存基准测试".to_string(),
            overall_progress: *overall_progress.lock().unwrap(),
            test_progress: 0.0,
            message: "开始内存性能测试...".to_string(),
            estimated_time_remaining: Some(30), // 估计30秒
        });
        
        let memory_config = MemoryConfig {
            buffer_size: config.memory_test.buffer_size,
            iterations: config.memory_test.iterations,
            test_duration: 30,
            enable_usage_monitoring: true,
        };
        let benchmark = MemoryBenchmark::new(memory_config);
        // 克隆需要在闭包中使用的变量
        let app_clone = app.clone();
        let session_id_clone = session_id.clone();
        let overall_progress_clone = overall_progress.clone();
        let progress_callback = move |progress: f64, message: String| {
            let _ = app_clone.emit("benchmark-progress", BenchmarkProgress {
                session_id: session_id_clone.clone(),
                current_test: "内存基准测试".to_string(),
                overall_progress: *overall_progress_clone.lock().unwrap() + (progress / total_tests),
                test_progress: progress,
                message,
                estimated_time_remaining: None,
            });
        };
        
        match benchmark.run_benchmark_with_progress(progress_callback) {
            Ok(result) => {
                test_result.memory_results = Some(result);
                // 锁定互斥锁以安全修改共享进度变量
                *overall_progress.lock().unwrap() += 1.0 / total_tests;
            }
            Err(e) => {
                let _ = app.emit("test-error", ipc::TestCompleteEvent {
                    session_id: session_id.clone(),
                    test_type: "memory".to_string(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    // 运行存储测试
    if config.storage_test.enabled && !check_cancelled() {
        let _ = app.emit("benchmark-progress", BenchmarkProgress {
            session_id: session_id.clone(),
            current_test: "存储基准测试".to_string(),
            overall_progress: *overall_progress.lock().unwrap(),
            test_progress: 0.0,
            message: "开始存储性能测试...".to_string(),
            estimated_time_remaining: Some(60), // 估计60秒
        });
        
        let storage_config = StorageConfig {
            file_size: config.storage_test.file_size,
            block_size: config.storage_test.block_size,
            test_duration: 60,
            test_file_path: None,
        };
        let benchmark = StorageBenchmark::new(storage_config);
        // 克隆需要在闭包中使用的变量
        let app_clone = app.clone();
        let session_id_clone = session_id.clone();
        let overall_progress_clone = overall_progress.clone();
        let progress_callback = move |progress: f64, message: String| {
            let _ = app_clone.emit("benchmark-progress", BenchmarkProgress {
                session_id: session_id_clone.clone(),
                current_test: "存储基准测试".to_string(),
                overall_progress: *overall_progress_clone.lock().unwrap() + (progress / total_tests),
                test_progress: progress,
                message,
                estimated_time_remaining: None,
            });
        };
        
        match benchmark.run_benchmark_with_progress(progress_callback) {
            Ok(result) => {
                test_result.storage_results = Some(result);
                // 锁定互斥锁以安全修改共享进度变量
                *overall_progress.lock().unwrap() += 1.0 / total_tests;
            }
            Err(e) => {
                let _ = app.emit("test-error", ipc::TestCompleteEvent {
                    session_id: session_id.clone(),
                    test_type: "storage".to_string(),
                    success: false,
                    result: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    // 计算总体评分
    test_result.overall_score = calculate_overall_score(&test_result);
    
    // 发送完成事件
    let _ = app.emit("benchmark-complete", ipc::BenchmarkSuiteCompleteEvent {
        session_id: session_id.clone(),
        success: true,
        results: Some(test_result),
        error: None,
    });
    
    // 更新会话状态
    {
        let mut sessions_guard = sessions.lock().unwrap();
        sessions_guard.insert(session_id, TestStatus::Completed);
    }
    
    Ok(())
}

// 计算总体评分的辅助函数
fn calculate_overall_score(result: &TestResult) -> f64 {
    let mut total_score = 0.0;
    let mut count = 0;
    
    if let Some(cpu_result) = &result.cpu_results {
        total_score += (cpu_result.single_thread_score + cpu_result.multi_thread_score + cpu_result.floating_point_score) / 3.0;
        count += 1;
    }
    
    if let Some(memory_result) = &result.memory_results {
        // 简化的内存评分计算
        total_score += (memory_result.sequential_read_speed + memory_result.sequential_write_speed) / 2.0;
        count += 1;
    }
    
    if let Some(storage_result) = &result.storage_results {
        // 简化的存储评分计算
        total_score += (storage_result.sequential_read.throughput + storage_result.sequential_write.throughput) / 2.0;
        count += 1;
    }
    
    if count > 0 {
        total_score / count as f64
    } else {
        0.0
    }
}

// 保留原有的greet命令用于测试
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let test_sessions: TestSessions = Arc::new(Mutex::new(HashMap::new()));
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(test_sessions)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            start_benchmark_suite,
            cancel_benchmark,
            get_test_status,
            get_all_test_sessions,
            get_system_monitoring_data,
            cleanup_completed_sessions,
            pause_benchmark,
            resume_benchmark,
            run_cpu_benchmark,
            run_memory_benchmark,
            run_storage_benchmark
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use benchmark::core::{CpuTestConfig, MemoryTestConfig, StorageTestConfig};

    #[test]
    fn test_calculate_overall_score() {
        let test_result = TestResult {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            system_info: benchmark::system_info::SystemInfo {
                os: "Test OS".to_string(),
                cpu: benchmark::system_info::CpuInfo {
                    name: "Test CPU".to_string(),
                    cores: 4,
                    threads: 8,
                    base_frequency: 2400,
                    max_frequency: 3600,
                },
                memory: benchmark::system_info::MemoryInfo {
                    total: 16,
                    available: 8,
                    memory_type: "DDR4".to_string(),
                    speed: 3200,
                },
                storage: vec![],
            },
            cpu_results: Some(CpuTestResult {
                single_thread_score: 100.0,
                multi_thread_score: 200.0,
                floating_point_score: 150.0,
                average_temperature: 50.0,
                max_temperature: 60.0,
                test_duration: 60,
                operations_per_second: 1000,
            }),
            memory_results: Some(MemoryTestResult {
                sequential_read_speed: 1000.0,
                sequential_write_speed: 800.0,
                random_access_speed: 500.0,
                latency: 100.0,
                memory_usage_peak: 1024,
                error_rate: 0.0,
                test_duration: 30,
            }),
            storage_results: None,
            overall_score: 0.0,
        };

        let score = calculate_overall_score(&test_result);
        assert!(score > 0.0, "Overall score should be greater than 0");
        
        // 预期分数应该是CPU和内存测试的平均值
        let expected_cpu_score = (100.0 + 200.0 + 150.0) / 3.0;
        let expected_memory_score = (1000.0 + 800.0) / 2.0;
        let expected_overall = (expected_cpu_score + expected_memory_score) / 2.0;
        
        assert!((score - expected_overall).abs() < 0.1, "Score calculation should be accurate");
    }

    #[test]
    fn test_ipc_error_conversion() {
        let benchmark_error = BenchmarkError::CpuTestError("Test error".to_string());
        let ipc_error: ipc::IpcError = benchmark_error.into();
        
        assert_eq!(ipc_error.code, "CPU_TEST_ERROR");
        assert_eq!(ipc_error.message, "CPU测试失败");
        assert_eq!(ipc_error.details, Some("Test error".to_string()));
    }
}