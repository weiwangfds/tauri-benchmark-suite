import { ref, reactive, computed, onUnmounted } from 'vue';
import { TauriApiService, EventListenerManager, handleTauriError } from '../services/tauri-api';
import {
    TestStatus,
} from '../types';
import type {
    BenchmarkConfig,
    SystemInfo,
    TestResults,
    TestSession,
    SystemMonitoringData,
    BenchmarkProgress,
    RealTimePerformanceData,
    TestWarningEvent,
    TestCompleteEvent,
    BenchmarkSuiteCompleteEvent,
} from '../types';

/**
 * 基准测试组合式 API
 */
export function useBenchmark() {
    // 响应式状态
    const systemInfo = ref<SystemInfo | null>(null);
    const currentSession = ref<string | null>(null);
    const testStatus = ref<TestStatus>(TestStatus.Pending);
    const testResults = ref<TestResults | null>(null);
    const allSessions = ref<TestSession[]>([]);
    const isLoading = ref(false);
    const error = ref<string | null>(null);

    // 进度相关状态
    const progress = reactive({
        overall: 0,
        current: 0,
        currentTest: '',
        message: '',
        estimatedTimeRemaining: null as number | null,
    });

    // 监控数据
    const monitoringData = ref<SystemMonitoringData | null>(null);
    const performanceData = ref<RealTimePerformanceData[]>([]);
    const warnings = ref<TestWarningEvent[]>([]);

    // 事件监听器管理
    const eventManager = new EventListenerManager();

    // 计算属性
    const isRunning = computed(() => testStatus.value === 'Running');
    const isCompleted = computed(() => testStatus.value === 'Completed');
    const isFailed = computed(() => testStatus.value === 'Failed');
    const isCancelled = computed(() => testStatus.value === 'Cancelled');

    /**
     * 初始化系统信息
     */
    const initializeSystemInfo = async () => {
        try {
            isLoading.value = true;
            error.value = null;
            systemInfo.value = await TauriApiService.getSystemInfo();
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to get system info:', err);
        } finally {
            isLoading.value = false;
        }
    };

    /**
     * 启动基准测试套件
     */
    const startBenchmarkSuite = async (config: BenchmarkConfig) => {
        try {
            isLoading.value = true;
            error.value = null;
            
            const sessionId = await TauriApiService.startBenchmarkSuite(config);
            currentSession.value = sessionId;
            testStatus.value = TestStatus.Running;
            
            // 重置进度和数据
            progress.overall = 0;
            progress.current = 0;
            progress.currentTest = '';
            progress.message = '正在启动测试...';
            performanceData.value = [];
            warnings.value = [];
            
            return sessionId;
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to start benchmark suite:', err);
            throw err;
        } finally {
            isLoading.value = false;
        }
    };

    /**
     * 取消基准测试
     */
    const cancelBenchmark = async () => {
        if (!currentSession.value) return;
        
        try {
            await TauriApiService.cancelBenchmark(currentSession.value);
            testStatus.value = TestStatus.Cancelled;
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to cancel benchmark:', err);
        }
    };

    /**
     * 暂停基准测试
     */
    const pauseBenchmark = async () => {
        if (!currentSession.value) return;
        
        try {
            await TauriApiService.pauseBenchmark(currentSession.value);
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to pause benchmark:', err);
        }
    };

    /**
     * 恢复基准测试
     */
    const resumeBenchmark = async () => {
        if (!currentSession.value) return;
        
        try {
            await TauriApiService.resumeBenchmark(currentSession.value);
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to resume benchmark:', err);
        }
    };

    /**
     * 获取测试状态
     */
    const getTestStatus = async (sessionId?: string) => {
        const targetSessionId = sessionId || currentSession.value;
        if (!targetSessionId) return;
        
        try {
            const status = await TauriApiService.getTestStatus(targetSessionId);
            if (targetSessionId === currentSession.value) {
                testStatus.value = status;
            }
            return status;
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to get test status:', err);
        }
    };

    /**
     * 获取所有测试会话
     */
    const getAllTestSessions = async () => {
        try {
            allSessions.value = await TauriApiService.getAllTestSessions();
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to get all test sessions:', err);
        }
    };

    /**
     * 获取系统监控数据
     */
    const getSystemMonitoringData = async () => {
        try {
            monitoringData.value = await TauriApiService.getSystemMonitoringData();
        } catch (err) {
            console.error('Failed to get system monitoring data:', err);
        }
    };

    /**
     * 清理已完成的会话
     */
    const cleanupCompletedSessions = async () => {
        try {
            const cleanedCount = await TauriApiService.cleanupCompletedSessions();
            await getAllTestSessions(); // 刷新会话列表
            return cleanedCount;
        } catch (err) {
            error.value = handleTauriError(err);
            console.error('Failed to cleanup completed sessions:', err);
        }
    };

    /**
     * 设置事件监听器
     */
    const setupEventListeners = async () => {
        try {
            // 监听基准测试进度
            await eventManager.addListener<BenchmarkProgress>('benchmark-progress', (data) => {
                if (data.session_id === currentSession.value) {
                    progress.overall = data.overall_progress;
                    progress.current = data.test_progress;
                    progress.currentTest = data.current_test;
                    progress.message = data.message;
                    progress.estimatedTimeRemaining = data.estimated_time_remaining || null;
                }
            });

            // 监听系统监控数据
            await eventManager.addListener<SystemMonitoringData>('system-monitoring', (data) => {
                monitoringData.value = data;
            });

            // 监听实时性能数据
            await eventManager.addListener<RealTimePerformanceData>('real-time-performance', (data) => {
                if (data.session_id === currentSession.value) {
                    performanceData.value.push(data);
                    // 保持最近的100条记录
                    if (performanceData.value.length > 100) {
                        performanceData.value = performanceData.value.slice(-100);
                    }
                }
            });

            // 监听测试警告
            await eventManager.addListener<TestWarningEvent>('test-warning', (data) => {
                if (data.session_id === currentSession.value) {
                    warnings.value.push(data);
                }
            });

            // 监听测试完成
            await eventManager.addListener<TestCompleteEvent>('test-complete', (data) => {
                if (data.session_id === currentSession.value) {
                    if (!data.success) {
                        error.value = data.error || '测试失败';
                        testStatus.value = TestStatus.Failed;
                    }
                }
            });

            // 监听基准测试套件完成
            await eventManager.addListener<BenchmarkSuiteCompleteEvent>('benchmark-complete', (data) => {
                if (data.session_id === currentSession.value) {
                    if (data.success && data.results) {
                        testResults.value = data.results;
                        testStatus.value = TestStatus.Completed;
                    } else {
                        error.value = data.error || '测试套件失败';
                        testStatus.value = TestStatus.Failed;
                    }
                }
            });

            // 监听测试错误
            await eventManager.addListener<TestCompleteEvent>('test-error', (data) => {
                if (data.session_id === currentSession.value) {
                    error.value = data.error || '测试出现错误';
                    testStatus.value = TestStatus.Failed;
                }
            });

        } catch (err) {
            console.error('Failed to setup event listeners:', err);
        }
    };

    /**
     * 清理资源
     */
    const cleanup = () => {
        eventManager.removeAllListeners();
        currentSession.value = null;
        testStatus.value = TestStatus.Pending;
        testResults.value = null;
        error.value = null;
        progress.overall = 0;
        progress.current = 0;
        progress.currentTest = '';
        progress.message = '';
        performanceData.value = [];
        warnings.value = [];
    };

    // 组件卸载时清理资源
    onUnmounted(() => {
        cleanup();
    });

    return {
        // 状态
        systemInfo,
        currentSession,
        testStatus,
        testResults,
        allSessions,
        isLoading,
        error,
        progress,
        monitoringData,
        performanceData,
        warnings,

        // 计算属性
        isRunning,
        isCompleted,
        isFailed,
        isCancelled,

        // 方法
        initializeSystemInfo,
        startBenchmarkSuite,
        cancelBenchmark,
        pauseBenchmark,
        resumeBenchmark,
        getTestStatus,
        getAllTestSessions,
        getSystemMonitoringData,
        cleanupCompletedSessions,
        setupEventListeners,
        cleanup,
    };
}