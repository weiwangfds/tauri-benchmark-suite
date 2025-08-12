<template>
    <div class="benchmark-suite">
        <h2>基准测试套件</h2>

        <!-- 系统信息展示 -->
        <div class="system-info-section">
            <h3>系统信息</h3>
            <button @click="initializeSystemInfo" :disabled="isLoading" class="load-btn">
                {{ isLoading ? '加载中...' : '获取系统信息' }}
            </button>

            <div v-if="systemInfo" class="system-info">
                <div class="info-card">
                    <h4>操作系统</h4>
                    <p>{{ systemInfo.os }}</p>
                </div>

                <div class="info-card">
                    <h4>CPU信息</h4>
                    <p><strong>名称:</strong> {{ systemInfo.cpu.name }}</p>
                    <p><strong>厂商:</strong> {{ systemInfo.cpu.vendor }}</p>
                    <p><strong>架构:</strong> {{ systemInfo.cpu.architecture }}</p>
                    <p><strong>核心数:</strong> {{ systemInfo.cpu.cores }}</p>
                    <p><strong>线程数:</strong> {{ systemInfo.cpu.threads }}</p>
                    <p><strong>频率:</strong> {{ systemInfo.cpu.base_frequency }} MHz</p>
                </div>

                <div class="info-card">
                    <h4>内存信息</h4>
                    <p><strong>总内存:</strong> {{ systemInfo.memory.total }} GB</p>
                    <p><strong>可用内存:</strong> {{ systemInfo.memory.available }} GB</p>
                    <p><strong>已用内存:</strong> {{ systemInfo.memory.used }} GB</p>
                </div>

                <div class="info-card">
                    <h4>系统详情</h4>
                    <p><strong>主机名:</strong> {{ systemInfo.system_details.hostname }}</p>
                    <p><strong>运行时间:</strong> {{ formatUptime(systemInfo.system_details.uptime) }}</p>
                    <p><strong>进程数:</strong> {{ systemInfo.system_details.total_processes }}</p>
                </div>
            </div>

            <div v-if="error" class="error">
                <p>错误: {{ error }}</p>
            </div>
        </div>

        <!-- 测试会话管理 -->
        <div class="session-management">
            <h3>测试会话管理</h3>
            <div class="session-controls">
                <button @click="getAllTestSessions" :disabled="isLoading" class="session-btn">
                    刷新会话列表
                </button>
                <button @click="cleanupCompletedSessions" :disabled="isLoading" class="session-btn">
                    清理已完成会话
                </button>
            </div>
            
            <div v-if="allSessions.length > 0" class="sessions-list">
                 <h4>当前会话</h4>
                 <div v-for="session in allSessions" :key="session.session_id" class="session-item">
                     <span>{{ session.session_id }}</span>
                     <span class="session-status">{{ session.status }}</span>
                     <span class="session-time">{{ new Date(session.start_time).toLocaleString() }}</span>
                 </div>
             </div>
        </div>

        <!-- 系统监控数据 -->
        <div v-if="monitoringData" class="monitoring-section">
            <h3>系统监控</h3>
            <div class="monitoring-grid">
                <div class="monitoring-item">
                    <span>CPU使用率:</span>
                    <span>{{ monitoringData.cpu_usage.toFixed(1) }}%</span>
                </div>
                <div class="monitoring-item">
                    <span>内存使用率:</span>
                    <span>{{ monitoringData.memory_usage.toFixed(1) }}%</span>
                </div>
                <div v-if="monitoringData.temperature" class="monitoring-item">
                    <span>温度:</span>
                    <span>{{ monitoringData.temperature.toFixed(1) }}°C</span>
                </div>
            </div>
        </div>

        <!-- 基准测试套件控制 -->
        <div class="benchmark-suite-section">
            <h3>完整基准测试套件</h3>
            <div class="suite-config">
                 <div class="config-item">
                     <label>CPU测试时长 (秒):</label>
                     <input v-model.number="benchmarkConfig.cpuTest.duration" type="number" min="1" max="300" />
                 </div>
                 <div class="config-item">
                     <label>内存缓冲区大小 (MB):</label>
                     <input v-model.number="benchmarkConfig.memoryTest.bufferSize" type="number" min="1" max="1024" />
                 </div>
                 <div class="config-item">
                     <label>存储文件大小 (MB):</label>
                     <input v-model.number="benchmarkConfig.storageTest.fileSize" type="number" min="1" max="1024" />
                 </div>
                 <div class="config-item">
                     <label>
                         <input v-model="benchmarkConfig.cpuTest.enabled" type="checkbox" />
                         启用CPU测试
                     </label>
                 </div>
                 <div class="config-item">
                     <label>
                         <input v-model="benchmarkConfig.memoryTest.enabled" type="checkbox" />
                         启用内存测试
                     </label>
                 </div>
                 <div class="config-item">
                     <label>
                         <input v-model="benchmarkConfig.storageTest.enabled" type="checkbox" />
                         启用存储测试
                     </label>
                 </div>
             </div>

            <div class="suite-controls">
                 <button @click="() => startBenchmarkSuite(benchmarkConfig)" :disabled="isLoading || testStatus === TestStatus.Running" class="suite-btn start">
                     {{ testStatus === TestStatus.Running ? '测试进行中...' : '开始完整测试套件' }}
                 </button>
                 <button @click="pauseBenchmark" :disabled="testStatus !== TestStatus.Running" class="suite-btn pause">
                     暂停测试
                 </button>
                 <button @click="resumeBenchmark" :disabled="testStatus !== TestStatus.Running" class="suite-btn resume">
                     恢复测试
                 </button>
                 <button @click="cancelBenchmark" :disabled="testStatus === TestStatus.Pending" class="suite-btn cancel">
                     取消测试
                 </button>
             </div>

            <!-- 测试进度显示 -->
             <div v-if="progress" class="progress-section">
                 <h4>测试进度</h4>
                 <div class="progress-bar">
                     <div class="progress-fill" :style="{ width: progress.overall + '%' }"></div>
                     <span class="progress-text">{{ progress.overall.toFixed(1) }}%</span>
                 </div>
                 <div class="progress-details">
                     <div class="progress-item">
                         <span>当前测试:</span>
                         <span>{{ progress.currentTest }}</span>
                     </div>
                     <div class="progress-item">
                         <span>测试进度:</span>
                         <span>{{ progress.current.toFixed(1) }}%</span>
                     </div>
                     <div class="progress-item">
                         <span>预计剩余时间:</span>
                         <span>{{ progress.estimatedTimeRemaining || 0 }}秒</span>
                     </div>
                 </div>
             </div>

            <!-- 测试结果显示 -->
             <div v-if="testResults" class="results-section">
                 <h4>测试结果</h4>
                 <div class="results-summary">
                     <div class="result-item">
                         <span>总分:</span>
                         <span>{{ Math.round(testResults.overallScore) }}</span>
                     </div>
                     <div class="result-item">
                         <span>测试时间:</span>
                         <span>{{ new Date(testResults.timestamp).toLocaleString() }}</span>
                     </div>
                 </div>

                 <!-- CPU结果 -->
                 <div v-if="testResults.cpuResults" class="test-result">
                     <h5>CPU测试结果</h5>
                     <div class="result-grid">
                         <div class="result-item">
                             <span>单线程分数:</span>
                             <span>{{ Math.round(testResults.cpuResults.single_thread_score) }}</span>
                         </div>
                         <div class="result-item">
                             <span>多线程分数:</span>
                             <span>{{ Math.round(testResults.cpuResults.multi_thread_score) }}</span>
                         </div>
                         <div class="result-item">
                             <span>浮点运算分数:</span>
                             <span>{{ Math.round(testResults.cpuResults.floating_point_score) }}</span>
                         </div>
                     </div>
                 </div>

                 <!-- 内存结果 -->
                 <div v-if="testResults.memoryResults" class="test-result">
                     <h5>内存测试结果</h5>
                     <div class="result-grid">
                         <div class="result-item">
                             <span>顺序读取速度:</span>
                             <span>{{ Math.round(testResults.memoryResults.sequential_read_speed) }} MB/s</span>
                         </div>
                         <div class="result-item">
                             <span>顺序写入速度:</span>
                             <span>{{ Math.round(testResults.memoryResults.sequential_write_speed) }} MB/s</span>
                         </div>
                         <div class="result-item">
                             <span>随机访问速度:</span>
                             <span>{{ Math.round(testResults.memoryResults.random_access_speed) }} MB/s</span>
                         </div>
                     </div>
                 </div>

                 <!-- 存储结果 -->
                 <div v-if="testResults.storageResults" class="test-result">
                     <h5>存储测试结果</h5>
                     <div class="result-grid">
                         <div class="result-item">
                             <span>顺序读取:</span>
                             <span>{{ Math.round(testResults.storageResults.sequential_read.throughput) }} MB/s</span>
                         </div>
                         <div class="result-item">
                             <span>顺序写入:</span>
                             <span>{{ Math.round(testResults.storageResults.sequential_write.throughput) }} MB/s</span>
                         </div>
                         <div class="result-item">
                             <span>随机读取:</span>
                             <span>{{ Math.round(testResults.storageResults.random_read.throughput) }} MB/s</span>
                         </div>
                         <div class="result-item">
                             <span>随机写入:</span>
                             <span>{{ Math.round(testResults.storageResults.random_write.throughput) }} MB/s</span>
                         </div>
                     </div>
                 </div>
             </div>
        </div>

        <!-- 警告和错误显示 -->
        <div v-if="warnings.length > 0" class="warnings-section">
            <h4>测试警告</h4>
            <div v-for="warning in warnings" :key="warning.session_id" class="warning-item" :class="warning.severity.toLowerCase()">
                <span class="warning-message">{{ warning.message }}</span>
                <span class="warning-time">{{ new Date().toLocaleString() }}</span>
            </div>
        </div>

        <!-- CPU测试区域 -->
        <div class="cpu-test-section">
            <h3>CPU基准测试</h3>
            <div class="test-config">
                <div class="config-item">
                    <label>测试时长 (秒):</label>
                    <input v-model.number="cpuConfig.test_duration" type="number" min="1" max="300" />
                </div>
                <div class="config-item">
                    <label>线程数 (0=自动):</label>
                    <input v-model.number="cpuConfig.thread_count" type="number" min="0" max="32" />
                </div>
                <div class="config-item">
                    <label>
                        <input v-model="cpuConfig.enable_temperature_monitoring" type="checkbox" />
                        启用温度监控
                    </label>
                </div>
            </div>

            <button @click="runCpuTest" :disabled="cpuTesting" class="test-btn">
                {{ cpuTesting ? 'CPU测试中...' : '开始CPU测试' }}
            </button>

            <div v-if="cpuResult" class="test-result">
                <h4>CPU测试结果</h4>
                <div class="result-grid">
                    <div class="result-item">
                        <span>单线程分数:</span>
                        <span>{{ Math.round(cpuResult.single_thread_score) }}</span>
                    </div>
                    <div class="result-item">
                        <span>多线程分数:</span>
                        <span>{{ Math.round(cpuResult.multi_thread_score) }}</span>
                    </div>
                    <div class="result-item">
                        <span>浮点运算分数:</span>
                        <span>{{ Math.round(cpuResult.floating_point_score) }}</span>
                    </div>
                    <div class="result-item">
                        <span>每秒操作数:</span>
                        <span>{{ cpuResult.operations_per_second.toLocaleString() }}</span>
                    </div>
                    <div class="result-item">
                        <span>测试时长:</span>
                        <span>{{ cpuResult.test_duration }}秒</span>
                    </div>
                    <div v-if="cpuConfig.enable_temperature_monitoring" class="result-item">
                        <span>平均温度:</span>
                        <span>{{ cpuResult.average_temperature.toFixed(1) }}°C</span>
                    </div>
                </div>
            </div>

            <div v-if="cpuError" class="error">
                <p>CPU测试错误: {{ cpuError }}</p>
            </div>
        </div>

        <!-- 内存测试区域 -->
        <div class="memory-test-section">
            <h3>内存基准测试</h3>
            <div class="test-config">
                <div class="config-item">
                    <label>缓冲区大小 (MB):</label>
                    <input v-model.number="memoryConfig.buffer_size" type="number" min="1" max="1024" />
                </div>
                <div class="config-item">
                    <label>迭代次数:</label>
                    <input v-model.number="memoryConfig.iterations" type="number" min="1" max="100" />
                </div>
                <div class="config-item">
                    <label>测试时长 (秒):</label>
                    <input v-model.number="memoryConfig.test_duration" type="number" min="1" max="300" />
                </div>
                <div class="config-item">
                    <label>
                        <input v-model="memoryConfig.enable_usage_monitoring" type="checkbox" />
                        启用内存使用监控
                    </label>
                </div>
            </div>

            <button @click="runMemoryTest" :disabled="memoryTesting" class="test-btn">
                {{ memoryTesting ? '内存测试中...' : '开始内存测试' }}
            </button>

            <div v-if="memoryResult" class="test-result">
                <h4>内存测试结果</h4>
                <div class="result-grid">
                    <div class="result-item">
                        <span>顺序读取速度:</span>
                        <span>{{ Math.round(memoryResult.sequential_read_speed) }} MB/s</span>
                    </div>
                    <div class="result-item">
                        <span>顺序写入速度:</span>
                        <span>{{ Math.round(memoryResult.sequential_write_speed) }} MB/s</span>
                    </div>
                    <div class="result-item">
                        <span>随机访问速度:</span>
                        <span>{{ Math.round(memoryResult.random_access_speed) }} MB/s</span>
                    </div>
                    <div class="result-item">
                        <span>内存延迟:</span>
                        <span>{{ memoryResult.latency.toFixed(2) }} ns</span>
                    </div>
                    <div class="result-item">
                        <span>测试时长:</span>
                        <span>{{ memoryResult.test_duration }}秒</span>
                    </div>
                    <div v-if="memoryConfig.enable_usage_monitoring" class="result-item">
                        <span>峰值内存使用:</span>
                        <span>{{ memoryResult.memory_usage_peak }} MB</span>
                    </div>
                    <div class="result-item">
                        <span>错误率:</span>
                        <span>{{ memoryResult.error_rate.toFixed(2) }}%</span>
                    </div>
                </div>
            </div>

            <div v-if="memoryError" class="error">
                <p>内存测试错误: {{ memoryError }}</p>
            </div>
        </div>

        <!-- 存储测试区域 -->
        <div class="storage-test-section">
            <h3>存储基准测试</h3>
            <div class="test-config">
                <div class="config-item">
                    <label>文件大小 (MB):</label>
                    <input v-model.number="storageConfig.file_size" type="number" min="1" max="1024" />
                </div>
                <div class="config-item">
                    <label>块大小 (KB):</label>
                    <input v-model.number="storageConfig.block_size" type="number" min="1" max="1024" />
                </div>
                <div class="config-item">
                    <label>测试时长 (秒):</label>
                    <input v-model.number="storageConfig.test_duration" type="number" min="1" max="300" />
                </div>
                <div class="config-item">
                    <label>测试文件路径 (可选):</label>
                    <input v-model="storageConfig.test_file_path" type="text" placeholder="留空使用临时文件" />
                </div>
            </div>

            <button @click="runStorageTest" :disabled="storageTesting" class="test-btn">
                {{ storageTesting ? '存储测试中...' : '开始存储测试' }}
            </button>

            <div v-if="storageResult" class="test-result">
                <h4>存储测试结果</h4>
                <div class="result-grid">
                    <div class="result-item">
                        <span>顺序读取:</span>
                        <span>{{ Math.round(storageResult.sequential_read.throughput) }} MB/s ({{ storageResult.sequential_read.iops }} IOPS)</span>
                    </div>
                    <div class="result-item">
                        <span>顺序写入:</span>
                        <span>{{ Math.round(storageResult.sequential_write.throughput) }} MB/s ({{ storageResult.sequential_write.iops }} IOPS)</span>
                    </div>
                    <div class="result-item">
                        <span>随机读取:</span>
                        <span>{{ Math.round(storageResult.random_read.throughput) }} MB/s ({{ storageResult.random_read.iops }} IOPS)</span>
                    </div>
                    <div class="result-item">
                        <span>随机写入:</span>
                        <span>{{ Math.round(storageResult.random_write.throughput) }} MB/s ({{ storageResult.random_write.iops }} IOPS)</span>
                    </div>
                    <div class="result-item">
                        <span>平均延迟:</span>
                        <span>{{ ((storageResult.sequential_read.latency + storageResult.sequential_write.latency + storageResult.random_read.latency + storageResult.random_write.latency) / 4).toFixed(2) }} ms</span>
                    </div>
                    <div class="result-item">
                        <span>测试时长:</span>
                        <span>{{ storageResult.test_duration }}秒</span>
                    </div>
                    <div class="result-item">
                        <span>处理数据量:</span>
                        <span>{{ storageResult.total_data_processed }} MB</span>
                    </div>
                </div>
            </div>

            <div v-if="storageError" class="error">
                <p>存储测试错误: {{ storageError }}</p>
            </div>
        </div>

        <!-- 测试选择区域 -->
        <div class="test-selection">
            <h3>测试完成</h3>
            <p>所有基准测试功能已实现！你可以分别运行CPU、内存和存储测试来评估工控机性能。</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useBenchmark } from '../composables/useBenchmark'
