<script setup lang="ts">
import { RouterView } from 'vue-router'
</script>

<template>
  <div id="app">
    <header class="app-header">
      <h1>Tauri基准测试套件</h1>
      <nav>
        <router-link to="/">测试套件</router-link>
        <router-link to="/test">执行测试</router-link>
        <router-link to="/results">查看结果</router-link>
      </nav>
    </header>
    
    <main class="app-main">
      <RouterView />
    </main>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background-color: #f5f5f5;
}

#app {
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.app-header {
  background-color: #2c3e50;
  color: white;
  padding: 1rem 2rem;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.app-header h1 {
  margin-bottom: 1rem;
  font-size: 1.5rem;
}

.app-header nav {
  display: flex;
  gap: 2rem;
}

.app-header nav a {
  color: white;
  text-decoration: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  transition: background-color 0.3s;
}

.app-header nav a:hover,
.app-header nav a.router-link-active {
  background-color: #34495e;
}

.app-main {
  flex: 1;
  overflow-y: auto;
  padding: 2rem;
}
</style>