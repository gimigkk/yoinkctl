use std::path::PathBuf;
use std::fs;
use std::env;

pub struct Autostart {
    desktop_file_path: PathBuf,
}

impl Autostart {
    pub fn new() -> Self {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
        path.push("autostart");
        fs::create_dir_all(&path).ok();
        path.push("yoinkctl.desktop");
        
        Self {
            desktop_file_path: path,
        }
    }
    
    pub fn is_enabled(&self) -> bool {
        self.desktop_file_path.exists()
    }
    
    pub fn enable(&self) -> Result<(), String> {
        let exe_path = env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;
        
        let exe_path_str = exe_path.to_str()
            .ok_or_else(|| "Invalid executable path".to_string())?;
        
        let desktop_content = format!(
            "[Desktop Entry]
Type=Application
Name=yoinkctl
Comment=Color picker daemon with global hotkey
Exec={} daemon
Terminal=false
Hidden=false
X-GNOME-Autostart-enabled=true
",
            exe_path_str
        );
        
        fs::write(&self.desktop_file_path, desktop_content)
            .map_err(|e| format!("Failed to create autostart file: {}", e))?;
        
        Ok(())
    }
    
    pub fn disable(&self) -> Result<(), String> {
        if self.desktop_file_path.exists() {
            fs::remove_file(&self.desktop_file_path)
                .map_err(|e| format!("Failed to remove autostart file: {}", e))?;
        }
        Ok(())
    }
}