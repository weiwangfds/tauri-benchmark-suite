#[derive(Debug, thiserror::Error)]
pub enum BenchmarkError {
    #[error("系统信息获取失败: {0}")]
    SystemInfoError(String),
    
    #[error("CPU测试失败: {0}")]
    CpuTestError(String),
    
    #[error("内存测试失败: {0}")]
    MemoryTestError(String),
    
    #[error("存储测试失败: {0}")]
    StorageTestError(String),
    
    #[error("数据保存失败: {0}")]
    DataSaveError(String),
    
    #[error("权限不足: {0}")]
    PermissionError(String),
}