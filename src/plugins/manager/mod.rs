use egui::{Ui, WidgetText};
use crate::{Plugin, AppCommand, TabInstance};
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, value};
use std::collections::BTreeMap;

// --- 数据模型 (严格对照独立启动器) ---

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
enum BuildMode {
    Debug,
    Release,
}

impl Default for BuildMode {
    fn default() -> Self { BuildMode::Debug }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct LauncherConfig {
    project_dir: Option<PathBuf>,
    #[serde(default)]
    enabled_plugins: Vec<String>,
    #[serde(default = "default_true")]
    build_and_run: bool,
    #[serde(default)]
    build_mode: BuildMode,
    #[serde(default)]
    export_path: Option<PathBuf>,
}

fn default_true() -> bool { true }

#[derive(Deserialize, Debug, Clone)]
struct PluginMeta {
    plugin: PluginInfo,
    #[serde(default)]
    external_dependencies: Option<toml::Table>,
}

#[derive(Deserialize, Debug, Clone)]
struct PluginInfo {
    name: String,
    display_name: String,
    #[allow(dead_code)]
    version: String,
    #[allow(dead_code)]
    description: String,
}

#[derive(Debug, Clone)]
struct PluginEntry {
    id: String,
    meta: PluginMeta,
    enabled: bool,
}

// --- Tab 实现 ---

#[derive(Debug, Clone)]
pub struct LauncherTab {
    config: LauncherConfig,
    plugins: Arc<Mutex<Vec<PluginEntry>>>,
    logs: Arc<Mutex<String>>,
    is_running: Arc<Mutex<bool>>,
}

impl LauncherTab {
    fn new() -> Self {
        let config = Self::load_config().unwrap_or_default();
        let mut s = Self {
            config,
            plugins: Arc::new(Mutex::new(Vec::new())),
            logs: Arc::new(Mutex::new(String::new())),
            is_running: Arc::new(Mutex::new(false)),
        };
        s.refresh_plugins();
        s
    }

    fn load_config() -> anyhow::Result<LauncherConfig> {
        let path = Path::new("launcher_config.toml");
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(LauncherConfig::default())
        }
    }

    fn save_config(&self) -> anyhow::Result<()> {
        let content = toml::to_string(&self.config)?;
        fs::write("launcher_config.toml", content)?;
        Ok(())
    }

