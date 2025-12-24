# 任务 2：抽象系统级操作 (解除 OS 耦合)

## 1. 目标
将插件中散落的操作系统特定代码（如调用资源管理器）收归到 Kernel 统一管理，确保插件保持纯粹的业务逻辑。

## 2. 当前问题
- `FileManager` 插件直接调用 `explorer` 命令。
- 代码中散布 `#[cfg(target_os = "windows")]`。
- 违反了中介者模式：插件应该通过 `AppCommand` 提出需求，由 Host 决定如何实现。

## 3. 技术要求
1. **扩展全局指令集**：
   - 在 `src/lib.rs` 的 `AppCommand` 中添加：
     - `RevealInShell(PathBuf)`：在系统文件管理器中定位。
     - `CopyToClipboard(String)`：将字符串拷贝到系统剪贴板。
2. **中心化实现系统操作**：
   - 将 `FileManager` 中的 `reveal_in_explorer` 逻辑移动到 `src/app.rs`。
   - 在 `VerbiumApp::process_commands` 中统一处理这些系统指令，使用多平台适配宏（Windows/Linux/macOS）。
3. **插件清理**：
   - 移除插件中所有对 `std::process::Command` 的直接调用。
   - 将 UI 点击事件改为向 `control` 向量推送对应的 `AppCommand`。

## 4. 涉及文件
- `src/lib.rs` (枚举定义)
- `src/app.rs` (系统调用实现)
- `src/plugins/file_manager/mod.rs` (清理冗余逻辑)

## 5. 验收标准
- 插件代码中不再包含 `#[cfg(target_os = ...)]` 或平台特定的 Shell 命令。
- “在资源管理器中显示”功能依然正常工作。
