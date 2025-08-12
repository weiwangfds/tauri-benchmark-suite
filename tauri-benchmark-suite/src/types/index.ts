// 基准测试配置接口
export interface BenchmarkConfig {
    cpuTest: {
        enabled: boolean;
        duration: number; // seconds
        threadCount: number;
    };
    memoryTest: {
        enabled: boolean;
        bufferSize: number; // MB
        iterations: number;
    };
    storageTest: {
        enabled: boolean;
        fileSize: number; // MB
        blockSize: number; // KB
    };
}

// 测试结果接口
export interface TestResults {
    timestamp: string;
    systemInfo: SystemInfo;
    cpuResults?: CpuTestResult;
    memoryResults?: MemoryTestResult;
    storageResults?: StorageTestResult;
    overallScore: number;
}

export interface CpuTestResult {
    single_thread_score: number;
    multi_thread_score: number;
    floating_point_score: number;
    average_temperature: number;
    max_temperature: number;
    test_duration: number; // seconds
    operations_per_second: number;
}

// CPU测试配置接口
export interface CpuTestConfig {
    thread_count: number; // 0 means use all available threads
    test_duration: number; // seconds
    enable_temperature_monitoring: boolean;
}

export interface MemoryTestResult {
    sequential_read_speed: number; // MB/s
    sequential_write_speed: number; // MB/s
    random_access_speed: number; // MB/s
    latency: number; // nanoseconds
    memory_usage_peak: number; // MB
    error_rate: number; // percentage
    test_duration: number; // seconds
}

// 内存测试配置接口
export interface MemoryTestConfig {
    buffer_size: number; // MB
    iterations: number;
    test_duration: number; // seconds
    enable_usage_monitoring: boolean;
}

export interface StorageTestResult {
    sequential_read: StorageMetrics;
    sequential_write: StorageMetrics;
    random_read: StorageMetrics;
    random_write: StorageMetrics;
    test_duration: number; // seconds
    total_data_processed: number; // MB
}

export interface StorageMetrics {
    throughput: number; // MB/s
    iops: number;
    latency: number; // milliseconds
}

// 存储测试配置接口
export interface StorageTestConfig {
    file_size: number; // MB
    block_size: number; // KB
    test_duration: number; // seconds
    test_file_path?: string; // 可选的测试文件路径
}

// 系统信息接口
export interface SystemInfo {
    os: string;
    cpu: {
        name: string;
        vendor: string;
        cores: number;
        threads: number;
        base_frequency: number;
        max_frequency: number;
        architecture: string;
        cache_info: {
            l1_data?: number;
            l1_instruction?: number;
            l2?: number;
            l3?: number;
        };
    };
    memory: {
        total: number; // GB
        available: number; // GB
        used: number; // GB
        memory_type: string;
        speed: number; // MHz
        slots_used: number;
        slots_total: number;
    };
    storage: Array<{
        name: string;
        storage_type: 'SSD' | 'HDD' | 'NVMe' | 'Unknown';
        capacity: number; // GB
        available: number; // GB
        interface: string;
        file_system: string;
        mount_point: string;
    }>;
    system_details: {
        hostname: string;
        uptime: number; // seconds
        boot_time: number; // timestamp
        kernel_version: string;
        total_processes: number;
        temperatures: Record<string, number>; // component -> temperature
    };
}

// IPC 通信相关接口
export interface BenchmarkProgress {
    session_id: string;
    current_test: string;
    overall_progress: number;
    test_progress: number;
    message: string;
    estimated_time_remaining?: number; // seconds
}

export interface SystemMonitoringData {
    cpu_usage: number;
    memory_usage: number;
    temperature?: number;
    timestamp: string;
}

export interface TestSession {
    session_id: string;
    status: TestStatus;
    start_time: string;
    end_time?: string;
    config?: BenchmarkConfig;
}

export interface RealTimePerformanceData {
    session_id: string;
    test_type: string;
    metrics: Record<string, number>;
    timestamp: string;
}

export interface TestWarningEvent {
    session_id: string;
    test_type: string;
    warning_type: string;
    message: string;
    severity: WarningSeverity;
}

export enum WarningSeverity {
    Low = 'Low',
    Medium = 'Medium',
    High = 'High',
    Critical = 'Critical',
}

export enum TestStatus {
    Pending = 'Pending',
    Running = 'Running',
    Completed = 'Completed',
    Failed = 'Failed',
    Cancelled = 'Cancelled',
}

export interface TestCompleteEvent {
    session_id: string;
    test_type: string;
    success: boolean;
    result?: any;
    error?: string;
}

export interface BenchmarkSuiteCompleteEvent {
    session_id: string;
    success: boolean;
    results?: TestResults;
    error?: string;
}

export interface ProgressUpdate {
    progress: number;
    message: string;
    test_type: string;
}

export interface IpcError {
    code: string;
    message: string;
    details?: string;
}