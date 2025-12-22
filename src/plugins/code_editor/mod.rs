use egui::{Ui, WidgetText, TextEdit, TextStyle};
use crate::{Tab, Plugin, AppCommand, TabInstance};

/// 代码编辑器标签页实例
#[derive(Debug, Clone)]
pub struct CodeEditorTab {
    title: String,
    content: String,
    #[allow(dead_code)]
    language: String,
}

impl Default for CodeEditorTab {
    fn default() -> Self {
        Self {
            title: "Code Editor".into(),
            content: r#"// Verbium Code Editor
fn main() {
    println!("Hello, Verbium!");
}
"#.into(),
            language: "rs".into(),
        }
    }
}

impl TabInstance for CodeEditorTab {
    fn title(&self) -> WidgetText { self.title.clone().into() }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        // 注意：这里使用了 egui 的 code_editor 样式，它提供了行号显示。
        // 为了实现语法高亮，通常需要集成 egui_extras 的 syntax_highlighting。
        // 在 Cargo.toml 中添加 egui_extras 后，可以启用以下代码。
        
        /*
        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                &theme,
                string,
                &self.language,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };
        */

        egui::ScrollArea::vertical().show(ui, |ui| {
            let edit = TextEdit::multiline(&mut self.content)
                .font(TextStyle::Monospace) // 使用等宽字体
                .code_editor()             // 启用代码编辑器模式（含行号）
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .desired_rows(25);

            // 如果启用了 syntax_highlighting，取消下面注释
            // let edit = edit.layouter(&mut layouter);

            ui.add(edit);
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

/// 代码编辑器插件
pub struct CodeEditorPlugin;

impl Plugin for CodeEditorPlugin {
    fn name(&self) -> &str {
        "code_editor"
    }

    fn dependencies(&self) -> Vec<String> {
        // 建议依赖核心插件
        vec!["core".to_string()]
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("Code Editor").clicked() {
            // 点击菜单项时，向应用发送打开新标签页的命令
            control.push(AppCommand::OpenTab(Tab::new(Box::new(CodeEditorTab::default()))));
            ui.close_menu();
        }
    }
}

/// 插件工厂函数
pub fn create() -> CodeEditorPlugin {
    CodeEditorPlugin
}
