use egui::{Ui, WidgetText};
use crate::{AppCommand, TabInstance};
use super::models::{ChatSession, ChatMessage, MessageRole};

#[derive(Debug, Clone, Default)]
struct InputState {
    text: String,
    // Future: attachments, focus state, etc.
}

#[derive(Debug, Clone)]
pub struct AgentTab {
    session: ChatSession,
    input: InputState,
    available_modes: Vec<String>,
    available_models: Vec<String>,
}

impl AgentTab {
    pub fn new(session: ChatSession, available_modes: Vec<String>) -> Self {
        Self {
            session,
            input: InputState::default(),
            available_modes,
            available_models: vec!["Gemini Pro".into(), "GPT-4".into(), "Local Llama".into()],
        }
    }

    fn send_message(&mut self) {
        let text = self.input.text.trim().to_string();
        if text.is_empty() {
            return;
        }

        // Add user message to session
        self.session.messages.push(ChatMessage {
            role: MessageRole::User,
            content: text.clone(),
        });

        // Mock response (Phase 1)
        let mode = self.session.context_mode.clone();
        let model = self.session.model_name.clone();
        
        self.session.messages.push(ChatMessage {
            role: MessageRole::Agent,
            content: format!("(Mock Response in [{}] mode using [{}])\nReceived: {}", mode, model, text),
        });

        // Auto-save logic
        if let Err(e) = self.session.save() {
            eprintln!("Failed to save session: {}", e);
        }

        self.input.text.clear();
    }
}



impl TabInstance for AgentTab {
    fn title(&self) -> WidgetText {
        "Agent".into()
    }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        // 1. Input Area (Bottom) with Top Row Controls
        let mut sent_text = None;

        egui::TopBottomPanel::bottom(ui.make_persistent_id("agent_modern_input"))
            .frame(egui::Frame::none().inner_margin(12.0))
            .show_inside(ui, |ui| {
                // The "Card" container
                let card_rounding = 12.0;
                let card_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                let card_bg = ui.visuals().extreme_bg_color; // Slightly darker/contrast

                egui::Frame::group(ui.style())
                    .fill(card_bg)
                    .stroke(card_stroke)
                    .rounding(card_rounding)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_min_height(80.0);
                        
                        // A. Header Row: Mode & Model Selectors (Integrated)
                        ui.horizontal(|ui| {
                            ui.visuals_mut().widgets.inactive.rounding = egui::Rounding::same(4.0);
                            
                            // Mode Selector
                            ui.label("Mode:");
                            egui::ComboBox::from_id_salt("mode_select_input")
                                .selected_text(&self.session.context_mode)
                                .show_ui(ui, |ui| {
                                    for mode in &self.available_modes {
                                        ui.selectable_value(&mut self.session.context_mode, mode.clone(), mode);
                                    }
                                });

                            ui.add_space(8.0);

                            // Model Selector
                            ui.label("Model:");
                            egui::ComboBox::from_id_salt("model_select_input")
                                .selected_text(&self.session.model_name)
                                .show_ui(ui, |ui| {
                                    for model in &self.available_models {
                                        ui.selectable_value(&mut self.session.model_name, model.clone(), model);
                                    }
                                });

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Context attachment indicator (Mock)
                                ui.label(egui::RichText::new("No Context").size(10.0).weak());
                            });
                        });
                        
                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // B. Input Field (Frameless)
                        let text_area = egui::TextEdit::multiline(&mut self.input.text)
                            .frame(false)
                            .hint_text("Ask me anything...")
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .lock_focus(true);
                        
                        let response = ui.add(text_area);

                        // C. Action Bar (Bottom Right)
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                             if ui.button("📎").on_hover_text("Attach File").clicked() { /* TODO */ }
                             
                             ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let send_btn = egui::Button::new("  🚀 Send  ").rounding(8.0);
                                if ui.add(send_btn).clicked() {
                                    sent_text = Some(self.input.text.clone());
                                }
                                if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.command) {
                                    sent_text = Some(self.input.text.clone());
                                }
                            });
                        });
                    });
            });

        // Handle sending
        if let Some(_) = sent_text {
            self.send_message();
        }

        // 2. Chat Area (Fill Rest)
        ui.vertical(|ui| {
             // Simple Top Header (Title Only)
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.heading("Agent");
            });
            ui.separator();

            // Chat Scroll
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    let inner_w = ui.available_width() - 16.0; 
                    for msg in &self.session.messages {
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
