pub mod core;
include!("generated.rs");

use crate::Plugin;
use std::collections::{HashMap, HashSet};

pub fn all_plugins() -> Vec<Box<dyn Plugin>> {
    let mut raw_plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(core::CorePlugin::default()),
    ];
    raw_plugins.extend(get_extra_plugins());

    sort_plugins(raw_plugins)
}

/// 拓扑排序插件列表，确保依赖项排在前面
fn sort_plugins(plugins: Vec<Box<dyn Plugin>>) -> Vec<Box<dyn Plugin>> {
    let mut name_to_plugin: HashMap<String, Box<dyn Plugin>> = plugins
        .into_iter()
        .map(|p| (p.name().to_string(), p))
        .collect();

    let mut sorted_names = Vec::new();
    let mut visited = HashSet::new();
    let mut visiting = HashSet::new();

    let names: Vec<String> = name_to_plugin.keys().cloned().collect();

    for name in names {
        if !visited.contains(&name) {
            if !visit(&name, &name_to_plugin, &mut visited, &mut visiting, &mut sorted_names) {
                // 如果发现循环依赖，这里简单处理：打印警告并继续
                eprintln!("Warning: Circular dependency or missing dependency detected for plugin: {}", name);
            }
        }
    }

    sorted_names
        .into_iter()
        .filter_map(|name| name_to_plugin.remove(&name))
        .collect()
}

fn visit(
    name: &str,
    registry: &HashMap<String, Box<dyn Plugin>>,
    visited: &mut HashSet<String>,
    visiting: &mut HashSet<String>,
    sorted: &mut Vec<String>,
) -> bool {
    if visiting.contains(name) { return false; } // 发现环
    if visited.contains(name) { return true; }

    visiting.insert(name.to_string());

    if let Some(plugin) = registry.get(name) {
        for dep in plugin.dependencies() {
            if !visit(&dep, registry, visited, visiting, sorted) {
                return false;
            }
        }
    }

    visiting.remove(name);
    visited.insert(name.to_string());
    sorted.push(name.to_string());
    true
}