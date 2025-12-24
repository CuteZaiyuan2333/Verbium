# 任务 1：重构文件打开流程 (异步 IO 化)

## 1. 目标
将同步的文件读取操作从主 UI 线程移出，解决打开大文件时界面卡死的问题，确保 Verbium 符合“响应式注入”的设计思想。

## 2. 当前问题
- `AppCommand::OpenFile` 在 `app.rs` 中同步触发插件的 `try_open_file`。
- `CodeEditorPlugin` 在 `try_open_file` 内部直接调用 `std::fs::read_to_string`。
- 如果文件较大或处于慢速磁盘，主线程将阻塞，导致 UI 掉帧或无响应。

## 3. 技术要求
1. **定义加载状态**：
   - 在 `src/lib.rs` 或插件内部创建一个 `LoadingTab` 结构体，实现 `TabInstance` Trait，显示“Loading...”文案及旋转进度条。
2. **异步执行读取**：
   - 插件的 `try_open_file` 在匹配到路径后，不再直接读取内容，而是立即返回 `LoadingTab`。
   - 在返回前，启动 `std::thread::spawn` 或使用异步 Runtime 任务进行文件读取。
3. **引入回调指令**：
   - 在 `AppCommand` 枚举中增加 `ReplaceTab { id: u64, new_tab: Tab }` 指令。
   - 后台线程读取完成后，将结果封装成真正的 `CodeEditorTab`，并通过 `ReplaceTab` 指令请求主程序替换掉之前的 `LoadingTab`。
4. **主程序适配**：
   - 修改 `src/app.rs` 中的 `process_commands`，实现 `ReplaceTab` 逻辑：根据 ID 找到旧 Tab 并替换其内容。

## 4. 涉及文件
- `src/lib.rs` (AppCommand 定义)
- `src/app.rs` (指令处理逻辑)
- `src/plugins/code_editor/mod.rs` (读取逻辑重构)

## 5. 验收标准
- 打开 10MB 以上的文件时，UI 依然流畅，且能看到“加载中”状态。
- 文件读取完成后，内容能正确显示在原本的标签页位置。
