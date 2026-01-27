use std::sync::Arc;
use std::sync::mpsc::{channel, Receiver, Sender};
use parking_lot::Mutex;
use egui::Ui;
use crate::{Plugin, AppCommand, Tab};

pub mod tab;
pub mod webview;
pub mod widgets;

pub struct BrowserPlugin {
    new_tab_tx: Arc<Sender<String>>,
    new_tab_rx: Receiver<String>,
}

impl BrowserPlugin {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            new_tab_tx: Arc::new(tx),
            new_tab_rx: rx,
        }
    }
}

impl Plugin for BrowserPlugin {
    fn name(&self) -> &str {
        crate::plugins::PLUGIN_NAME_BROWSER
    }

    fn update(&mut self, control: &mut Vec<AppCommand>) {
        // 在每帧开始时处理新标签页请求，确保指令在同一帧被 process_commands 处理
        while let Ok(url) = self.new_tab_rx.try_recv() {
            let tab = tab::BrowserTab::new(url, self.new_tab_tx.clone());
            control.push(AppCommand::OpenTab(Tab::new(Box::new(tab))));
        }
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("🌐 New Browser").clicked() {
            let tab = tab::BrowserTab::new("https://www.google.com".to_string(), self.new_tab_tx.clone());
            control.push(AppCommand::OpenTab(Tab::new(Box::new(tab))));
            ui.close_menu();
        }
    }
}

pub fn create() -> BrowserPlugin {
    BrowserPlugin::new()
}