import { TestStatus } from '../types'
import { TauriApiService } from '../services/tauri-api'
import type { BenchmarkConfig, TestWarningEvent, CpuTestResult, MemoryTestResult, StorageTestResult } from '../types'

// 使用基准测试组合式 API
const {
    systemInfo,
    currentSession,
    testStatus,
    testResults,
    allSessions,
    isLoading,
    error,
    progress,
    monitoringData,
    initializeSystemInfo,
    startBenchmarkSuite,
    cancelBenchmark,
    pauseBenchmark,
    resumeBenchmark,
    getAllTestSessions,
    getSystemMonitoringData,
    cleanupCompletedSessions,
    setupEventListeners,
    cleanup
} = useBenchmark()

// 本地状态
const warnings = ref<TestWarningEvent[]>([])
const benchmarkConfig = ref<BenchmarkConfig>({
    cpuTest: {
        enabled: true,
        duration: 30,
        threadCount: 0
    },
    memoryTest: {
        enabled: true,
        bufferSize: 10,
        iterations: 5
    },
    storageTest: {
        enabled: true,
        fileSize: 10,
        blockSize: 4
    }
})

// 单独测试的配置和状态
const cpuConfig = ref({
    test_duration: 30,
    thread_count: 0,
    enable_temperature_monitoring: false
})

