# Verbium

Verbium 是一款采用**静态插件架构**的高性能、可扩展代码编辑器。本项目现已实现“自举”架构，将启动器逻辑完全集成于内部插件中。

## 🖼 预览 (Preview)

![Verbium Screenshot](./Verbium%202026_1_21%2018_39_16.png)

## 🚀 快速开始

Verbium 采用“启动器驱动”的构建模式。为了获得最佳体验，请遵循以下流程：

1.  **准备环境**：确保你的系统已安装 [Rust 编译环境](https://www.rust-lang.org/)。
2.  **获取源码**：克隆本仓库到本地。
    ```bash
    git clone https://github.com/CuteZaiyuan2333/Verbium.git
    ```
3.  **下载启动器**：从本仓库的 [Releases](https://github.com/CuteZaiyuan2333/Verbium/releases) 页面下载最新版本的 `verbium-launcher.exe`。
4.  **配置与编译**：
    *   运行下载的 `verbium-launcher.exe`。
    *   在界面中选择你刚才克隆的项目根目录。
    *   根据需求勾选插件（如 Terminal, Code Editor 等）。
    *   点击 **▶ Build & Run**，启动器将自动完成依赖同步并为你编译出完整的编辑器。

## 🏗 自举架构 (Self-Bootstrapping)

我们通过一个轻量级的独立启动器来引导整个编辑器的生命周期：

1.  **外部引导**：用户通过预编译的 `verbium-launcher.exe` 介入构建流程，无需手动修改配置文件。
2.  **动态进化**：启动器根据用户勾选的插件实时重写 `Cargo.toml`，并调用 `cargo` 编译出针对当前环境优化的生产版本。
3.  **开发闭环**：在生成的编辑器内部，依然保留了 `manager` 插件（集成启动器），方便开发者在运行期间随时调整配置并重新构建。

## 🛠 功能特性

*   **内置启动器**：支持项目路径管理、插件开关、依赖自动同步。
*   **导出功能**：支持将编译好的特定版本（例如仅包含特定工具的编辑器）导出到指定目录。
*   **清理工具**：内置 `cargo clean` 支持。
*   **静态编译**：享受 Rust 全程序优化 (LTO) 带来的极致性能。

## 📦 部署建议

直接从 GitHub Releases 下载预编译的 `verbium-launcher.exe`（仅包含启动器功能的轻量级版本）作为你的“编辑器管理中心”。或者，你也可以直接使用 `cargo build --release` 生成的全功能版本。

## 📜 许可证

本项目采用 [MIT License](LICENSE) 开源。