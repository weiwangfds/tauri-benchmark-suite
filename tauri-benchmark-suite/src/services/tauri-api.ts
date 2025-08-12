import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
    BenchmarkConfig,
    SystemInfo,
    TestResults,
    CpuTestConfig,
    CpuTestResult,
    MemoryTestConfig,
    MemoryTestResult,
    StorageTestConfig,
    StorageTestResult,
    TestStatus,
    TestSession,
    SystemMonitoringData,
    BenchmarkProgress,
    RealTimePerformanceData,
    TestWarningEvent,
    TestCompleteEvent,
    BenchmarkSuiteCompleteEvent,
    ProgressUpdate,
} from '../types';

/**
 * Tauri API 服务类
 * 封装所有与后端的通信逻辑
 */
export class TauriApiService {
    /**
     * 获取系统信息
     */
    static async getSystemInfo(): Promise<SystemInfo> {
        return await invoke<SystemInfo>('get_system_info');
    }

    /**
     * 启动完整基准测试套件
     */
    static async startBenchmarkSuite(config: BenchmarkConfig): Promise<string> {
        return await invoke<string>('start_benchmark_suite', { config });
    }

    /**
     * 取消基准测试
     */
    static async cancelBenchmark(sessionId: string): Promise<void> {
        return await invoke<void>('cancel_benchmark', { sessionId });
    }

    /**
     * 获取测试状态
     */
    static async getTestStatus(sessionId: string): Promise<TestStatus> {
        return await invoke<TestStatus>('get_test_status', { sessionId });
    }

    /**
     * 获取所有测试会话
     */
    static async getAllTestSessions(): Promise<TestSession[]> {
        return await invoke<TestSession[]>('get_all_test_sessions');
    }

    /**
     * 获取系统监控数据
     */
    static async getSystemMonitoringData(): Promise<SystemMonitoringData> {
        return await invoke<SystemMonitoringData>('get_system_monitoring_data');
    }

    /**
     * 清理已完成的测试会话
     */
    static async cleanupCompletedSessions(): Promise<number> {
        return await invoke<number>('cleanup_completed_sessions');
    }

    /**
     * 暂停基准测试
     */
    static async pauseBenchmark(sessionId: string): Promise<void> {
        return await invoke<void>('pause_benchmark', { sessionId });
    }

    /**
     * 恢复基准测试
     */
    static async resumeBenchmark(sessionId: string): Promise<void> {
        return await invoke<void>('resume_benchmark', { sessionId });
    }

    /**
     * 运行单个CPU基准测试
     */
    static async runCpuBenchmark(config: CpuTestConfig): Promise<CpuTestResult> {
        return await invoke<CpuTestResult>('run_cpu_benchmark', { config });
    }

    /**
     * 运行单个内存基准测试
     */
    static async runMemoryBenchmark(config: MemoryTestConfig): Promise<MemoryTestResult> {
        return await invoke<MemoryTestResult>('run_memory_benchmark', { config });
    }

    /**
     * 运行单个存储基准测试
     */
    static async runStorageBenchmark(config: StorageTestConfig): Promise<StorageTestResult> {
        return await invoke<StorageTestResult>('run_storage_benchmark', { config });
    }

    /**
     * 监听基准测试进度事件
     */
    static async onBenchmarkProgress(callback: (progress: BenchmarkProgress) => void) {
        return await listen<BenchmarkProgress>('benchmark-progress', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听系统监控数据事件
     */
    static async onSystemMonitoring(callback: (data: SystemMonitoringData) => void) {
        return await listen<SystemMonitoringData>('system-monitoring', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听实时性能数据事件
     */
    static async onRealTimePerformance(callback: (data: RealTimePerformanceData) => void) {
        return await listen<RealTimePerformanceData>('real-time-performance', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听测试警告事件
     */
    static async onTestWarning(callback: (warning: TestWarningEvent) => void) {
        return await listen<TestWarningEvent>('test-warning', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听测试完成事件
     */
    static async onTestComplete(callback: (event: TestCompleteEvent) => void) {
        return await listen<TestCompleteEvent>('test-complete', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听基准测试套件完成事件
     */
    static async onBenchmarkComplete(callback: (event: BenchmarkSuiteCompleteEvent) => void) {
        return await listen<BenchmarkSuiteCompleteEvent>('benchmark-complete', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听测试错误事件
     */
    static async onTestError(callback: (event: TestCompleteEvent) => void) {
        return await listen<TestCompleteEvent>('test-error', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听CPU测试进度事件
     */
    static async onCpuTestProgress(callback: (progress: ProgressUpdate) => void) {
        return await listen<ProgressUpdate>('cpu-test-progress', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听内存测试进度事件
     */
    static async onMemoryTestProgress(callback: (progress: ProgressUpdate) => void) {
        return await listen<ProgressUpdate>('memory-test-progress', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 监听存储测试进度事件
     */
    static async onStorageTestProgress(callback: (progress: ProgressUpdate) => void) {
        return await listen<ProgressUpdate>('storage-test-progress', (event) => {
            callback(event.payload);
        });
    }

    /**
     * 取消所有事件监听器
     */
    static async removeAllListeners() {
        // 这里可以存储所有的 unlisten 函数并调用它们
        // 实际实现中需要管理监听器的生命周期
    }
}

/**
 * 错误处理工具函数
 */
export function handleTauriError(error: any): string {
    if (typeof error === 'string') {
        return error;
    }
    if (error?.message) {
        return error.message;
    }
    return '未知错误';
}

/**
 * 事件监听器管理类
 */
export class EventListenerManager {
    private listeners: Array<() => void> = [];

    async addListener<T>(eventName: string, callback: (data: T) => void) {
        const unlisten = await listen<T>(eventName, (event) => {
            callback(event.payload);
        });
        this.listeners.push(unlisten);
        return unlisten;
    }

    removeAllListeners() {
        this.listeners.forEach(unlisten => unlisten());
        this.listeners = [];
    }
}