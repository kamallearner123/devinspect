use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, File};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub default_theme: Option<String>,
    pub enable_telemetry: Option<bool>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let mut config_builder = Config::builder();
        
        if let Some(home_dir) = dirs::home_dir() {
            let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config", "devinspect", "config.toml"].iter().collect();
            if config_path.exists() {
                config_builder = config_builder.add_source(File::from(config_path));
            }
        }
        
        let config = config_builder.build()?;
        config.try_deserialize()
    }
}
