use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::error::Result;

const CONFIG_FILE: &str = "settings.toml";
const APP_NAME: &str = "IcedLens";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub language: Option<String>,
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push(APP_NAME);
        path.push(CONFIG_FILE);
        path
    })
}

pub fn load() -> Result<Config> {
    if let Some(path) = get_config_path() {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            return Ok(toml::from_str(&content).unwrap_or_default());
        }
    }
    Ok(Config::default())
}

pub fn save(config: &Config) -> Result<()> {
    if let Some(path) = get_config_path() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(config).unwrap();
        fs::write(path, content)?;
    }
    Ok(())
}
