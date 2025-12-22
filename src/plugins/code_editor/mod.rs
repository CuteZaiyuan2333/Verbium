use egui::{Ui, WidgetText};
use crate::{Tab, Plugin, AppCommand, TabInstance};

#[derive(Debug, Clone)]
pub struct CodeEditorTab {
    pub name: String,
    pub code: String,
    pub language: String,
}

impl TabInstance for CodeEditorTab {
    fn title(&self) -> WidgetText {
        format!(" {}", self.name).into()
    }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style()),
                string,
                &self.language,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.vertical(|ui| {
            egui::ScrollArea::both()
                .id_salt("code_editor_scroll") // 使用 id_salt 消除 warning
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        // 1. 简单的行号显示器
                        let line_count = self.code.lines().count().max(1);
                        let mut line_numbers = String::new();
                        for i in 1..=line_count {
                            line_numbers.push_str(&format!("{}\n", i));
                        }
                        
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(line_numbers)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(egui::Color32::from_gray(100))
                            )
                        );

                        ui.separator();

                        // 2. 编辑器主体
                        let editor = egui::TextEdit::multiline(&mut self.code)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .lock_focus(true)
                            .desired_width(f32::INFINITY)
                            .layouter(&mut layouter);

                        ui.add_sized(ui.available_size(), editor);
                    });
                });
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

pub struct CodeEditorPlugin;

impl Plugin for CodeEditorPlugin {
    fn name(&self) -> &str { "code_editor" }

    fn dependencies(&self) -> Vec<String> {
        vec!["core".to_string()]
    }

    fn try_open_file(&mut self, path: &std::path::Path) -> Option<Box<dyn TabInstance>> {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        // 简单的白名单检查
        if ["rs", "toml", "txt", "md", "json", "js", "html", "css", "py", "c", "cpp"].contains(&ext) {
             if let Ok(content) = std::fs::read_to_string(path) {
                 return Some(Box::new(CodeEditorTab {
                     name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                     code: content,
                     language: ext.to_string(),
                 }));
             }
        }
        None
    }

    fn on_settings_ui(&mut self, ui: &mut Ui) {
        ui.label("Editor Settings Placeholder");
        ui.label("Here you could configure font size, theme, etc.");
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("Code Editor").clicked() {
            control.push(AppCommand::OpenTab(Tab::new(Box::new(CodeEditorTab {
                name: "unsaved".into(),      // 默认显示 unsaved
                code: String::new(),         // 默认内容为空
                language: "rs".into(),
            }))));
            ui.close_menu();
        }
    }
}

pub fn create() -> CodeEditorPlugin {
    CodeEditorPlugin
}
