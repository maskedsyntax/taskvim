use serde::{Deserialize, Serialize};
use std::fs;
use directories::ProjectDirs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub theme: String,
    pub font: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "System".to_string(),
            font: None,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("com", "maskedsyntax", "taskit") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.json");

            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(config_path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(proj_dirs) = ProjectDirs::from("com", "maskedsyntax", "taskit") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir)?;
            let config_path = config_dir.join("config.json");
            let content = serde_json::to_string_pretty(self)?;
            fs::write(config_path, content)?;
        }
        Ok(())
    }
}
