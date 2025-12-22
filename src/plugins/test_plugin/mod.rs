use egui::{Ui, WidgetText};
use crate::{Tab, Plugin, AppCommand, TabInstance};

#[derive(Debug, Clone)]
pub struct TestTab {
    text: String,
}

impl TabInstance for TestTab {
    fn title(&self) -> WidgetText { "TESTTAB".into() }
    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        ui.label("This is a Test Tab from plugin.");
        ui.text_edit_multiline(&mut self.text);
    }
    fn box_clone(&self) -> Box<dyn TabInstance> { Box::new(self.clone()) }
}

pub struct TestPlugin;

impl Plugin for TestPlugin {
    fn name(&self) -> &str { "test_plugin" }

    fn dependencies(&self) -> Vec<String> {
        vec!["core".to_string()]
    }

    fn on_file_menu(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        if ui.button("TEST").clicked() {
            println!("TEST menu item clicked!");
            ui.close_menu();
        }
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("TESTTAB").clicked() {
            control.push(AppCommand::OpenTab(Tab::new(Box::new(TestTab {
                text: "Hello from plugin!".into(),
            }))));
            ui.close_menu();
        }
    }
}

pub fn create() -> TestPlugin {
    TestPlugin
}