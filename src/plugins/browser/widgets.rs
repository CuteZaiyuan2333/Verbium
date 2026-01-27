use eframe::egui;

/// 自定义导航按钮小部件
pub struct NavButton {
    text: &'static str,
}

impl NavButton {
    pub fn new(text: &'static str) -> Self {
        Self { text }
    }
}

impl egui::Widget for NavButton {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::vec2(36.0, 36.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            
            // Draw circle background on hover/active
            if response.hovered() || response.clicked() || response.has_focus() {
                ui.painter().circle_filled(
                    rect.center(),
                    rect.width() / 2.0,
                    visuals.bg_fill,
                );
            }

            // Draw text centered
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.text,
                egui::FontId::proportional(20.0),
                visuals.text_color(),
            );
        }
        response
    }
}
