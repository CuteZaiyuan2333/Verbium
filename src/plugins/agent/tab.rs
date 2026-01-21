use egui::{Ui, WidgetText};
use crate::{AppCommand, TabInstance};
use super::models::{ChatMessage, MessageRole};

#[derive(Debug, Clone)]
pub struct AgentTab {
    messages: Vec<ChatMessage>,
    input_text: String,
    selected_mode: String,
    available_modes: Vec<String>,
    // We keep `input_height` to track the user's preferred height for the input area
    input_height: f32,
}

impl AgentTab {
    pub fn new(available_modes: Vec<String>) -> Self {
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
            input_height: 80.0,
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
        "Agent".into()
    }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        // Use a Bottom-Up layout approach to pin the input area to the bottom.
        // This avoids manual height calculations (like `total_h - input_h`).
        
        // 1. Bottom: Input Area
        // We use `allocate_ui_with_layout` to reserve space at the bottom.
        let available_height = ui.available_height();
        let input_h = self.input_height.clamp(40.0, available_height * 0.5);

        // Define a region for the input + separator at the bottom
        egui::TopBottomPanel::bottom("agent_input_panel_internal") 
            .resizable(true)
            .min_height(40.0)
            .default_height(80.0)
            .show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    // Input Text
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        let btn_size = 32.0;
                         // Main Text Edit
                        let available_w = ui.available_width();
                        let text_edit_w = (available_w - btn_size - 12.0).max(0.0);
                        
                        egui::ScrollArea::vertical()
                            .id_salt("input_scroll")
                            .max_height(ui.available_height() - 8.0) // Leave some padding
                            .show(ui, |ui| {
                                ui.add_sized(
                                    [text_edit_w, ui.available_height()], 
                                    egui::TextEdit::multiline(&mut self.input_text)
                                        .hint_text("Type a message...")
                                        .desired_width(text_edit_w)
                                        .lock_focus(true)
                                );
                            });

                        // Send Button
                        if ui.add(egui::Button::new("🚀").min_size(egui::vec2(btn_size, btn_size))).clicked() 
                           || (ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift)) {
                            self.send_message();
                        }
                    });
                     ui.add_space(4.0);
                });
            });

        // 2. The Rest: Header + Chat
        // Whatever is left in the `ui` (which is now the top part) is used here.
        ui.vertical(|ui| {
             // Header
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(rect.center(), 5.0, egui::Color32::from_rgb(96, 165, 250));
                ui.add_space(4.0);
                ui.strong("AI Agent");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
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
            ui.separator();

            // Chat Area
            // Automatically fills the remaining space
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    let inner_w = ui.available_width() - 8.0; 
                    for msg in &self.messages {
                        render_message(ui, msg, inner_w);
                    }
                    ui.add_space(8.0);
                });
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

fn render_message(ui: &mut Ui, msg: &ChatMessage, max_width: f32) {
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
        let max_bubble_w = max_width * 0.85;
        egui::Frame::none()
            .fill(fill_color)
            .stroke(stroke_color)
            .rounding(8.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.set_max_width(max_bubble_w);
                ui.label(egui::RichText::new(&msg.content).color(label_color));
            });
    });
    ui.add_space(8.0);
}
