# DictNavi Windows 应用构建指南

## 快速开始

### 方法一：使用 GitHub Actions 自动构建（推荐）

**无需本地环境，自动构建 Windows 和 macOS 版本**

1. **推送代码到 GitHub**
   ```bash
   git add .
   git commit -m "Update application"
   git push origin master
   ```

2. **下载构建产物**
   - 在 GitHub 仓库页面，点击 "Actions" 标签
   - 选择最新的 workflow run
   - 在 Artifacts 部分下载 `DictNavi-Windows` 或 `DictNavi-macOS`

3. **手动触发构建**
   - 在 Actions 页面选择 "Build All Platforms"
   - 点击 "Run workflow" 手动触发

**优势**：
- ✅ 无需本地安装 Rust 环境
- ✅ 自动构建 Windows 和 macOS 版本
- ✅ 构建产物自动打包为 ZIP 文件
- ✅ 完全免费（GitHub 提供免费构建时间）

### 方法二：本地一键构建和打包

双击运行 `build-and-package.bat`，脚本会自动完成：
1. 编译发布版本
2. 打包所有文件到 `dist/DictNavi` 目录

### 方法三：分步执行

1. **编译应用**
   ```batch
   build.bat
   ```
   这会编译发布版本到 `target/release/DictNavi.exe`

2. **打包应用**
   ```batch
   package.bat
   ```
   这会将可执行文件和资源文件复制到 `dist/DictNavi` 目录

## 构建要求

- **Rust 工具链**：需要安装 Rust（https://www.rust-lang.org/）
- **Windows 系统**：Windows 7 或更高版本
- **磁盘空间**：至少 500MB 可用空间（用于编译和依赖）

## 打包内容

打包后的 `dist/DictNavi` 目录包含：

```
DictNavi/
├── DictNavi.exe      # 主程序（已优化，体积较大但性能好）
├── words/            # 字典数据目录
│   ├── .index/       # 搜索索引（自动生成）
│   └── *.json        # 单词定义文件
└── README.txt        # 使用说明
```

## 分发应用

1. 将 `dist/DictNavi` 目录压缩为 ZIP 文件
2. 用户解压后即可使用，无需安装
3. 确保 `words` 目录和 `DictNavi.exe` 在同一目录下

## 优化建议

### 减小可执行文件体积

如果可执行文件过大，可以在 `Cargo.toml` 中添加：

```toml
[profile.release]
strip = true          # 移除符号表
lto = true            # 链接时优化
codegen-units = 1     # 更好的优化
```

### 静态链接运行时库

默认情况下，Rust 会使用动态链接。如果需要静态链接所有依赖（生成完全独立的可执行文件），需要：

1. 安装 `x86_64-pc-windows-msvc` 工具链
2. 使用 `cargo build --release --target x86_64-pc-windows-msvc`

## 常见问题

### Q: 编译很慢怎么办？
A: 首次编译需要下载和编译所有依赖，可能需要 10-30 分钟。后续编译会快很多。

### Q: 可执行文件很大？
A: 这是正常的。发布版本包含所有依赖，通常 20-50MB。可以使用 `strip` 选项减小体积。

### Q: 用户需要安装什么？
A: 不需要！Rust 编译的 Windows 应用是独立的，不需要安装任何运行时库。

### Q: words 目录必须打包吗？
A: 是的，应用依赖 `words` 目录和其中的 `.index` 索引文件才能正常工作。

## 跨平台构建

### 使用 GitHub Actions 构建所有平台（推荐）

GitHub Actions 可以同时构建 Windows 和 macOS 版本，无需本地环境：

- **Windows 构建**：在 `windows-latest` runner 上自动构建
- **macOS 构建**：在 `macos-latest` runner 上自动构建
- **自动打包**：构建产物自动打包为 ZIP 文件
- **自动发布**：如果推送了版本标签（如 `v1.0.0`），会自动创建 GitHub Release

### macOS 本地构建

**重要**：由于 macOS 需要 Apple 工具链，无法在 Windows 上直接交叉编译。

请参考 [BUILD-MACOS.md](BUILD-MACOS.md) 了解如何在 macOS 设备上本地构建：
- **推荐方案**：使用 GitHub Actions 自动构建（免费）
- **备选方案**：在 macOS 设备上本地构建

## 高级选项

### 交叉编译

如果需要为其他平台编译，可以使用 `cargo build --release --target <target-triple>`

常用目标平台：
- `x86_64-pc-windows-msvc` - Windows 64位
- `x86_64-pc-windows-gnu` - Windows 64位（GNU工具链）
- `x86_64-apple-darwin` - macOS 64位（需要在 macOS 上构建）

### 构建优化

编辑 `Cargo.toml` 添加发布配置：

```toml
[profile.release]
opt-level = "z"       # 优化大小
strip = true          # 移除调试符号
lto = "fat"           # 全程序链接时优化
```

