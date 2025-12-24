# Refactor Task 5: File Manager Interaction Upgrade

本文档详细说明了提升 `file_manager` 插件交互体验的设计方案，主要包括多选逻辑（Ctrl/Shift）和拖拽移动功能（Drag & Drop）。

## 1. 目标 (Objectives)
在保持插件独立性的前提下，实现符合桌面操作系统习惯的文件管理操作：
- **多选支持**：通过 Ctrl 键点选和 Shift 键范围选择。
- **拖拽移动**：支持将选中的文件或文件夹拖拽到其他文件夹中。
- **批量操作**：上下文菜单（右键菜单）支持对所有选中项进行统一操作。

## 2. 状态管理重构 (State Management)

### 2.1 选中状态 (SelectionState)
引入一个专门的结构体来管理复杂的选中逻辑。

```rust
struct SelectionState {
    /// 当前选中的所有路径
    selected: HashSet<PathBuf>,
    /// 最后一个交互的路径（作为 Shift 选择的锚点）
    last_interacted: Option<PathBuf>,
    /// 当前帧中所有可见节点的线性索引映射
    /// key: 路径, value: 渲染顺序索引
    visible_nodes: Vec<PathBuf>,
}
```

### 2.2 拖拽上下文 (DragContext)
使用 `egui` 的 `Id` 和 `payload` 系统来跟踪拖拽状态。

## 3. 核心逻辑实现

### 3.1 线性化渲染 (Linearization)
由于 `egui` 是即时模式 UI，且文件树是递归渲染的，为了支持 Shift 范围选择，必须在渲染循环中：
1.  每渲染一个项（无论是文件夹还是文件），递增一个计数器。
2.  将该项的 `PathBuf` 存入 `visible_nodes` 向量中。
3.  通过比较 `last_interacted` 在向量中的索引和当前项的索引，确定 Shift 选择的范围。

### 3.2 交互响应 (Interaction Handling)
在 `render_tree` 的 `selectable_label` 响应中处理：

- **普通点击**：
    - `selection.selected.clear()`
    - `selection.selected.insert(current_path)`
    - `selection.last_interacted = Some(current_path)`
- **Ctrl + 点击**：
    - 切换 `current_path` 在 `selection.selected` 中的存在状态。
    - `selection.last_interacted = Some(current_path)`
- **Shift + 点击**：
    - 获取 `last_interacted` 的索引 `A`。
    - 获取 `current_path` 的索引 `B`。
    - 将索引 `min(A, B)` 到 `max(A, B)` 之间的所有路径加入 `selection.selected`。

### 3.3 拖拽移动 (Drag & Drop)
1.  **Drag Source**：使用 `ui.dnd_drag_source` 包裹文件/文件夹项。
    - 如果拖拽开始时项未选中，则先将其设为单选选中。
    - 拖拽的 Payload 为 `Vec<PathBuf>` (即 `selection.selected`)。
2.  **Drop Target**：仅文件夹节点和根目录区域接受放置。
    - 使用 `ui.dnd_drop_zone`。
    - 放置成功后，遍历 Payload，执行 `std::fs::rename(src, target_dir.join(src.file_name()))`。
    - 发送 `AppCommand::Notify` 反馈结果。

## 4. 约束与安全
- **禁止循环移动**：在执行移动前，检查目标路径是否是被拖拽项本身或其子目录。
- **原子性操作**：虽然 `std::fs` 很难保证跨卷移动的原子性，但应对批量移动进行错误收集，并在结束后统一通知。
- **UI 隔离**：所有逻辑必须包含在 `src/plugins/file_manager/` 内，不得修改 `src/app.rs` 或其他核心组件。

## 5. 开发步骤
1.  **Phase 1**：在 `FileExplorerTab` 中引入 `SelectionState` 并实现基础的 Ctrl 点选。
2.  **Phase 2**：实现 `visible_nodes` 的收集逻辑和 Shift 范围选择。
3.  **Phase 3**：集成 `egui` 的 DnD API 实现文件移动。
4.  **Phase 4**：重构上下文菜单，使其支持 `selected` 集合的批量删除和路径复制。
