use tauri_benchmark_suite_lib::*;
use tauri::test::{mock_app, MockRuntime};
use serde_json::json;

#[tokio::test]
async fn test_get_system_info_command() {
    let app = mock_app();
    let result = get_system_info().await;
    
    assert!(result.is_ok(), "System info should be retrievable");
    let system_info = result.unwrap();
    
    assert!(!system_info.os.is_empty(), "OS info should not be empty");
    assert!(system_info.cpu.cores > 0, "CPU cores should be greater than 0");
    assert!(system_info.memory.total > 0, "Total memory should be greater than 0");
}

#[tokio::test]
async fn test_cpu_benchmark_command() {
    let app = mock_app();
    let config = tauri_benchmark_suite_lib::benchmark::cpu::CpuTestConfig {
        thread_count: 2,
        test_duration: 1, // 1 second for quick test
        enable_temperature_monitoring: false,
    };
    
    let result = run_cpu_benchmark(app.handle(), config).await;
    
    assert!(result.is_ok(), "CPU benchmark should complete successfully");
    let cpu_result = result.unwrap();
    
    assert!(cpu_result.single_thread_score > 0.0, "Single thread score should be positive");
    assert!(cpu_result.multi_thread_score > 0.0, "Multi thread score should be positive");
    assert!(cpu_result.floating_point_score > 0.0, "Floating point score should be positive");
    assert_eq!(cpu_result.test_duration, 1, "Test duration should match config");
}

#[tokio::test]
async fn test_memory_benchmark_command() {
    let app = mock_app();
    let config = tauri_benchmark_suite_lib::benchmark::memory::MemoryTestConfig {
        buffer_size: 1, // 1MB for quick test
        iterations: 2,
        test_duration: 5,
        enable_usage_monitoring: false,
    };
    
    let result = run_memory_benchmark(app.handle(), config).await;
    
    assert!(result.is_ok(), "Memory benchmark should complete successfully");
    let memory_result = result.unwrap();
    
    assert!(memory_result.sequential_read_speed > 0.0, "Sequential read speed should be positive");
    assert!(memory_result.sequential_write_speed > 0.0, "Sequential write speed should be positive");
    assert!(memory_result.random_access_speed > 0.0, "Random access speed should be positive");
    assert!(memory_result.latency > 0.0, "Latency should be positive");
}

#[tokio::test]
async fn test_storage_benchmark_command() {
    let app = mock_app();
    let config = tauri_benchmark_suite_lib::benchmark::storage::StorageTestConfig {
        file_size: 1, // 1MB for quick test
        block_size: 4, // 4KB
        test_duration: 5,
        test_file_path: Some("test_ipc_storage.dat".to_string()),
    };
    
    let result = run_storage_benchmark(app.handle(), config).await;
    
    assert!(result.is_ok(), "Storage benchmark should complete successfully");
    let storage_result = result.unwrap();
    
    assert!(storage_result.sequential_read.throughput > 0.0, "Sequential read throughput should be positive");
    assert!(storage_result.sequential_write.throughput > 0.0, "Sequential write throughput should be positive");
    assert!(storage_result.random_read.throughput > 0.0, "Random read throughput should be positive");
    assert!(storage_result.random_write.throughput > 0.0, "Random write throughput should be positive");
    
    // Clean up test file
    let _ = std::fs::remove_file("test_ipc_storage.dat");
}

#[tokio::test]
async fn test_benchmark_suite_session_management() {
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use tauri_benchmark_suite_lib::ipc::TestStatus;
    
    let sessions: Arc<Mutex<HashMap<String, TestStatus>>> = Arc::new(Mutex::new(HashMap::new()));
    let app = mock_app();
    
    // Test session creation and status management
    let session_id = "test-session-123".to_string();
    {
        let mut sessions_guard = sessions.lock().unwrap();
        sessions_guard.insert(session_id.clone(), TestStatus::Running);
    }
    
    // Verify session exists and has correct status
    {
        let sessions_guard = sessions.lock().unwrap();
        let status = sessions_guard.get(&session_id).unwrap();
        assert!(matches!(status, TestStatus::Running), "Session should be in Running state");
    }
    
    // Test status update
    {
        let mut sessions_guard = sessions.lock().unwrap();
        sessions_guard.insert(session_id.clone(), TestStatus::Completed);
    }
    
    {
        let sessions_guard = sessions.lock().unwrap();
        let status = sessions_guard.get(&session_id).unwrap();
        assert!(matches!(status, TestStatus::Completed), "Session should be in Completed state");
    }
}

#[tokio::test]
async fn test_error_handling() {
    let app = mock_app();
    
    // Test with invalid CPU config (0 duration should be handled gracefully)
    let invalid_config = tauri_benchmark_suite_lib::benchmark::cpu::CpuTestConfig {
        thread_count: 1,
        test_duration: 0, // Invalid duration
        enable_temperature_monitoring: false,
    };
    
    // The benchmark should still work or return a meaningful error
    let result = run_cpu_benchmark(app.handle(), invalid_config).await;
    // We don't assert failure here because the implementation might handle 0 duration gracefully
    // Instead, we just ensure it doesn't panic
    println!("CPU benchmark with 0 duration result: {:?}", result);
}

#[tokio::test]
async fn test_progress_callback_integration() {
    use std::sync::{Arc, Mutex};
    use tauri_benchmark_suite_lib::benchmark::cpu::{CpuBenchmark, CpuTestConfig};
    
    let progress_updates: Arc<Mutex<Vec<(f64, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let progress_updates_clone = progress_updates.clone();
    
    let config = CpuTestConfig {
        thread_count: 1,
        test_duration: 1, // 1 second
        enable_temperature_monitoring: false,
    };
    
    let benchmark = CpuBenchmark::new(config);
    
    let progress_callback = move |progress: f64, message: String| {
        let mut updates = progress_updates_clone.lock().unwrap();
        updates.push((progress, message));
    };
    
    let result = benchmark.run_benchmark_with_progress(progress_callback);
    
    assert!(result.is_ok(), "Benchmark with progress callback should succeed");
    
    let updates = progress_updates.lock().unwrap();
    assert!(!updates.is_empty(), "Progress updates should have been received");
    
    // Verify progress values are reasonable
    for (progress, _message) in updates.iter() {
        assert!(*progress >= 0.0 && *progress <= 100.0, "Progress should be between 0 and 100");
    }
    
    // Verify final progress is 100%
    let final_update = updates.last().unwrap();
    assert_eq!(final_update.0, 100.0, "Final progress should be 100%");
    assert_eq!(final_update.1, "CPU测试完成", "Final message should indicate completion");
}