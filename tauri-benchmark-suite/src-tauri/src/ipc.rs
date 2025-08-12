use serde::{Deserialize, Serialize};

/// 基准测试进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkProgress {
    pub session_id: String,
    pub current_test: String,
    pub overall_progress: f64,
    pub test_progress: f64,
    pub message: String,
    pub estimated_time_remaining: Option<u64>, // seconds
}

/// 系统资源监控信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMonitoringData {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub temperature: Option<f64>,
    pub timestamp: String,
}

/// 测试状态枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// 进度更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub progress: f64,
    pub message: String,
    pub test_type: String,
}

/// 测试完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCompleteEvent {
    pub session_id: String,
    pub test_type: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// 基准测试套件完成事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuiteCompleteEvent {
    pub session_id: String,
    pub success: bool,
    pub results: Option<crate::benchmark::core::TestResult>,
    pub error: Option<String>,
}

/// 测试会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSession {
    pub session_id: String,
    pub status: TestStatus,
    pub start_time: String,
    pub end_time: Option<String>,
    pub config: Option<crate::benchmark::core::BenchmarkConfig>,
}

/// 实时性能数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimePerformanceData {
    pub session_id: String,
    pub test_type: String,
    pub metrics: std::collections::HashMap<String, f64>,
    pub timestamp: String,
}

/// 测试警告事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWarningEvent {
    pub session_id: String,
    pub test_type: String,
    pub warning_type: String,
    pub message: String,
    pub severity: WarningSeverity,
}

/// 警告严重程度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// IPC错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl From<crate::benchmark::error::BenchmarkError> for IpcError {
    fn from(error: crate::benchmark::error::BenchmarkError) -> Self {
        match error {
            crate::benchmark::error::BenchmarkError::SystemInfoError(msg) => IpcError {
                code: "SYSTEM_INFO_ERROR".to_string(),
                message: "系统信息获取失败".to_string(),
                details: Some(msg),
            },
            crate::benchmark::error::BenchmarkError::CpuTestError(msg) => IpcError {
                code: "CPU_TEST_ERROR".to_string(),
                message: "CPU测试失败".to_string(),
                details: Some(msg),
            },
            crate::benchmark::error::BenchmarkError::MemoryTestError(msg) => IpcError {
                code: "MEMORY_TEST_ERROR".to_string(),
                message: "内存测试失败".to_string(),
                details: Some(msg),
            },
            crate::benchmark::error::BenchmarkError::StorageTestError(msg) => IpcError {
                code: "STORAGE_TEST_ERROR".to_string(),
                message: "存储测试失败".to_string(),
                details: Some(msg),
            },
            crate::benchmark::error::BenchmarkError::DataSaveError(msg) => IpcError {
                code: "DATA_SAVE_ERROR".to_string(),
                message: "数据保存失败".to_string(),
                details: Some(msg),
            },
            crate::benchmark::error::BenchmarkError::PermissionError(msg) => IpcError {
                code: "PERMISSION_ERROR".to_string(),
                message: "权限不足".to_string(),
                details: Some(msg),
            },
        }
    }
}