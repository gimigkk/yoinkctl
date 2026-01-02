use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use global_hotkey::hotkey::{Modifiers, Code};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: String,
    pub show_hex: bool,
    pub show_rgb: bool,
    pub show_hsl: bool,
    pub preview_size: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "Super+Shift+A".to_string(),
            show_hex: true,
            show_rgb: true,
            show_hsl: true,
            preview_size: 120,
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("yoinkctl");
        std::fs::create_dir_all(&path).ok();
        path.push("config.json");
        path
    }
    
    pub fn load() -> Result<Self, String> {
        let path = Self::config_path();
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write config: {}", e))
    }
    
    pub fn get_modifiers(&self) -> Modifiers {
        let mut modifiers = Modifiers::empty();
        
        if self.hotkey.contains("Super") {
            modifiers |= Modifiers::SUPER;
        }
        if self.hotkey.contains("Shift") {
            modifiers |= Modifiers::SHIFT;
        }
        if self.hotkey.contains("Ctrl") {
            modifiers |= Modifiers::CONTROL;
        }
        if self.hotkey.contains("Alt") {
            modifiers |= Modifiers::ALT;
        }
        
        modifiers
    }
    
    pub fn get_key_code(&self) -> Code {
        // Extract the key from the hotkey string
        let parts: Vec<&str> = self.hotkey.split('+').collect();
        let key = parts.last().unwrap_or(&"A").trim();
        
        // Map the key string to a Code
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
            _ => Code::KeyA, // Default fallback
        }
    }
    
    /// Validates that the hotkey has at least one modifier
    pub fn validate_hotkey(&self) -> Result<(), String> {
        let modifiers = self.get_modifiers();
        if modifiers.is_empty() {
            Err("Hotkey must have at least one modifier (Super, Shift, Ctrl, or Alt)".to_string())
        } else {
            Ok(())
        }
    }
}