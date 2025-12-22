# Verbium 系统架构设计

本文档详细描述了 Verbium 编辑器框架的核心架构原则、模块划分及通信机制。

## 1. 核心设计原则

### 1.1 静态插件架构 (Static Plugin Architecture)
与常见的动态链接库 (DLL/so) 或 WASM 插件系统不同，Verbium 选择将插件**静态编译**到最终的可执行文件中。
- **优点**：
    - **性能**：零运行时开销，享受全程序优化 (LTO)。
    - **安全**：利用 Rust 的编译期类型检查，避免 ABI 兼容性问题。
    - **简单**：无需复杂的动态加载器或沙盒环境。
- **缺点**：添加/移除插件需要重新编译（由 Launcher 自动化处理）。

### 1.2 响应式注入 (Reactive Injection)
主程序 (Host) 不持有特定插件的引用，而是定义了一系列**挂载点 (Hooks)**。在渲染每一帧时，主程序会遍历所有已注册的插件，调用其对应的方法，允许插件将自己的 UI “画”到指定位置。

## 2. 系统模块

### 2.1 核心层 (Core / Kernel)
位于 `src/lib.rs` 和 `src/app.rs`。
- **职责**：
    - 管理窗口生命周期 (`eframe`).
    - 管理 Docking 布局 (`egui_dock`).
    - 维护插件列表与加载顺序 (`src/plugins/mod.rs` 拓扑排序).
    - 消息分发 (Command Dispatch).
- **特点**：不知道具体业务逻辑，只负责调度。

### 2.2 插件层 (Plugins)
位于 `src/plugins/`。每个子目录为一个独立插件。
- **Core Plugin**：提供基础功能（退出、布局重置、关于页面）。
- **User Plugins**：实现具体业务（如 Code Editor, File Manager）。

### 2.3 Launcher
位于 `launcher/`。
- **职责**：
    - 扫描插件目录 (`plugin.toml`).
    - 修改根目录 `Cargo.toml` 注入依赖。
    - 生成注册代码。
    - 调用 `cargo` 命令构建项目。

## 3. 通信与交互机制

为了保持插件解耦，Verbium 采用了基于消息队列的中介者模式。

### 3.1 AppCommand 协议
所有状态变更必须通过 `AppCommand` 枚举发送到命令队列，由主程序在帧末统一处理。

```rust
pub enum AppCommand {
    OpenTab(Tab),            // 直接打开一个 Tab 实例
    CloseTab(String),        // 根据标题关闭 Tab
    TileAll,                 // 平铺布局
    ResetLayout,             // 重置布局
    OpenFile(PathBuf),       // [v0.2] 请求打开文件
    ToggleSettings,          // [v0.2] 打开设置面板
}
```

### 3.2 文件打开流程 (Mediator Pattern)
为了让“文件管理器”不依赖“代码编辑器”，我们设计了如下流程：

1.  **发起**：插件 A (File Manager) 发送 `AppCommand::OpenFile(path)`.
2.  **中介**：主程序捕获该命令。
3.  **询问**：主程序遍历插件列表，依次调用 `plugin.try_open_file(path)`.
4.  **响应**：插件 B (Code Editor) 检查文件扩展名。如果支持，返回 `Some(TabInstance)`.
5.  **执行**：主程序接收到 Tab 实例，将其添加到 UI 布局中。

### 3.3 设置系统
统一的设置界面由主程序托管。
- 插件实现 `on_settings_ui(&mut self, ui: &mut Ui)`.
- 用户点击 "Settings" 时，主程序弹出模态窗口，并遍历调用所有插件的 `on_settings_ui`，将它们的配置界面聚合在一个滚动列表中。

## 4. 目录结构规范

- **项目根目录**：包含 `Cargo.toml` (由 Launcher 托管) 和 `PLUGIN_REQUIREMENTS.md`.
- **src/plugins/**：插件源码仓库。每个插件必须包含 `mod.rs` 和可选的 `plugin.toml`.
- **Docs/**：项目文档。
