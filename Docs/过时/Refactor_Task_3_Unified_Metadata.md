# 任务 3：统一元数据源 (消除双源真理)

## 1. 目标
消除插件代码硬编码名称与 `plugin.toml` 定义不一致的风险，确保 Launcher 与核心代码的元数据同步。

## 2. 当前问题
- 插件 ID 定义在 `plugin.toml`。
- 插件名称硬编码在 `mod.rs` 的 `fn name(&self)`。
- 如果两者不一致，拓扑排序（依赖 ID 匹配）将导致逻辑崩溃。

## 3. 技术要求
1. **修改 Build Script (`build.rs`)**：
   - 在生成 `generated.rs` 的循环中，读取每个插件目录下的 `plugin.toml`。
   - 解析其中的 `plugin.name`。
   - 为每个插件生成一个与其 ID 绑定的常量，例如 `pub const PLUGIN_NAME_CODE_EDITOR: &str = "code_editor";`。
2. **重构 Plugin Trait 实现**：
   - 修改所有插件的 `mod.rs`，不再返回硬编码字符串。
   - 引用 `generated.rs` 中生成的常量作为 `name()` 的返回值。
3. **自动化校验**：
   - 在 `src/plugins/mod.rs` 的 `all_plugins()` 函数中，增加一段校验逻辑：对比生成的元数据与实例化后的插件 `name()` 是否一致，不一致则触发 `panic!` 或编译期警告。

## 4. 涉及文件
- `build.rs` (生成逻辑)
- `src/plugins/generated.rs` (目标生成文件)
- 各插件目录下的 `mod.rs` (实现重构)

## 5. 验收标准
- 修改任一 `plugin.toml` 中的 `name` 字段并重新编译后，程序应能自动识别新名称，无需手动修改 `mod.rs` 代码。
- 拓扑排序依然能够正确识别依赖关系。