    /// 严格对照独立启动器的扫描逻辑
    fn refresh_plugins(&mut self) {
        let mut plugins_lock = self.plugins.lock().unwrap();
        plugins_lock.clear();
        let Some(main_dir) = &self.config.project_dir else { return; };
        
        let plugins_dir = main_dir.join("src/plugins");
        if let Ok(entries) = fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let toml_path = path.join("plugin.toml");
                    if toml_path.exists() {
                        if let Ok(content) = fs::read_to_string(toml_path) {
                            if let Ok(meta) = toml::from_str::<PluginMeta>(&content) {
                                let id = meta.plugin.name.clone();
                                let enabled = self.config.enabled_plugins.contains(&id);
                                plugins_lock.push(PluginEntry { id, meta, enabled });
                            }
                        }
                    }
                }
            }
        }
    }

    /// 严格对照独立启动器的 Cargo.toml 同步逻辑
    fn sync_cargo_toml(&self) -> anyhow::Result<()> {
        let Some(main_dir) = &self.config.project_dir else { 
            return Err(anyhow::anyhow!("No project directory selected.")); 
        };
        let cargo_path = main_dir.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_path)?;
        let mut doc = content.parse::<DocumentMut>()?;

        let plugins = self.plugins.lock().unwrap();

        // 1. 同步 Features
        let mut enabled_features = Vec::new();
        let mut all_plugin_features = Vec::new();

        for plugin in plugins.iter() {
            let feat = format!("plugin_{}", plugin.id);
            all_plugin_features.push(feat.clone());
            if plugin.enabled {
                enabled_features.push(feat);
            }
        }

        if let Some(features) = doc.get_mut("features").and_then(|v| v.as_table_mut()) {
            // 清理所有已存在的 plugin_ 关键特征
            let keys: Vec<String> = features.iter().map(|(k, _)| k.to_string()).collect();
            for k in keys {
                if k.starts_with("plugin_") {
                    features.remove(&k);
                }
            }

            // 重新插入所有扫描到的插件特征（保持 Cargo.toml feature 定义完整）
            for feat in &all_plugin_features {
                features.insert(feat, value(toml_edit::Array::new()));
            }

            // 更新 default 特征
            let mut default_array = toml_edit::Array::new();
            for feat in enabled_features {
                default_array.push(feat);
            }
            features.insert("default", value(default_array));
        }

        // 2. 同步并去重外部依赖
        let mut merged_deps: BTreeMap<String, (toml::Value, Vec<String>)> = BTreeMap::new();
        for plugin in plugins.iter() {
            if plugin.enabled {
                if let Some(deps) = &plugin.meta.external_dependencies {
                    for (name, val) in deps {
                        let entry = merged_deps.entry(name.clone()).or_insert_with(|| (val.clone(), Vec::new()));
                        entry.1.push(plugin.id.clone());
                    }
                }
            }
        }

        let mut dep_string = String::from("\n");
        for (name, (val, sources)) in merged_deps {
            dep_string.push_str(&format!("# From {}\n", sources.join(" & ")));
            dep_string.push_str(&format!("{} = {}\n", name, val));
        }

        let mut final_content = doc.to_string();
        let begin_dep = "# --- BEGIN PLUGIN DEPENDENCIES ---";
        let end_dep = "# --- END PLUGIN DEPENDENCIES ---";

        if let (Some(start_idx), Some(end_idx)) = (final_content.find(begin_dep), final_content.find(end_dep)) {
            final_content.replace_range((start_idx + begin_dep.len())..end_idx, &dep_string);
        }

        fs::write(cargo_path, final_content)?;
        Ok(())
    }

    fn run_cargo_command(&self, args: Vec<String>) {
        if *self.is_running.lock().unwrap() { return; }
        
        let Some(main_dir) = self.config.project_dir.clone() else { return; };
        let logs = self.logs.clone();
        let is_running = self.is_running.clone();

        *is_running.lock().unwrap() = true;
        {
            let mut l = logs.lock().unwrap();
            l.clear();
            l.push_str(&format!("Executing: cargo {}\n", args.join(" ")));
        }

        std::thread::spawn(move || {
            let mut child = Command::new("cargo")
                .args(&args)
                .current_dir(&main_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start cargo");

            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();

            let l1 = logs.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    let mut l = l1.lock().unwrap(); l.push_str(&line); l.push('\n');
                }
            });

            let l2 = logs.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    let mut l = l2.lock().unwrap(); l.push_str(&line); l.push('\n');
                }
            });

            let status = child.wait();
            *is_running.lock().unwrap() = false;
            
            if let Ok(s) = status {
                let mut l = logs.lock().unwrap();
                l.push_str(&format!("\nProcess finished with exit code: {:?}\n", s.code()));
            }
        });
    }

    fn start_build_process(&self) {
        if let Err(e) = self.sync_cargo_toml() {
            let mut l = self.logs.lock().unwrap();
            l.push_str(&format!("Error syncing Cargo.toml: {}\n", e));
            return;
        }

        let mut args = if self.config.build_and_run {
            vec!["run".to_string()]
        } else {
            vec!["build".to_string()]
        };

        if self.config.build_mode == BuildMode::Release {
            args.push("--release".to_string());
        }

        self.run_cargo_command(args);
    }

    fn import_plugin(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let Some(main_dir) = &self.config.project_dir else { 
            return Err(anyhow::anyhow!("No project dir selected")); 
        };
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut plugin_name = None;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == "plugin.toml" {
                let mut content = String::new();
                use std::io::Read;
                file.read_to_string(&mut content)?;
                let meta: PluginMeta = toml::from_str(&content)?;
                plugin_name = Some(meta.plugin.name);
                break;
            }
        }

        if let Some(name) = plugin_name {
            let dest_dir = main_dir.join("src/plugins").join(&name);
            if !dest_dir.exists() {
                fs::create_dir_all(&dest_dir)?;
            }
            archive.extract(&dest_dir)?;
            self.refresh_plugins();
            return Ok(())
        }

        Err(anyhow::anyhow!("Invalid .verbium file: plugin.toml not found"))
    }
}

impl TabInstance for LauncherTab {
    fn title(&self) -> WidgetText { "Verbium Launcher".into() }

