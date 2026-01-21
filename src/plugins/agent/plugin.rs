use egui::Ui;
use crate::{Plugin, AppCommand, Tab};
use super::models::AgentConfig;
use super::tab::AgentTab;

pub struct AgentPlugin {
    config: AgentConfig,
}

impl AgentPlugin {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::load(),
        }
    }

    fn get_available_modes(&self) -> Vec<String> {
        let mut modes = vec!["Chat".to_string(), "Plan".to_string(), "Solo".to_string()];
        
        if let Some(dir) = &self.config.script_directory {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            let mode_name = name.to_string();
                            if !modes.contains(&mode_name) {
                                modes.push(mode_name);
                            }
                        }
                    }
                }
            }
        }
        modes.sort();
        modes
    }
}

impl Plugin for AgentPlugin {
    fn name(&self) -> &str {
        crate::plugins::PLUGIN_NAME_AGENT
    }

    fn on_settings_ui(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Agent Settings");
            ui.add_space(4.0);
            
            ui.group(|ui| {
                ui.label("Script Directory Configuration");
                ui.horizontal(|ui| {
                    let path_str = self.config.script_directory.as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "No directory specified".into());
                    
                    ui.label(format!("Path: {}", path_str));
                    
                    if ui.button("Select...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.config.script_directory = Some(path);
                            self.config.save();
                        }
                    }
                });
                ui.add_space(4.0);
                ui.weak("Each .rhai file here becomes a selectable Agent mode.");
            });
        });
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("Agent Tab").clicked() {
            let modes = self.get_available_modes();
            // Use Box::new logic as before
            control.push(AppCommand::OpenTab(Tab::new(Box::new(AgentTab::new(modes)))));
            ui.close_menu();
        }
    }
}

pub fn create() -> AgentPlugin {
    AgentPlugin::new()
}
