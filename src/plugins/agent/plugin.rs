use egui::Ui;
use crate::{Plugin, AppCommand, Tab};
use super::models::{AgentConfig, ChatSession};
use super::tab::AgentTab;
use std::path::PathBuf;

pub struct AgentPlugin {
    config: AgentConfig,
    show_session_creator: bool,
    new_session_name: String,
}

impl AgentPlugin {
    pub fn new() -> Self {
        Self {
            config: AgentConfig::load(),
            show_session_creator: false,
            new_session_name: "New Chat".to_string(),
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

    fn create_and_open_session(&mut self, path: PathBuf, control: &mut Vec<AppCommand>) {
        if let Ok(session) = ChatSession::load(&path) {
             let modes = self.get_available_modes();
             control.push(AppCommand::OpenTab(Tab::new(Box::new(AgentTab::new(session, modes)))));
             self.show_session_creator = false;
        }
    }

    fn get_available_sessions(&self) -> Vec<PathBuf> {
        let mut sessions = Vec::new();
        let folder = self.config.default_chat_dir.clone().unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_default()
        });

        if let Ok(entries) = std::fs::read_dir(folder) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    // Check if it's not a config file (simple heuristic: if it contains session data)
                    // For now, let's just include all .toml except known configs
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if filename != "agent_config.toml" && filename != "launcher_config.toml" {
                            sessions.push(path);
                        }
                    }
                }
            }
        }
        sessions.sort_by(|a, b| b.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .cmp(&a.metadata().and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH)));
        sessions
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

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label("Default Chat Storage");
                ui.horizontal(|ui| {
                    let path_str = self.config.default_chat_dir.as_ref()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|| "No directory specified".into());
                    
                    ui.label(format!("Path: {}", path_str));
                    
                    if ui.button("Select...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.config.default_chat_dir = Some(path);
                            self.config.save();
                        }
                    }
                });
            });
        });
    }

    fn on_global_ui(&mut self, ctx: &egui::Context, control: &mut Vec<AppCommand>) {
        if self.show_session_creator {
            let mut open = true;
            egui::Window::new("Agent Session Manager")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_min_width(400.0);
                    ui.set_max_height(500.0);
                    
                    ui.heading("Agent Sessions");
                    ui.add_space(8.0);

                    // 1. New Session Area
                    ui.group(|ui| {
                        ui.label(egui::RichText::new("Create New Session").strong());
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_session_name);
                            if ui.button("🚀 Create").clicked() {
                                let folder = self.config.default_chat_dir.clone().unwrap_or_else(|| {
                                    std::env::current_dir().unwrap_or_default()
                                });
                                
                                let safe_name = self.new_session_name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_");
                                let filename = format!("{}.toml", safe_name);
                                let full_path = folder.join(filename);

                                let mode = self.get_available_modes().get(0).cloned().unwrap_or("Chat".into());
                                let mut session = ChatSession::new(mode, "Gemini Pro".into());
                                session.path = Some(full_path.clone());

                                if let Err(e) = session.save() {
                                     control.push(AppCommand::Notify { 
                                         message: format!("Failed to create session: {}", e), 
                                         level: crate::NotificationLevel::Error 
                                     });
                                } else {
                                    self.create_and_open_session(full_path, control);
                                }
                            }
                        });
                    });

                    ui.add_space(12.0);

                    // 2. Existing Sessions List
                    ui.label(egui::RichText::new("Open Existing Session").strong());
                    ui.add_space(4.0);
                    
                    let sessions = self.get_available_sessions();
                    if sessions.is_empty() {
                        ui.weak("No sessions found in storage directory.");
                    } else {
                        egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                            for path in sessions {
                                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("Unknown");
                                let date_str = path.metadata().and_then(|m| m.modified()).ok()
                                    .map(|t| {
                                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                                        datetime.format("%Y-%m-%d %H:%M").to_string()
                                    }).unwrap_or_default();

                                ui.horizontal(|ui| {
                                    if ui.button(format!("💬 {}", filename)).clicked() {
                                        self.create_and_open_session(path.clone(), control);
                                    }
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.weak(date_str);
                                    });
                                });
                                ui.separator();
                            }
                        });
                    }

                    ui.add_space(8.0);
                    if ui.button("📂 Browse Files...").clicked() {
                         if let Some(path) = rfd::FileDialog::new().add_filter("TOML", &["toml"]).pick_file() {
                             self.create_and_open_session(path, control);
                         }
                    }
                });
            
            self.show_session_creator = open;
        }
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        if ui.button("Agent Tab").clicked() {
            self.show_session_creator = true;
            ui.close_menu();
        }
    }
}

pub fn create() -> AgentPlugin {
    AgentPlugin::new()
}
