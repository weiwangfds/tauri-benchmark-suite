# 发布指南

## 自动发布流程

本项目配置了 GitHub Actions 自动构建和发布流程，支持 Windows、macOS 和 Linux 三个平台的自动打包。

### 触发发布

#### 方法一：推送版本标签（推荐）

1. 更新版本号：
   ```bash
   # 更新 package.json 中的版本号
   npm version patch  # 补丁版本 (0.1.0 -> 0.1.1)
   npm version minor  # 次要版本 (0.1.0 -> 0.2.0)  
   npm version major  # 主要版本 (0.1.0 -> 1.0.0)
   ```

2. 推送标签：
   ```bash
   git push origin main --tags
   ```

#### 方法二：手动触发

1. 访问 GitHub 仓库的 Actions 页面
2. 选择 "Release" 工作流
3. 点击 "Run workflow"
4. 输入版本号（如 v0.1.1）
5. 点击 "Run workflow" 按钮

### 发布产物

发布完成后，会在 GitHub Releases 页面生成以下文件：

#### Windows
- `tauri-benchmark-suite_0.1.0_x64_en-US.msi` - Windows 安装包
- `tauri-benchmark-suite_0.1.0_x64-setup.exe` - Windows 安装程序

#### macOS
- `tauri-benchmark-suite_0.1.0_aarch64.dmg` - Apple Silicon 版本
- `tauri-benchmark-suite_0.1.0_x64.dmg` - Intel 版本

#### Linux
- `tauri-benchmark-suite_0.1.0_amd64.deb` - Debian/Ubuntu 包
- `tauri-benchmark-suite_0.1.0_amd64.AppImage` - AppImage 格式

## 本地构建

### 开发环境要求

- Node.js 18+
- Rust 1.70+
- 平台特定依赖：
  - Windows: Visual Studio Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: libwebkit2gtk-4.0-dev, libappindicator3-dev 等

### 构建命令

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri:dev

# 构建发布版本
npm run release

# 仅构建 Tauri 应用
npm run tauri:build
```

## 版本管理

### 版本号规范

采用语义化版本控制 (SemVer)：

- **主版本号**：不兼容的 API 修改
- **次版本号**：向下兼容的功能性新增
- **修订号**：向下兼容的问题修正

### 更新检查清单

发布前请确认：

- [ ] 更新 `package.json` 中的版本号
- [ ] 更新 `src-tauri/tauri.conf.json` 中的版本号
- [ ] 更新 `src-tauri/Cargo.toml` 中的版本号
- [ ] 测试所有核心功能
- [ ] 检查构建是否成功
- [ ] 编写发布说明

## 故障排除

### 常见问题

1. **构建失败**：检查依赖是否正确安装
2. **签名问题**：确保证书配置正确（macOS/Windows）
3. **权限错误**：检查 GitHub Token 权限

### 调试方法

```bash
# 查看详细构建日志
npm run tauri:build -- --verbose

# 检查 Rust 环境
rustc --version
cargo --version

# 检查 Node.js 环境  
node --version
npm --version
```

## 联系方式

如有问题，请在 GitHub Issues 中提交反馈。