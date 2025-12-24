# 任务 4：全局通知与错误处理系统

## 1. 目标
为插件提供统一的反馈机制，确保错误发生时用户有感知，同时保持 UI 风格的一致性。

## 2. 当前问题
- 插件目前大量使用 `if let Ok` 吞掉错误。
- 缺乏向用户展示操作结果（如“保存成功”、“删除失败”）的统一 UI 机制。

## 3. 技术要求
1. **指令与数据结构**：
   - 在 `src/lib.rs` 中定义 `NotificationLevel { Info, Success, Warning, Error }`。
   - 在 `AppCommand` 中添加 `Notify { message: String, level: NotificationLevel }`。
2. **状态管理**：
   - 在 `VerbiumApp` 结构体中添加 `notifications: Vec<NotificationInstance>`，包含消息、级别和剩余显示时长。
3. **渲染 Toast UI**：
   - 在 `app.rs` 的 `update` 循环末尾，使用 `egui::Area` 或 `egui::Window` 在屏幕右下角渲染一个非阻塞的通知列表。
   - 实现自动消失逻辑（随着每帧流逝减少时长）。
4. **插件重构**：
   - 修改 `FileManager` 和 `CodeEditor` 的 IO 代码。
   - 捕捉错误并将其转化为 `AppCommand::Notify` 推送到队列。

## 4. 涉及文件
- `src/lib.rs` (数据结构定义)
- `src/app.rs` (UI 渲染逻辑)
- `src/plugins/file_manager/mod.rs` (错误处理接入)
- `src/plugins/code_editor/mod.rs` (错误处理接入)

## 5. 验收标准
- 当删除文件失败或保存文件成功时，屏幕右下角应弹出对应的彩色通知条。
- 通知在显示 3-5 秒后自动平滑消失。
