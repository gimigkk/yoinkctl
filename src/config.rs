use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub hotkey: String,
    pub show_rgb: bool,
    pub show_hex: bool,
    pub show_hsl: bool,
    pub preview_size: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "Super+Shift+A".to_string(),
            show_rgb: true,
            show_hex: true,
            show_hsl: false,
            preview_size: 100,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let contents = fs::read_to_string(&path)?;
        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&path, contents)?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("chromasnap").join("config.json"))
    }
}