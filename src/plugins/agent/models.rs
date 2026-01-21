use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AgentConfig {
    pub script_directory: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

impl AgentConfig {
    pub fn load() -> Self {
        let path = std::path::Path::new("agent_config.toml");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                return toml::from_str(&content).unwrap_or_default();
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = std::path::Path::new("agent_config.toml");
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = std::fs::write(path, content);
        }
    }
}
