import { createRouter, createWebHistory } from 'vue-router'
import BenchmarkSuite from '../components/BenchmarkSuite.vue'
import TestRunner from '../components/TestRunner.vue'
import ResultsViewer from '../components/ResultsViewer.vue'

const routes = [
  {
    path: '/',
    name: 'BenchmarkSuite',
    component: BenchmarkSuite
  },
  {
    path: '/test',
    name: 'TestRunner',
    component: TestRunner
  },
  {
    path: '/results',
    name: 'ResultsViewer',
    component: ResultsViewer
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router