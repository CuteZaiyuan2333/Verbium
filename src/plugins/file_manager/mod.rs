use std::path::PathBuf;
use std::collections::HashSet;
use egui::{Ui, WidgetText, CollapsingHeader};
use crate::{Plugin, AppCommand, TabInstance, Tab};

// ----------------------------------------------------------------------------
// Tab Instance
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FileExplorerTab {
    root_path: Option<PathBuf>,
    expanded_nodes: HashSet<PathBuf>,
    rename_path: Option<PathBuf>,
    new_item_parent: Option<(PathBuf, bool)>, // (parent_path, is_dir)
    input_text: String,
}

impl FileExplorerTab {
    fn new() -> Self {
        Self {
            root_path: None,
            expanded_nodes: HashSet::new(),
            rename_path: None,
            new_item_parent: None,
            input_text: String::new(),
        }
    }

    fn render_tree(&mut self, ui: &mut Ui, path: PathBuf, control: &mut Vec<AppCommand>) {
        let name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "/".to_string());

        if path.is_dir() {
            let is_expanded = self.expanded_nodes.contains(&path);
            
            let header = CollapsingHeader::new(format!("📁 {}", name))
                .id_salt(&path)
                .open(Some(is_expanded));

            let response = header.show(ui, |ui| {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).collect();
                    // Directories first, then sort by name
                    paths.sort_by(|a, b| {
                        let a_is_dir = a.is_dir();
                        let b_is_dir = b.is_dir();
                        if a_is_dir != b_is_dir {
                            b_is_dir.cmp(&a_is_dir)
                        } else {
                            a.cmp(b)
                        }
                    });

                    for child_path in paths {
                        self.render_tree(ui, child_path, control);
                    }
                }
            });

            let header_response = response.header_response;

            header_response.context_menu(|ui| {
                if ui.button("New File").clicked() {
                    self.new_item_parent = Some((path.clone(), false));
                    self.input_text = "new_file.txt".to_string();
                    ui.close_menu();
                }
                if ui.button("New Folder").clicked() {
                    self.new_item_parent = Some((path.clone(), true));
                    self.input_text = "new_folder".to_string();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Rename").clicked() {
                    self.rename_path = Some(path.clone());
                    self.input_text = name.clone();
                    ui.close_menu();
                }
                if ui.button("Reveal in Explorer").clicked() {
                    reveal_in_explorer(&path);
                    ui.close_menu();
                }
                if ui.button("Copy Path").clicked() {
                    ui.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Delete").clicked() {
                    if let Ok(_) = std::fs::remove_dir_all(&path) {
                        self.expanded_nodes.remove(&path);
                    }
                    ui.close_menu();
                }
            });

            if header_response.clicked() {
                if is_expanded {
                    self.expanded_nodes.remove(&path);
                } else {
                    self.expanded_nodes.insert(path.clone());
                }
            }
        } else {
            // File display
            ui.horizontal(|ui| {
                ui.add_space(16.0); // Indentation
                let response = ui.selectable_label(false, format!("📄 {}", name));
                
                if response.double_clicked() {
                    control.push(AppCommand::OpenFile(path.clone()));
                }

                response.context_menu(|ui| {
                    if ui.button("Open").clicked() {
                        control.push(AppCommand::OpenFile(path.clone()));
                        ui.close_menu();
                    }
                    if ui.button("Rename").clicked() {
                        self.rename_path = Some(path.clone());
                        self.input_text = name.clone();
                        ui.close_menu();
                    }
                    if ui.button("Reveal in Explorer").clicked() {
                        reveal_in_explorer(&path);
                        ui.close_menu();
                    }
                    if ui.button("Copy Path").clicked() {
                        ui.output_mut(|o| o.copied_text = path.to_string_lossy().to_string());
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Delete").clicked() {
                        let _ = std::fs::remove_file(&path);
                        ui.close_menu();
                    }
                });
            });
        }
    }
}

fn reveal_in_explorer(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let path_str = path.to_string_lossy().to_string();
        if path.is_file() {
            let _ = Command::new("explorer")
                .arg("/select,")
                .arg(path_str)
                .spawn();
        } else {
            let _ = Command::new("explorer")
                .arg(path_str)
                .spawn();
        }
    }
}

impl TabInstance for FileExplorerTab {
    fn title(&self) -> WidgetText {
        if let Some(path) = &self.root_path {
            format!("📂 {}", path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()).into()
        } else {
            "📁 Explorer".into()
        }
    }

    fn ui(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        ui.vertical(|ui| {
            // Toolbar
            ui.horizontal(|ui| {
                if ui.button("Open Folder...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.root_path = Some(path);
                    }
                }
                if self.root_path.is_some() {
                    if ui.button("Refresh").clicked() {
                        self.expanded_nodes.retain(|p| p.exists());
                    }
                    if ui.button("Close").clicked() {
                        self.root_path = None;
                        self.expanded_nodes.clear();
                    }
                }
            });

            ui.separator();

            // Content
            if let Some(root) = self.root_path.clone() {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.render_tree(ui, root, control);
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No directory selected.\nClick the button above to start exploring.");
                });
            }
        });

        // Dialogs
        if let Some(path) = self.rename_path.clone() {
            let mut open = true;
            egui::Window::new("Rename")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Old name: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                    ui.text_edit_singleline(&mut self.input_text);
                    ui.horizontal(|ui| {
                        if ui.button("Rename").clicked() {
                            let new_path = path.parent().unwrap().join(&self.input_text);
                            if let Ok(_) = std::fs::rename(&path, new_path) {
                                self.rename_path = None;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.rename_path = None;
                        }
                    });
                });
            if !open { self.rename_path = None; }
        }

        if let Some((parent, is_dir)) = self.new_item_parent.clone() {
            let mut open = true;
            let title = if is_dir { "New Folder" } else { "New File" };
            egui::Window::new(title)
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label(format!("Parent: {}", parent.to_string_lossy()));
                    ui.text_edit_singleline(&mut self.input_text);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            let new_path = parent.join(&self.input_text);
                            let success = if is_dir {
                                std::fs::create_dir_all(&new_path).is_ok()
                            } else {
                                std::fs::File::create(&new_path).is_ok()
                            };
                            if success {
                                self.new_item_parent = None;
                                self.expanded_nodes.insert(parent);
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.new_item_parent = None;
                        }
                    });
                });
            if !open { self.new_item_parent = None; }
        }
    }

    fn box_clone(&self) -> Box<dyn TabInstance> {
        Box::new(self.clone())
    }
}

// ----------------------------------------------------------------------------
// Plugin Implementation
// ----------------------------------------------------------------------------

pub struct FileManagerPlugin;

impl Plugin for FileManagerPlugin {
    fn name(&self) -> &str {
        "file_manager"
    }

    fn on_tab_menu(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("File Explorer").clicked() {
            control.push(AppCommand::OpenTab(Tab::new(Box::new(FileExplorerTab::new()))));
            ui.close_menu();
        }
    }
}

pub fn create() -> FileManagerPlugin {
    FileManagerPlugin
}