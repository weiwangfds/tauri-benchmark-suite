import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { BenchmarkConfig, TestResults, ProgressData } from '../types'

export const useBenchmarkStore = defineStore('benchmark', () => {
  // 状态
  const config = ref<BenchmarkConfig>({
    cpuTest: {
      enabled: true,
      duration: 60,
      threadCount: 0 // 0 表示使用所有可用线程
    },
    memoryTest: {
      enabled: true,
      bufferSize: 1024, // 1GB
      iterations: 100
    },
    storageTest: {
      enabled: true,
      fileSize: 1024, // 1GB
      blockSize: 4 // 4KB
    }
  })

  const currentResults = ref<TestResults | null>(null)
  const testHistory = ref<TestResults[]>([])
  const isTestRunning = ref(false)
  const testProgress = ref<ProgressData | null>(null)

  // 动作
  const updateConfig = (newConfig: Partial<BenchmarkConfig>) => {
    config.value = { ...config.value, ...newConfig }
  }

  const setTestResults = (results: TestResults) => {
    currentResults.value = results
    testHistory.value.push(results)
  }

  const setTestRunning = (running: boolean) => {
    isTestRunning.value = running
  }

  const updateProgress = (progress: ProgressData) => {
    testProgress.value = progress
  }

  const clearProgress = () => {
    testProgress.value = null
  }

  return {
    // 状态
    config,
    currentResults,
    testHistory,
    isTestRunning,
    testProgress,
    // 动作
    updateConfig,
    setTestResults,
    setTestRunning,
    updateProgress,
    clearProgress
  }
})