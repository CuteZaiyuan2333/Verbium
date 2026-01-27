use egui::Ui;
use crate::{Plugin, AppCommand, Tab};
use self::tab::BrowserTab;

pub mod tab;
pub mod webview;
pub mod widgets;

pub struct BrowserPlugin;

impl BrowserPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Plugin for BrowserPlugin {
    fn name(&self) -> &str {
        crate::plugins::PLUGIN_NAME_BROWSER
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("🌐 New Browser").clicked() {
            let tab = BrowserTab::new("https://www.google.com".to_string());
            control.push(AppCommand::OpenTab(Tab::new(Box::new(tab))));
            ui.close_menu();
        }
    }
}

pub fn create() -> BrowserPlugin {
    BrowserPlugin::new()
}
