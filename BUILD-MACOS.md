# DictNavi macOS 应用构建指南

## 概述

由于 macOS 需要 Apple 的工具链和 SDK，**无法在 Windows 上直接交叉编译 macOS 应用**。本文档提供了几种在 Windows 环境下构建 macOS 应用的解决方案。

## 方案一：使用 GitHub Actions（推荐）

这是最简单且免费的方法，适合大多数开发者。**现在可以同时构建 Windows 和 macOS 版本！**

### 步骤

1. **将代码推送到 GitHub**
   ```bash
   git add .
   git commit -m "Update application"
   git push origin master
   ```

2. **自动构建**
   - 推送代码后，GitHub Actions 会自动构建 Windows 和 macOS 版本
   - 构建完成后，在 Actions 标签页下载构建产物
   - 可以分别下载 `DictNavi-Windows` 或 `DictNavi-macOS`

3. **手动触发构建**
   - 在 GitHub 仓库页面，点击 "Actions" 标签
   - 选择 "Build All Platforms" workflow
   - 点击 "Run workflow" 手动触发

### 优势

- ✅ 完全免费（GitHub 提供免费构建时间）
- ✅ 无需本地 macOS 或 Windows 环境
- ✅ **同时构建 Windows 和 macOS 版本**
- ✅ 自动化构建流程
- ✅ 支持自动发布到 Releases（推送版本标签时）

## 方案二：在 macOS 上本地构建

如果你有 macOS 设备（Mac、MacBook 或 macOS 虚拟机），可以直接在 macOS 上构建。

### 前置要求

- macOS 10.13 (High Sierra) 或更高版本
- Rust 工具链（安装方法：`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`）

### 构建步骤

1. **克隆或传输代码到 macOS**
   ```bash
   git clone <your-repo-url>
   cd DictNavi
   ```

2. **安装 Rust 目标平台**
   ```bash
   rustup target add x86_64-apple-darwin
   ```

3. **构建应用**
   ```bash
   chmod +x build-macos.sh
   ./build-macos.sh
   ```

4. **打包应用**
   ```bash
   chmod +x package-macos.sh
   ./package-macos.sh
   ```

5. **分发应用**
   - 将 `dist/DictNavi-macOS` 目录压缩为 ZIP 文件
   - 用户解压后即可使用

## 方案三：使用 macOS 云服务

如果需要在 Windows 上直接操作，可以使用 macOS 云服务：

- **MacStadium**：提供 macOS 云服务器（付费）
- **AWS EC2 Mac instances**：Amazon 提供的 macOS 实例（付费）
- **MacinCloud**：macOS 远程桌面服务（付费）

## 构建脚本说明

### build-macos.sh

用于编译 macOS 发布版本：
- 清理旧的构建文件
- 编译 `x86_64-apple-darwin` 目标平台
- 输出到 `target/x86_64-apple-darwin/release/DictNavi`

### package-macos.sh

用于打包 macOS 应用：
- 复制可执行文件到 `dist/DictNavi-macOS`
- 复制 `words` 目录和索引文件
- 创建 README.txt 使用说明

## 打包内容

打包后的 `dist/DictNavi-macOS` 目录包含：

```
DictNavi-macOS/
├── DictNavi          # 主程序（可执行文件）
├── words/            # 字典数据目录
│   ├── .index/       # 搜索索引（自动生成）
│   └── *.json        # 单词定义文件
└── README.txt        # 使用说明
```

## macOS 应用签名（可选）

如果要发布到 App Store 或让用户信任应用，需要进行代码签名：

```bash
# 需要 Apple Developer 账号
codesign --sign "Developer ID Application: Your Name" DictNavi
```

## 常见问题

### Q: 为什么不能在 Windows 上直接构建 macOS 应用？

A: macOS 应用需要 Apple 的工具链（Xcode）和 macOS SDK，这些只能在 macOS 系统上使用。Apple 不允许在其他平台上使用这些工具。

### Q: GitHub Actions 构建需要多长时间？

A: 首次构建可能需要 10-20 分钟（下载依赖），后续构建通常 5-10 分钟。

### Q: 构建产物在哪里下载？

A: 在 GitHub 仓库的 Actions 标签页，点击对应的 workflow run，在 Artifacts 部分下载。

### Q: 用户如何运行应用？

A: 用户解压 ZIP 文件后，双击 `DictNavi` 即可运行。如果 macOS 显示安全警告，需要右键点击选择"打开"。

### Q: 需要创建 .app 包吗？

A: 当前版本是简单的可执行文件，如果需要更标准的 macOS 应用体验，可以创建 `.app` 包结构（需要额外的打包脚本）。

## 相关文件

- `build-macos.sh` - macOS 本地构建脚本
- `package-macos.sh` - macOS 本地打包脚本
- `.github/workflows/build-all.yml` - GitHub Actions 统一构建工作流（Windows + macOS）
- `.github/workflows/build-macos.yml` - GitHub Actions macOS 单独构建工作流（可选）
- `BUILD.md` - Windows 构建指南

