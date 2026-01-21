# Verbium Launcher 设计规范

Launcher 是 Verbium 生态系统的重要组成部分，负责管理静态插件的编译配置。

## 1. 核心职责

为了兼顾性能与灵活性，Verbium 采用 **自举 (Self-Bootstrapping)** 架构。传统的独立启动器已被废弃，取而代之的是内置的 `manager` 插件（亦称为集成启动器）。

1.  **扫描**：遍历 `src/plugins/` 目录，通过识别 `plugin.toml` 发现可用插件。
2.  **配置生成**：主程序的 `build.rs` 自动读取 `plugin.toml` 并生成 `src/plugins/generated.rs`。
    - **模块定义**：自动生成的 `pub mod <name>;`。
    - **常量绑定**：生成 `PLUGIN_NAME_<ID>` 常量，用于编译期标识对齐。
    - **静态注册**：生成 `get_extra_plugins` 函数，实现插件的自动实例化。
3.  **依赖与特征注入**：`manager` 插件直接修改根目录 `Cargo.toml`：
    - **外部依赖**：将 `[external_dependencies]` 注入到 `# --- BEGIN PLUGIN DEPENDENCIES ---` 标记区。
    - **Features 同步**：自动生成 `plugin_<name>` 特征，并根据用户勾选状态更新 `default` 特征列表。
4.  **构建环境管理**：利用 `cargo` 命令流实现编译、清理、运行及特定版本的导出。

## 2. 插件发现协议

插件必须定义在 `src/plugins/` 的子目录下，并包含一个规范的 `plugin.toml`。

### plugin.toml 格式定义
```toml
[plugin]
name = "my_plugin"           # 唯一标识符（必须是合法的 Rust 模块名）
display_name = "我的插件"    # 显示在 Launcher 列表中的友好名称
version = "0.1.0"            # 插件版本
author = "Your Name"         # 作者信息
description = "插件功能描述"  # 插件简述
dependencies = ["core"]      # 内部插件依赖顺序（用于拓扑排序）

[external_dependencies]
# 将会被自动注入到根 Cargo.toml 的 [dependencies] 中
serde = { version = "1.0", features = ["derive"] }
rand = "0.8"
```

## 3. 配置文件同步逻辑

为了避免手动修改 `Cargo.toml` 导致冲突，`manager` 插件拥有其特定区域的管理权：

- **依赖注入点**：
  ```toml
  # --- BEGIN PLUGIN DEPENDENCIES ---
  # ... 自动生成：From <plugin_a> & <plugin_b> ...
  # ... 重复项将自动合并 ...
  # --- END PLUGIN DEPENDENCIES ---
  ```
- **特征同步**：`manager` 会自动在 `[features]` 节下维护 `plugin_*` 列表，并根据启用状态重写 `default = [...]`。

## 4. 元数据共享与校验

- **编译期常量**：插件实现 `name()` 方法时，**必须**引用 `crate::plugins::PLUGIN_NAME_...`。
- **一致性校验**：`generated.rs` 在实例化时会通过 `assert_eq!(p.name(), CONST_NAME)` 强制验证 Rust 实现中的名称与 `plugin.toml` 配置是否一致，防止配置漂移。

## 5. UI 交互流 (Integrated Launcher)

1.  **环境检查**：启动时检测 `launcher_config.toml`，自动加载项目路径及上次启用的插件状态。
2.  **插件列表**：中心面板显示所有扫描到的插件，点击复选框可实时更改待编译功能。
3.  **配置面板**：底部支持选择构建模式（Debug/Release）、勾选 "Compile & Start" 联动开关。
4.  **控制台交互**：所有 `cargo` 输出（stdout/stderr）会被重定向到右侧的 Console 面板，支持滚动追踪。
5.  **一键同步与运行**：点击 "▶ Build & Run" 后，系统按顺序执行：同步 `Cargo.toml` -> 调用 `cargo run` -> 进程自杀（或由 Cargo 接管新窗口）。