const memoryConfig = ref({
    buffer_size: 10,
    iterations: 5,
    test_duration: 30,
    enable_usage_monitoring: false
})

const storageConfig = ref({
    file_size: 10,
    block_size: 4,
    test_duration: 30,
    test_file_path: ''
})

// 测试状态
const cpuTesting = ref(false)
const memoryTesting = ref(false)
const storageTesting = ref(false)

// 测试结果
const cpuResult = ref<CpuTestResult | null>(null)
const memoryResult = ref<MemoryTestResult | null>(null)
const storageResult = ref<StorageTestResult | null>(null)

// 测试错误
const cpuError = ref('')
const memoryError = ref('')
const storageError = ref('')

// 测试方法
const runCpuTest = async () => {
    cpuTesting.value = true
    cpuError.value = ''
    cpuResult.value = null
    
    try {
        const result = await TauriApiService.runCpuBenchmark(cpuConfig.value)
        cpuResult.value = result
    } catch (error) {
        cpuError.value = error instanceof Error ? error.message : '未知错误'
    } finally {
        cpuTesting.value = false
    }
}

const runMemoryTest = async () => {
    memoryTesting.value = true
    memoryError.value = ''
    memoryResult.value = null
    
    try {
        const result = await TauriApiService.runMemoryBenchmark(memoryConfig.value)
        memoryResult.value = result
    } catch (error) {
        memoryError.value = error instanceof Error ? error.message : '未知错误'
    } finally {
        memoryTesting.value = false
    }
}

