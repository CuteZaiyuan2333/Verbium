# Bevy 引擎项目编辑器集成方案报告

## 1. 概述

本报告旨在探讨如何利用 **Verbium** 的静态插件架构，构建一个功能完备的 **Bevy Engine** 项目编辑器。基于 Bevy 的数据驱动（ECS）特性和 Verbium 的响应式注入机制，我们可以将编辑器拆分为多个协同工作的插件。

## 2. 核心插件构思

为了实现一个可用的编辑器，建议实现以下五个核心插件：

### 2.1 Bevy Viewport (视口插件)
*   **功能**：实时渲染 Bevy 场景。支持相机控制（平移、旋转、缩放）和实体拾取（Gizmos）。
*   **技术实现**：
    *   通过 `bevy_egui` 将 Bevy 的渲染输出定向到 wgpu 纹理。
    *   在 Verbium 的 Tab 中显示该纹理。
    *   将 egui 的输入事件（鼠标、键盘）转发给 Bevy 引擎。

### 2.2 Scene Hierarchy (场景层级插件)
*   **功能**：以树状结构展示当前世界中的所有 Entity。支持搜索、隐藏/显示、父子关系调整。
*   **交互**：点击 Entity 时，通过 `AppCommand` 发送 `SelectEntity(Entity)` 消息，通知 Inspector 插件更新。

### 2.3 Inspector (属性检查器插件)
*   **功能**：显示并编辑选中 Entity 的 Component 数据。
*   **关键技术**：
    *   利用 Bevy 的 `Reflect` 特性自动生成 UI 控件（类似于 `bevy-inspector-egui`）。
    *   通过 Verbium 的 `on_settings_ui` 模式扩展，支持自定义组件的编辑界面。

### 2.4 Asset Browser (资源管理器插件)
*   **功能**：专门针对 Bevy 资源（.png, .gltf, .ron 等）的预览和管理。
*   **交互**：支持从资源浏览器拖拽模型或贴图直接进入 Viewport 插件以放置或替换资源。

### 2.5 Bevy Runner & Console (运行控制与日志插件)
*   **功能**：控制游戏状态（运行、暂停、单帧步进）。捕获并分类显示 Bevy 的系统日志。

---

## 3. 技术挑战与解决方案

### 3.1 渲染上下文共享
*   **挑战**：Verbium 使用 `eframe` (egui)，Bevy 也有自己的渲染循环。
*   **方案**：采用 "Bevy as a Sub-system" 模式。Verbium 插件持有 Bevy `App` 实例。在插件的 `update` 钩子中手动触发 Bevy 的 `update()`，并利用 `wgpu` 的跨库纹理共享功能将渲染结果提交给 egui。

### 3.2 跨插件通信 (Mediator Pattern)
*   **挑战**：层级插件如何告诉检查器插件哪个物体被选中？
*   **方案**：扩展 `AppCommand` 协议，增加 Bevy 专用指令：
    ```rust
    pub enum AppCommand {
        // ... 原有指令
        BevySelectEntity(EntityId),
        BevySetPaused(bool),
        BevySpawnPrefab(PathBuf),
    }
    ```

### 3.3 依赖管理 (Launcher 角色)
*   **挑战**：Bevy 是一个大型依赖，不应在所有 Verbium 项目中强制存在。
*   **方案**：利用 Verbium Launcher 的 `external_dependencies` 功能。只有在启用 `bevy_editor` 插件时，Launcher 才将 `bevy = "0.x"` 注入到主 `Cargo.toml`。

---

## 4. 实施路线图

1.  **Phase 1 (MVP)**: 实现简单的 Viewport 插件，能在 Verbium 中跑通 Bevy 的 Hello World 渲染。
2.  **Phase 2 (Data Flow)**: 实现 Hierarchy 和基本的 Inspector，验证 `AppCommand` 在两个插件间的同步。
3.  **Phase 3 (Polishing)**: 引入 Gizmos (操作轴) 支持，优化资源管理器的预览功能。
4.  **Phase 4 (Integration)**: 完善 Launcher 模板，使得用户可以通过 Launcher 一键创建 "Verbium-Bevy Project"。

## 5. 结论

基于 Verbium 的架构，实现 Bevy 编辑器不仅可行，而且具有极高的性能优势。静态链接确保了编辑器与引擎之间的高速数据交换，而 Tab 系统则为多窗口、多场景的复杂编辑提供了天然的支持。
