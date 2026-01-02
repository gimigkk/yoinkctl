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
        
        Ok(config_dir.join("yoinkctl").join("config.json"))
    }
    
    // Parse hotkey string into modifiers and key
    pub fn get_modifiers(&self) -> global_hotkey::hotkey::Modifiers {
        use global_hotkey::hotkey::Modifiers;
        
        let parts: Vec<&str> = self.hotkey.split('+').collect();
        let mut modifiers = Modifiers::empty();
        
        for part in &parts[..parts.len().saturating_sub(1)] {
            match part.trim() {
                "Super" | "Meta" => modifiers |= Modifiers::SUPER,
                "Shift" => modifiers |= Modifiers::SHIFT,
                "Ctrl" | "Control" => modifiers |= Modifiers::CONTROL,
                "Alt" => modifiers |= Modifiers::ALT,
                _ => {}
            }
        }
        
        modifiers
    }
    
    pub fn get_key_code(&self) -> global_hotkey::hotkey::Code {
        use global_hotkey::hotkey::Code;
        
        let parts: Vec<&str> = self.hotkey.split('+').collect();
        let key = parts.last().unwrap_or(&"A").trim();
        
        match key {
            "A" => Code::KeyA,
            "B" => Code::KeyB,
            "C" => Code::KeyC,
            "D" => Code::KeyD,
            "E" => Code::KeyE,
            "F" => Code::KeyF,
            "G" => Code::KeyG,
            "H" => Code::KeyH,
            "I" => Code::KeyI,
            "J" => Code::KeyJ,
            "K" => Code::KeyK,
            "L" => Code::KeyL,
            "M" => Code::KeyM,
            "N" => Code::KeyN,
            "O" => Code::KeyO,
            "P" => Code::KeyP,
            "Q" => Code::KeyQ,
            "R" => Code::KeyR,
            "S" => Code::KeyS,
            "T" => Code::KeyT,
            "U" => Code::KeyU,
            "V" => Code::KeyV,
            "W" => Code::KeyW,
            "X" => Code::KeyX,
            "Y" => Code::KeyY,
            "Z" => Code::KeyZ,
            _ => Code::KeyA, // default fallback
        }
    }
}