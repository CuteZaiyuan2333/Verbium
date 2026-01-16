use std::path::PathBuf;
use egui::{Ui, WidgetText};
use serde::{Deserialize, Serialize};
use crate::{Plugin, AppCommand, TabInstance, Tab};

// ----------------------------------------------------------------------------
// Data Models
// ----------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct AgentConfig {
    script_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
enum MessageRole {
    User,
    Agent,
}

#[derive(Debug, Clone)]
struct ChatMessage {
    role: MessageRole,
    content: String,
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
// Tab Instance
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AgentTab {
    messages: Vec<ChatMessage>,
    input_text: String,
    selected_mode: String,
    available_modes: Vec<String>,
    input_height: f32,
}

impl AgentTab {
    fn new(available_modes: Vec<String>) -> Self {
        let selected_mode = available_modes.get(0).cloned().unwrap_or_else(|| "No Mode".to_string());
        Self {
            messages: vec![
                ChatMessage {
                    role: MessageRole::Agent,
                    content: "Hello! I am your AI assistant. Select a mode and start chatting.".to_string(),
                }
            ],
            input_text: String::new(),
            selected_mode,
            available_modes,
            input_height: 80.0, // 初始高度
        }
    }

    fn send_message(&mut self) {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return;
        }

        // Add user message
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
        });

        // Mock response (Phase 1)
        let mode = self.selected_mode.clone();
        self.messages.push(ChatMessage {
            role: MessageRole::Agent,
            content: format!("(Mock Response in [{}] mode)\nReceived: {}", mode, text),
        });

        self.input_text.clear();
    }
}

impl TabInstance for AgentTab {
    fn title(&self) -> WidgetText {
        "🤖 Agent".into()
    }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        ui.vertical(|ui| {
            // 1. Top Header
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, egui::Color32::from_rgb(96, 165, 250));
                ui.strong("AI Agent");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    egui::ComboBox::from_id_salt("mode_select")
                        .selected_text(&self.selected_mode)
                        .show_ui(ui, |ui| {
                            for mode in &self.available_modes {
                                ui.selectable_value(&mut self.selected_mode, mode.clone(), mode);
                            }
                        });
                    ui.label("Mode:");
                });
            });
            ui.add_space(4.0);
            ui.separator();

            // 2. Middle Chat Area (占据剩余空间减去底部输入框高度)
            let spacing = ui.spacing().item_spacing.y;
            let current_input_height = self.input_height.clamp(40.0, ui.available_height() * 0.7);
            let chat_area_height = ui.available_height() - current_input_height - spacing * 2.0;

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .max_height(chat_area_height)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    for msg in &self.messages {
                        let (align, fill_color, stroke_color, label_color) = match msg.role {
                            MessageRole::User => (
                                egui::Align::RIGHT,
                                ui.visuals().selection.bg_fill.gamma_multiply(0.2),
                                egui::Stroke::new(1.0, ui.visuals().selection.bg_fill.gamma_multiply(0.5)),
                                ui.visuals().strong_text_color(),
                            ),
                            MessageRole::Agent => (
                                egui::Align::LEFT,
                                ui.visuals().widgets.active.bg_fill.gamma_multiply(0.1),
                                egui::Stroke::new(1.0, ui.visuals().widgets.active.bg_fill.gamma_multiply(0.3)),
                                ui.visuals().text_color(),
                            ),
                        };

                        ui.with_layout(egui::Layout::top_down(align), |ui| {
                            let max_width = ui.available_width() * 0.8;
                            egui::Frame::none()
                                .fill(fill_color)
                                .stroke(stroke_color)
                                .rounding(8.0)
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    ui.set_max_width(max_width);
                                    ui.label(egui::RichText::new(&msg.content).color(label_color));
                                });
                        });
                        ui.add_space(8.0);
                    }
                });

            // 3. Draggable Separator
            let sep_response = ui.add(egui::Separator::default().horizontal().spacing(0.0));
            let sep_response = ui.interact(sep_response.rect.expand(2.0), ui.id().with("h_sep"), egui::Sense::drag());
            if sep_response.dragged() {
                self.input_height -= sep_response.drag_delta().y;
            }
            if sep_response.hovered() || sep_response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }

            // 4. Bottom Input Area
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                let btn_size = 36.0;
                let spacing_x = ui.spacing().item_spacing.x;
                let text_edit_width = ui.available_width() - btn_size - spacing_x - 4.0;
                let text_edit_height = self.input_height.clamp(40.0, 300.0);

                let text_edit = egui::TextEdit::multiline(&mut self.input_text)
                    .hint_text("Type a message...")
                    .desired_rows(1)
                    .lock_focus(true);
                
                let output = ui.add_sized([text_edit_width, text_edit_height], text_edit);
                
                // 正方形图标按钮
                let send_btn = egui::Button::new("🚀").min_size(egui::vec2(btn_size, btn_size));
                if ui.add(send_btn).clicked() 
                   || (output.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)) {
                    self.send_message();
                    output.request_focus();
                }
                ui.add_space(4.0);
            });
            ui.add_space(4.0);
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

// ----------------------------------------------------------------------------
// Plugin Implementation
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
        if ui.button("🤖 Agent Tab").clicked() {
            let modes = self.get_available_modes();
            control.push(AppCommand::OpenTab(Tab::new(Box::new(AgentTab::new(modes)))));
            ui.close_menu();
        }
    }
}

pub fn create() -> AgentPlugin {
    AgentPlugin::new()
}
