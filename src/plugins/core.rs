use egui::{Ui, WidgetText};
use crate::{Tab, Plugin, AppCommand, TabInstance};

// ----------------------------------------------------------------------------
// Core Tabs
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EmptyTab;
impl TabInstance for EmptyTab {
    fn title(&self) -> WidgetText { "Empty".into() }
    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        ui.centered_and_justified(|ui| { 
            ui.label("Verbium Layout Engine\nDrag tabs to split the screen."); 
        });
    }
    fn box_clone(&self) -> Box<dyn TabInstance> { Box::new(self.clone()) }
}

#[derive(Debug, Clone)]
pub struct EditorTab {
    pub name: String,
    pub content: String,
}
impl TabInstance for EditorTab {
    fn title(&self) -> WidgetText { format!("📝 {}", self.name).into() }
    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        ui.text_edit_multiline(&mut self.content);
    }
    fn box_clone(&self) -> Box<dyn TabInstance> { Box::new(self.clone()) }
}

// ----------------------------------------------------------------------------
// Core Plugin
// ----------------------------------------------------------------------------

pub struct CorePlugin {
    new_file_counter: usize,
    show_about: bool,
}

impl Default for CorePlugin {
    fn default() -> Self {
        Self { 
            new_file_counter: 1,
            show_about: false,
        }
    }
}

impl Plugin for CorePlugin {
    fn name(&self) -> &str { "core" }

    // Core 不依赖任何东西
    fn dependencies(&self) -> Vec<String> { Vec::new() }

    fn on_file_menu(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        if ui.button("Quit").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("New Editor").clicked() {
            let name = format!("Untitled-{}", self.new_file_counter);
            self.new_file_counter += 1;
            control.push(AppCommand::OpenTab(Tab::new(Box::new(EditorTab {
                name,
                content: String::new(),
            }))));
            ui.close_menu();
        }
        if ui.button("New Empty Tab").clicked() {
            control.push(AppCommand::OpenTab(Tab::new(Box::new(EmptyTab))));
            ui.close_menu();
        }
        ui.separator();
        if ui.button("Tile All").clicked() {
            control.push(AppCommand::TileAll);
            ui.close_menu();
        }
        if ui.button("Reset Layout").clicked() {
            control.push(AppCommand::ResetLayout);
            ui.close_menu();
        }
    }

    fn on_menu_bar(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        if ui.button("About").clicked() {
            self.show_about = true;
        }
    }

    fn on_global_ui(&mut self, ctx: &egui::Context, _control: &mut Vec<AppCommand>) {
        egui::Window::new("About Verbium")
            .open(&mut self.show_about)
            .show(ctx, |ui| {
                ui.heading("Verbium");
                ui.label("A plugin-based extensible editor framework.");
                ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
            });
    }
}
