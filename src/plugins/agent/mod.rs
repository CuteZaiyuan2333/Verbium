use std::path::PathBuf;
use egui::{Ui, WidgetText};
use serde::{Deserialize, Serialize};
use crate::{Plugin, AppCommand, TabInstance, Tab};

// ----------------------------------------------------------------------------
// 配置文件模型
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct AgentConfig {
    script_directory: Option<PathBuf>,
}

impl AgentConfig {
    fn load() -> Self {
        let path = std::path::Path::new("agent_config.toml");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                return toml::from_str(&content).unwrap_or_default();
            }
        }
        Self::default()
    }

    fn save(&self) {
        let path = std::path::Path::new("agent_config.toml");
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }
}

// ----------------------------------------------------------------------------
// Tab 实例实现
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentTab {
    // 未来将包含聊天记录、选中的模式等状态
}

impl AgentTab {
    fn new() -> Self {
        Self {}
    }
}

impl TabInstance for AgentTab {
    fn title(&self) -> WidgetText {
        "🤖 Agent".into()
    }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        ui.centered_and_justified(|ui| {
            ui.heading("[Place Holder]");
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

// ----------------------------------------------------------------------------
// 插件接口实现
// ----------------------------------------------------------------------------

pub struct AgentPlugin {
    config: AgentConfig,
}

impl AgentPlugin {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::load(),
        }
    }
}

impl Plugin for AgentPlugin {
    fn name(&self) -> &str {
        // build.rs 会生成 PLUGIN_NAME_AGENT 常量
        crate::plugins::PLUGIN_NAME_AGENT
    }

    fn on_settings_ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.label("Agent Script Directory Configuration");
            ui.horizontal(|ui| {
                let path_str = self.config.script_directory.as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "No directory specified".into());
                
                ui.label(format!("Current: {}", path_str));
                
                if ui.button("Select Directory...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.config.script_directory = Some(path);
                        self.config.save();
                    }
                }
            });
            ui.add_space(4.0);
            ui.weak("Each .rhai file in this directory will be loaded as an independent Agent mode.");
        });
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("🤖 Agent Tab").clicked() {
            control.push(AppCommand::OpenTab(Tab::new(Box::new(AgentTab::new()))));
            ui.close_menu();
        }
    }
}

pub fn create() -> AgentPlugin {
    AgentPlugin::new()
}