const runStorageTest = async () => {
    storageTesting.value = true
    storageError.value = ''
    storageResult.value = null
    
    try {
        const result = await TauriApiService.runStorageBenchmark(storageConfig.value)
        storageResult.value = result
    } catch (error) {
        storageError.value = error instanceof Error ? error.message : '未知错误'
    } finally {
        storageTesting.value = false
    }
}

// 生命周期钩子
onMounted(async () => {
    await initializeSystemInfo()
    setupEventListeners()
    
    // 定期获取系统监控数据
    const monitoringInterval = setInterval(async () => {
        if (testStatus.value === 'Running') {
            await getSystemMonitoringData()
        }
    }, 2000)
    
    // 清理定时器
    onUnmounted(() => {
        clearInterval(monitoringInterval)
        cleanup()
    })
})

const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400)
    const hours = Math.floor((seconds % 86400) / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)

    if (days > 0) {
        return `${days}天 ${hours}小时 ${minutes}分钟`
    } else if (hours > 0) {
        return `${hours}小时 ${minutes}分钟`
    } else {
        return `${minutes}分钟`
    }
}
</script>

<style scoped>
.benchmark-suite {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
}

.system-info-section {
    margin-bottom: 2rem;
}

.load-btn {
    background-color: #3498db;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 14px;
    margin-bottom: 1rem;
}