    fn ui(&mut self, ui: &mut Ui, _control: &mut Vec<AppCommand>) {
        egui::SidePanel::right("launcher_console")
            .resizable(true)
            .default_width(320.0)
            .width_range(200.0..=600.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading("📟 Console");
                    ui.separator();
                    
                    let logs = self.logs.lock().unwrap();
                    egui::ScrollArea::vertical()
                        .id_salt("log_scroll")
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut logs.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .lock_focus(true)
                            );
                        });
                });
            });

        egui::TopBottomPanel::bottom("launcher_config")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.heading("⚙ Configuration");
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("Project:");
                        let dir_str = self.config.project_dir.as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Not Selected".into());
                        
                        if ui.button(egui::RichText::new(dir_str).monospace()).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.config.project_dir = Some(path);
                                let _ = self.save_config();
                                self.refresh_plugins();
                            }
                        }
                                                if ui.button("🔄").clicked() { 
                                                    self.refresh_plugins(); 
                                                }
                                            });
                        
                                            ui.add_space(4.0);
                        
                                            ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui.radio_value(&mut self.config.build_mode, BuildMode::Debug, "Debug").changed() {
                            let _ = self.save_config();
                        }
                        if ui.radio_value(&mut self.config.build_mode, BuildMode::Release, "Release").changed() {
                            let _ = self.save_config();
                        }
                        ui.separator();
                        if ui.checkbox(&mut self.config.build_and_run, "Compile & Start").changed() {
                            let _ = self.save_config();
                        }
                    });

                    ui.add_space(4.0);

                    let running = *self.is_running.lock().unwrap();
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(!running && self.config.project_dir.is_some(), |ui| {
                            let btn_text = if self.config.build_and_run { "▶ Build & Run" } else { "🔨 Only Build" };
                            if ui.button(btn_text).clicked() {
                                self.start_build_process();
                            }
                            if ui.button("Clean").clicked() {
                                self.run_cargo_command(vec!["clean".to_string()]);
                            }
                        });
                        if running { ui.spinner(); }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Export to:");
                        let exp_str = self.config.export_path.as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Select path...".into());
                        
                        if ui.button(egui::RichText::new(exp_str).monospace()).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.config.export_path = Some(path);
                                let _ = self.save_config();
                            }
                        }

                        ui.add_enabled_ui(!running && self.config.export_path.is_some() && self.config.project_dir.is_some(), |ui| {
                            if ui.button("📤 Export").clicked() {
                                // 复用 build 逻辑但重定向结果
                                self.run_cargo_command(vec!["build".to_string(), "--release".to_string()]);
                            }
                        });
                    });
                });
                ui.add_space(4.0);
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Plugins");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("📥 Import .verbium").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Verbium Plugin", &["verbium", "zip"])
                                .pick_file() {
                                    if let Err(e) = self.import_plugin(path) {
                                        let mut l = self.logs.lock().unwrap();
                                        l.push_str(&format!("Import Error: {}\n", e));
                                    }
                                }
                        }
                    });
                });
                ui.separator();
                
                let mut plugins = self.plugins.lock().unwrap();
                let mut changed = false;

                egui::ScrollArea::vertical()
                    .id_salt("plugin_list")
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for plugin in plugins.iter_mut() {
                                if ui.checkbox(&mut plugin.enabled, &plugin.meta.plugin.display_name).changed() {
                                    changed = true;
                                }
                                ui.add_space(2.0);
                            }
                        });
                    });
                
                if changed {
                     self.config.enabled_plugins = plugins.iter()
                        .filter(|p| p.enabled)
                        .map(|p| p.id.clone())
                        .collect();
                    let _ = self.save_config();
                }
            });
        });
    }

    fn box_clone(&self) -> Box<dyn TabInstance> { Box::new(self.clone()) }
}

pub struct PluginLauncher;

impl Plugin for PluginLauncher {
    fn name(&self) -> &str { crate::plugins::PLUGIN_NAME_MANAGER }

    fn on_menu_bar(&mut self, ui: &mut Ui, control: &mut Vec<AppCommand>) {
        if ui.button("🚀 Launcher").clicked() {
            control.push(AppCommand::OpenTab(crate::Tab::new(Box::new(LauncherTab::new()))));
        }
    }
}

pub fn create() -> PluginLauncher {
    PluginLauncher
}