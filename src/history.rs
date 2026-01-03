use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorEntry {
    pub hex: String,
    pub rgb: (u8, u8, u8),
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorHistory {
    pub entries: Vec<ColorEntry>,
    max_entries: usize,
}

impl Default for ColorHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 50,
        }
    }
}

impl ColorHistory {
    pub fn history_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("yoinkctl");
        std::fs::create_dir_all(&path).ok();
        path.push("history.json");
        path
    }
    
    pub fn load() -> Result<Self, String> {
        let path = Self::history_path();
        
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read history: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse history: {}", e))
    }
    
    pub fn save(&self) -> Result<(), String> {
        let path = Self::history_path();
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;
        
        std::fs::write(&path, content)
            .map_err(|e| format!("Failed to write history: {}", e))
    }
    
    pub fn add_color(&mut self, hex: String, rgb: (u8, u8, u8)) {
        println!("🔍 DEBUG: Adding color {} to history", hex);
        
        // Check if this color already exists (don't add duplicates at the top)
        if let Some(pos) = self.entries.iter().position(|e| e.hex == hex) {
            println!("🔍 DEBUG: Color exists at position {}, moving to top", pos);
            // Move existing entry to top
            let entry = self.entries.remove(pos);
            self.entries.insert(0, entry);
        } else {
            println!("🔍 DEBUG: New color, adding to top");
            // Add new entry at top
            let entry = ColorEntry {
                hex: hex.clone(),
                rgb,
                timestamp: chrono::Utc::now().timestamp(),
            };
            
            self.entries.insert(0, entry);
            
            // Keep only max_entries
            if self.entries.len() > self.max_entries {
                self.entries.truncate(self.max_entries);
            }
        }
        
        println!("🔍 DEBUG: Total entries: {}", self.entries.len());
        
        match self.save() {
            Ok(_) => println!("🔍 DEBUG: Successfully saved history to {:?}", Self::history_path()),
            Err(e) => eprintln!("🔍 DEBUG: Failed to save history: {}", e),
        }
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.save().ok();
    }
}