# Verbium

**Verbium** 是一个基于 Rust 和 egui 构建的、高度可扩展的编辑器框架。

它采用独特的 **静态插件架构 (Static Plugin Architecture)**：插件源代码直接编译进主程序二进制文件中，由配套的 Launcher 工具管理配置和构建过程。这种设计在保证 Rust 极致性能和类型安全的同时，提供了灵活的功能扩展能力。

## 核心特性

- **模块化设计**：核心功能极简，几乎所有功能（包括代码编辑、文件管理）都由插件提供。
- **静态链接**：插件作为 Rust 模块编译，无 DLL/WASM 开销，享受完整的编译器优化。
- **响应式注入**：插件通过生命周期钩子（Hooks）将 UI 和逻辑注入到主程序的各个位置（菜单、Tab、全局界面）。
- **中介者模式通信**：插件间通过 `AppCommand` 协议解耦，支持跨插件打开文件、管理布局等交互。

## 快速开始

### 前置要求
- Rust (Cargo) 环境
- Windows / macOS / Linux

### 运行
目前项目包含一个核心 Launcher（负责配置）和编辑器本体。

1.  **启动 Launcher (推荐)**:
    ```bash
    cd launcher
    cargo run
    ```
    Launcher 将扫描 `src/plugins` 目录，并允许你选择启用哪些插件，然后一键编译运行编辑器。

2.  **直接运行编辑器 (开发模式)**:
    如果 `Cargo.toml` 已配置好，可以直接在根目录运行：
    ```bash
    cargo run
    ```

## 目录结构

- `src/` - 编辑器源代码
    - `lib.rs` / `app.rs` - 核心框架与窗口管理
    - `plugins/` - 插件目录（在此处创建新文件夹以添加插件）
- `launcher/` - 插件管理与构建工具源码
- `Docs/` - 开发文档
    - `Plugin_Development.md` - 插件开发指南与 API 参考
    - `Architecture.md` - 系统架构设计说明
    - `Launcher_Design.md` - Launcher 设计规范

## 许可证
MIT
