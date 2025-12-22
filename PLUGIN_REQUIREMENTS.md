# 插件开发需求与改动建议

为了支持 `CodeEditorPlugin` 插件的完整功能（特别是语法高亮），需要对项目基础架构进行以下调整。

## 1. 新增依赖项

在 `Cargo.toml` 中添加 `egui_extras` 依赖。该库提供了 `egui` 官方支持的语法高亮和高级 UI 组件。

**建议改动：**

```toml
# Cargo.toml

[dependencies]
# ... 现有依赖
egui_extras = { version = "0.29.1", features = ["syntax_highlighting"] }
```

## 2. 插件实现说明

已在 `src/plugins/code_editor/mod.rs` 中实现了基础的编辑器插件。

- **行号支持**：已通过 `egui` 内置的 `.code_editor()` 方法实现。
- **语法高亮**：目前在代码中已预留 `layouter` 逻辑，但由于缺少 `egui_extras` 依赖，该部分代码暂时处于注释状态。一旦添加依赖并取消注释，编辑器将支持 Rust 等语言的语法高亮。
- **菜单集成**：插件已注册到 `Tab` 菜单中，名称为 "Code Editor"。

## 3. 后续优化建议

- **语言切换**：可以在 `CodeEditorTab` 的 UI 中增加一个下拉框，允许用户选择不同的编程语言以切换高亮规则。
- **文件 IO**：目前编辑器内容仅存在于内存中，后续可增加 `Save` 和 `Open` 命令与核心系统的文件操作进行对接。
