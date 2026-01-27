use std::sync::Arc;
use parking_lot::Mutex;
use eframe::egui;
use crate::{TabInstance, AppCommand};
use super::widgets::NavButton;
use super::webview::{create_webview, steal_focus_from_webview};

/// Wrapper to make WebView Send + Sync
/// In Windows, WebView2 is thread-safe for the same thread or with proper synchronization.
pub struct SafeWebView(pub wry::WebView);
unsafe impl Send for SafeWebView {}
unsafe impl Sync for SafeWebView {}

#[derive(Clone)]
pub struct BrowserTab {
    url: String,
    webview: Arc<Mutex<Option<SafeWebView>>>,
    last_rect: Arc<Mutex<egui::Rect>>,
    last_ppp: Arc<Mutex<f32>>,
}

impl std::fmt::Debug for BrowserTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserTab").field("url", &self.url).finish()
    }
}

impl BrowserTab {
    pub fn new(url: String) -> Self {
        Self {
            url,
            webview: Arc::new(Mutex::new(None)),
            last_rect: Arc::new(Mutex::new(egui::Rect::NOTHING)),
            last_ppp: Arc::new(Mutex::new(0.0)),
        }
    }
}

impl TabInstance for BrowserTab {
    fn title(&self) -> egui::WidgetText {
        "Browser".into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _control: &mut Vec<AppCommand>) {
        // 1. Top Bar
        ui.horizontal(|ui| {
            if ui.add(NavButton::new("⬅")).clicked() {
                if let Some(safe_webview) = self.webview.lock().as_ref() {
                    let _ = safe_webview.0.evaluate_script("history.back()");
                }
            }
            if ui.add(NavButton::new("➡")).clicked() {
                if let Some(safe_webview) = self.webview.lock().as_ref() {
                    let _ = safe_webview.0.evaluate_script("history.forward()");
                }
            }
            if ui.add(NavButton::new("🔄")).clicked() {
                if let Some(safe_webview) = self.webview.lock().as_ref() {
                    let _ = safe_webview.0.reload();
                }
            }
            
            ui.add_space(8.0);

            let address_bar_frame = egui::Frame::group(ui.style())
                .fill(ui.visuals().extreme_bg_color)
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .rounding(15.0)
                .inner_margin(egui::Margin::symmetric(10.0, 5.0));

            address_bar_frame.show(ui, |ui| {
                let text_edit = egui::TextEdit::singleline(&mut self.url)
                    .frame(false)
                    .desired_width(ui.available_width());
                    
                let response = ui.add(text_edit);
                
                if response.clicked() || response.has_focus() {
                    steal_focus_from_webview();
                    
                    if response.has_focus() {
                         ui.painter().rect_stroke(
                            response.rect.expand(2.0),
                            15.0,
                            egui::Stroke::new(2.0, ui.visuals().selection.bg_fill),
                        );
                    }
                }

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(safe_webview) = self.webview.lock().as_ref() {
                        let url = if self.url.contains("://") {
                            self.url.clone()
                        } else {
                            format!("https://{}", self.url)
                        };
                        let _ = safe_webview.0.load_url(&url);
                    }
                }
            });
        });

        // 2. WebView Area
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                let rect = ui.available_rect_before_wrap();
                let ppp = ui.ctx().pixels_per_point();

                let mut webview_lock = self.webview.lock();
                if webview_lock.is_none() {
                    if let Some(webview) = create_webview(&self.url) {
                        *webview_lock = Some(SafeWebView(webview));
                    }
                }

                if let Some(safe_webview) = webview_lock.as_ref() {
                    let mut last_rect = self.last_rect.lock();
                    let mut last_ppp = self.last_ppp.lock();

                    if rect != *last_rect || ppp != *last_ppp {
                        *last_rect = rect;
                        *last_ppp = ppp;

                        // Convert to physical pixels
                        let physical_rect = egui::Rect::from_min_max(
                            egui::pos2(rect.min.x * ppp, rect.min.y * ppp),
                            egui::pos2(rect.max.x * ppp, rect.max.y * ppp),
                        );
                        
                        let _ = safe_webview.0.set_bounds(wry::Rect {
                            position: wry::dpi::PhysicalPosition::new(
                                physical_rect.min.x as i32,
                                physical_rect.min.y as i32
                            ).into(),
                            size: wry::dpi::PhysicalSize::new(
                                physical_rect.width() as u32,
                                physical_rect.height() as u32
                            ).into(),
                        });
                    }
                    
                    let _ = safe_webview.0.set_visible(true);
                }

                ui.centered_and_justified(|ui| {
                    ui.heading("Loading WebView...");
                });
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}