.load-btn:hover:not(:disabled) {
    background-color: #2980b9;
}

.load-btn:disabled {
    background-color: #bdc3c7;
    cursor: not-allowed;
}

.system-info {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
    margin-top: 1rem;
}

.info-card {
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 1rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.info-card h4 {
    color: #2c3e50;
    margin-bottom: 0.5rem;
    border-bottom: 2px solid #3498db;
    padding-bottom: 0.25rem;
}

.info-card p {
    margin: 0.25rem 0;
    color: #555;
}

.error {
    background-color: #f8d7da;
    color: #721c24;
    padding: 1rem;
    border-radius: 5px;
    border: 1px solid #f5c6cb;
    margin-top: 1rem;
}

.test-selection {
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.test-selection h3 {
    color: #2c3e50;
    margin-bottom: 1rem;
}

.cpu-test-section {
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
}

.cpu-test-section h3 {
    color: #2c3e50;
    margin-bottom: 1rem;
}

.test-config {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
}

.config-item {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.config-item label {
    font-weight: 500;
    color: #555;
}

.config-item input[type="number"] {
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
}

.config-item input[type="checkbox"] {
    margin-right: 0.5rem;
}

.test-btn {
    background-color: #27ae60;
    color: white;
    border: none;
    padding: 12px 24px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 16px;
    font-weight: 500;
    margin-bottom: 1rem;
}

.test-btn:hover:not(:disabled) {
    background-color: #219a52;
}

.test-btn:disabled {
    background-color: #bdc3c7;
    cursor: not-allowed;
}

.test-result {
    background: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    padding: 1rem;
    margin-top: 1rem;
}

.test-result h4 {
    color: #2c3e50;
    margin-bottom: 1rem;
}

.result-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 0.5rem;
}

.result-item {
    display: flex;
    justify-content: space-between;
    padding: 0.5rem;
    background: white;
    border-radius: 4px;
    border: 1px solid #dee2e6;
}

.result-item span:first-child {
    font-weight: 500;
    color: #555;
}

.result-item span:last-child {
    font-weight: 600;
    color: #2c3e50;
}

.memory-test-section {
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
}

.memory-test-section h3 {
    color: #2c3e50;
    margin-bottom: 1rem;
}

.storage-test-section {
    background: white;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    margin-bottom: 2rem;
}

.storage-test-section h3 {
    color: #2c3e50;
    margin-bottom: 1rem;
}

.config-item input[type="text"] {
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
}
</style>