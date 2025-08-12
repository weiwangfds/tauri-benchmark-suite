# Tauri基准测试套件

这是一个基于Tauri的基准测试应用程序，专门用于在不同的Windows工控机上运行性能测试，帮助用户找出最适合其应用场景的工控机配置。

## 功能特性

- CPU性能测试（单线程、多线程、浮点运算）
- 内存性能测试（读写速度、延迟测试）
- 存储性能测试（顺序/随机读写、IOPS）
- 系统信息收集和硬件检测
- 测试结果可视化和报告导出
- 历史记录和性能对比分析

## 技术栈

- **前端**: Vue.js 3 + TypeScript + Pinia + Vue Router
- **后端**: Rust + Tauri
- **UI组件**: Element Plus
- **图表**: Chart.js / Vue-ChartJS
- **数据库**: SQLite
- **构建工具**: Vite

## 开发环境设置

### 前置要求

- Node.js (>= 16)
- Rust (>= 1.70)
- Tauri CLI

### 安装依赖

```bash
# 安装前端依赖
npm install

# 安装Tauri CLI（如果尚未安装）
npm install -g @tauri-apps/cli
```

### 开发模式

```bash
# 启动开发服务器
npm run tauri dev
```

### 构建应用

```bash
# 构建生产版本
npm run tauri build
```

## 项目结构

```
tauri-benchmark-suite/
├── src/                    # Vue.js前端源码
│   ├── components/         # Vue组件
│   ├── stores/            # Pinia状态管理
│   ├── router/            # Vue Router路由
│   ├── types/             # TypeScript类型定义
│   └── assets/            # 静态资源
├── src-tauri/             # Rust后端源码
│   ├── src/
│   │   ├── benchmark/     # 基准测试模块
│   │   ├── lib.rs         # 库入口
│   │   └── main.rs        # 主程序入口
│   ├── Cargo.toml         # Rust依赖配置
│   └── tauri.conf.json    # Tauri配置
├── package.json           # Node.js依赖配置
└── README.md             # 项目说明
```

## 使用说明

1. 启动应用程序
2. 在"测试套件"页面选择要运行的测试项目
3. 配置测试参数（可选）
4. 在"执行测试"页面运行基准测试
5. 在"查看结果"页面查看测试结果和性能报告
6. 导出报告或进行历史对比分析

## 许可证

MIT License