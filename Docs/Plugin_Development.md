# Verbium 插件开发指南

本指南将帮助你编写 Verbium 插件。插件是 Verbium 的一等公民，可以控制从菜单栏到编辑器面板的几乎所有内容。

## 1. 插件结构

在 `src/plugins/` 下创建一个新文件夹（例如 `my_tool`），并包含以下文件：

### 1.1 `mod.rs` (逻辑实现)
必须包含一个实现了 `Plugin` trait 的结构体，以及一个导出的 `create()` 函数。

```rust
use egui::Ui;
use crate::{Plugin, AppCommand};

pub struct MyToolPlugin;

impl Plugin for MyToolPlugin {
    fn name(&self) -> &str { "my_tool" }
    
    // 实现钩子方法...
}

pub fn create() -> MyToolPlugin {
    MyToolPlugin
}
```

### 1.2 `plugin.toml` (元数据与依赖)
可选。如果你的插件需要外部 crate（如 serde, reqwest），必须在此声明。

```toml
[plugin]
name = "my_tool"
version = "0.1.0"
description = "A sample tool"

[external_dependencies]
serde = { version = "1.0", features = ["derive"] }
```

---

## 2. API 参考 (Trait Hooks)

`Plugin` trait 定义了以下钩子方法：

### UI 注入类
| 方法 | 描述 |
| :--- | :--- |
| `on_file_menu` | 注入内容到顶部 "File" 菜单。 |
| `on_tab_menu` | 注入内容到顶部 "Tab" 菜单。 |
| `on_menu_bar` | 在菜单栏添加自定义的顶级菜单（如 "Tools", "Help"）。 |
| `on_global_ui` | 绘制全局覆盖层（如弹窗、通知）。 |
| `on_settings_ui` | **[v0.2]** 绘制插件的配置选项到全局设置窗口中。 |

### 逻辑处理类
| 方法 | 描述 |
| :--- | :--- |
| `update` | 每帧调用。处理后台逻辑。 |
| `dependencies` | 返回依赖的插件名称列表，用于控制加载顺序。 |
| `try_open_file` | **[v0.2]** 询问插件是否能打开给定路径的文件。返回 `Option<Box<dyn TabInstance>>`。 |

---

## 3. 功能实现示例

### 3.1 创建自定义 Tab
如果你的插件需要显示一个工作区（如编辑器、浏览器），你需要实现 `TabInstance`。

```rust
use crate::TabInstance;

#[derive(Debug, Clone)]
pub struct MyTab {
    content: String,
}

impl TabInstance for MyTab {
    fn title(&self) -> egui::WidgetText { "My Tab".into() }

    fn ui(&mut self, ui: &mut egui::Ui, control: &mut Vec<AppCommand>) {
        ui.label("Hello from My Tab!");
        ui.text_edit_multiline(&mut self.content);
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}
```

然后在菜单点击事件中打开它：
```rust
fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
    if ui.button("Open My Tab").clicked() {
        // AppCommand::OpenTab 用于打开新标签页
        control.push(AppCommand::OpenTab(crate::Tab::new(Box::new(MyTab { 
            content: String::new() 
        }))));
    }
}
```

### 3.2 响应打开文件请求
如果你的插件是一个查看器（如图片查看器），你应该实现 `try_open_file`。

```rust
fn try_open_file(&mut self, path: &std::path::Path) -> Option<Box<dyn TabInstance>> {
    let ext = path.extension()?.to_str()?;
    if ["png", "jpg", "jpeg"].contains(&ext) {
        // 返回一个能够显示该图片的 TabInstance
        Some(Box::new(ImageViewerTab::new(path)))
    } else {
        None
    }
}
```

### 3.3 请求打开文件
如果你的插件是一个文件浏览器，你可以发送指令请求系统打开文件，而无需关心谁来处理它。

```rust
if ui.button("Open Report.pdf").clicked() {
    control.push(AppCommand::OpenFile(PathBuf::from("Report.pdf")));
}
```

### 3.4 添加设置项
让用户配置你的插件。

```rust
fn on_settings_ui(&mut self, ui: &mut Ui) {
    ui.heading("My Tool Settings");
    ui.checkbox(&mut self.my_config_bool, "Enable Feature X");
}
```

---

## 4. 常见问题

**Q: 如何保存插件数据？**
A: 目前你需要自己在 `update` 或操作触发时写入文件。未来版本将提供统一的 `on_save/on_load` 接口。

**Q: 为什么修改了代码不生效？**
A: 确保在 `launcher` 中勾选了你的插件，并点击了 "Sync & Run"。如果是手动运行 `cargo run`，确保 `Cargo.toml` 中的 `[features]` 包含了你的插件。
