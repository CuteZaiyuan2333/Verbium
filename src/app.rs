use eframe::egui;
use egui_dock::{DockArea, DockState, Style, TabViewer};
use crate::{Tab, Plugin, AppCommand};
use crate::plugins;

// ----------------------------------------------------------------------------
// TabViewer 实现
// ----------------------------------------------------------------------------
struct VerbiumTabViewer<'a> {
    command_queue: &'a mut Vec<AppCommand>,
}

impl<'a> TabViewer for VerbiumTabViewer<'a> {
    type Tab = Tab;

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.id)
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.instance.title()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.push_id(tab.id, |ui| {
            tab.instance.ui(ui, self.command_queue);
        });
    }

    fn closeable(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }

    fn on_close(&mut self, _tab: &mut Self::Tab) -> bool {
        true
    }
}

// ----------------------------------------------------------------------------
// Main Application State
// ----------------------------------------------------------------------------
pub struct VerbiumApp {
    dock_state: DockState<Tab>,
    plugins: Vec<Box<dyn Plugin>>,
    command_queue: Vec<AppCommand>,
}

impl VerbiumApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let dock_state = DockState::new(Vec::new());
        // 使用自动化注册函数
        let plugins = plugins::all_plugins();

        let app = Self {
            dock_state,
            plugins,
            command_queue: Vec::new(),
        };
        app
    }

    fn process_commands(&mut self) {
        // 使用 while 循环处理，防止指令执行中产生新指令被遗漏
        let mut i = 0;
        while i < self.command_queue.len() {
            let cmd = &self.command_queue[i];
            match cmd {
                AppCommand::OpenTab(tab) => {
                    self.dock_state.main_surface_mut().push_to_focused_leaf(tab.clone());
                }
                AppCommand::TileAll => {
                    let mut all_tabs = Vec::new();
                    self.dock_state.retain_tabs(|tab| {
                        all_tabs.push(tab.clone());
                        true
                    });
                    if !all_tabs.is_empty() {
                        self.dock_state = DockState::new(all_tabs);
                    }
                }
                AppCommand::ResetLayout => {
                    self.dock_state = DockState::new(Vec::new());
                }
                AppCommand::CloseTab(title) => {
                    self.dock_state.retain_tabs(|tab| {
                        tab.instance.title().text() != title
                    });
                }
            }
            i += 1;
        }
        self.command_queue.clear();
    }
}

impl eframe::App for VerbiumApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. 插件逻辑更新
        for plugin in &mut self.plugins {
            plugin.update(&mut self.command_queue);
        }

        // 2. 顶部栏渲染
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // 标准 "File" 菜单
                ui.menu_button("File", |ui| {
                    for plugin in &mut self.plugins {
                        plugin.on_file_menu(ui, &mut self.command_queue);
                    }
                });

                // 标准 "Tab" 菜单
                ui.menu_button("Tab", |ui| {
                    for plugin in &mut self.plugins {
                        plugin.on_tab_menu(ui, &mut self.command_queue);
                    }
                });

                // 插件自定义的顶级菜单项
                for plugin in &mut self.plugins {
                    plugin.on_menu_bar(ui, &mut self.command_queue);
                }
            });
        });

        // 3. 全局 UI
        for plugin in &mut self.plugins {
            plugin.on_global_ui(ctx, &mut self.command_queue);
        }

        // 4. 处理指令
        self.process_commands();

        // 5. 中心 Dock 区域
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = VerbiumTabViewer {
                command_queue: &mut self.command_queue,
            };
            let style = Style::from_egui(ui.style().as_ref());

            DockArea::new(&mut self.dock_state)
                .style(style)
                .show_window_collapse_buttons(false) // 移除悬浮窗三角形收起按钮
                .show_window_close_buttons(false)
                .show_close_buttons(true)
                .show_inside(ui, &mut viewer);
        });
    }
}
