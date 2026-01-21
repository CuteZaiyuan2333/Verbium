use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AgentConfig {
    pub script_directory: Option<PathBuf>,
    pub default_chat_dir: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Agent,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChatSession {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    
    pub created_at: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub context_mode: String,
    pub model_name: String,
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

impl ChatSession {
    pub fn new(mode: String, model: String) -> Self {
        Self {
            path: None,
            created_at: Some(chrono::Local::now().to_rfc3339()),
            messages: Vec::new(),
            context_mode: mode,
            model_name: model,
        }
    }

    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut session: ChatSession = toml::from_str(&content)?;
        session.path = Some(path.to_path_buf());
        Ok(session)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.path {
            let content = toml::to_string_pretty(self)?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }
}
