use eframe::egui;
use std::env;
use std::process::Command;
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey};

mod picker;
mod config;
mod autostart;
mod history;
mod gui;

use picker::ColorPicker;
use config::Config;
use gui::ConfigApp;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "pick" => {
                return run_picker();
            }
            "daemon" => {
                if let Err(e) = run_daemon() {
                    eprintln!("Daemon error: {}", e);
                }
                return Ok(());
            }
            _ => {}
        }
    }
    
    run_config_gui()
}

fn run_daemon() -> Result<(), String> {
    let config = Config::load().unwrap_or_default();
    
    println!("🚀 yoinkctl daemon starting...");
    println!("📌 Hotkey: {}", config.hotkey);
    
    let manager = GlobalHotKeyManager::new()
        .map_err(|e| format!("Failed to create hotkey manager: {}", e))?;
    
    let modifiers = config.get_modifiers();
    let key_code = config.get_key_code();
    
    if modifiers.is_empty() {
        return Err("Hotkey must have at least one modifier (Super, Shift, Ctrl, or Alt)".to_string());
    }
    
    let hotkey = HotKey::new(Some(modifiers), key_code);
    
    manager.register(hotkey)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;
    
    println!("✅ Hotkey registered! Press {} to pick colors", config.hotkey);
    
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let receiver = global_hotkey::GlobalHotKeyEvent::receiver();
    let mut last_activation = std::time::Instant::now();
    
    loop {
        if let Ok(_event) = receiver.recv() {
            let now = std::time::Instant::now();
            
            if now.duration_since(last_activation).as_millis() < 500 {
                continue;
            }
            
            let lock_path = std::env::temp_dir().join("yoinkctl-picker.lock");
            if lock_path.exists() {
                if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
                    if let Ok(pid) = pid_str.trim().parse::<i32>() {
                        #[cfg(target_os = "linux")]
                        {
                            if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                                println!("Picker already running, ignoring hotkey");
                                continue;
                            }
                        }
                    }
                }
            }
            
            last_activation = now;
            
            Command::new(&exe_path)
                .arg("pick")
                .spawn()
                .ok();
        }
    }
}

fn run_picker() -> Result<(), eframe::Error> {
    let lock_path = std::env::temp_dir().join("yoinkctl-picker.lock");
    
    let lock_acquired = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(mut file) => {
            use std::io::Write;
            let pid = std::process::id();
            writeln!(file, "{}", pid).ok();
            drop(file);
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    #[cfg(target_os = "linux")]
                    {
                        if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                            println!("Color picker already running (PID: {})", pid);
                            return Ok(());
                        }
                    }
                }
            }
            
            std::fs::remove_file(&lock_path).ok();
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut file) => {
                    use std::io::Write;
                    let pid = std::process::id();
                    writeln!(file, "{}", pid).ok();
                    drop(file);
                    true
                }
                Err(_) => {
                    println!("Another picker instance started first");
                    return Ok(());
                }
            }
        }
        Err(_) => {
            println!("Failed to create lock file");
            return Ok(());
        }
    };
    
    if !lock_acquired {
        return Ok(());
    }
    
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_fullscreen(true)
            .with_always_on_top()
            .with_mouse_passthrough(false)
            .with_active(true),
        centered: false,
        ..Default::default()
    };
    
    let result = eframe::run_native(
        "yoinkctl Picker",
        options,
        Box::new(|cc| Ok(Box::new(ColorPicker::new(cc)))),
    );
    
    std::fs::remove_file(&lock_path).ok();
    result
}

fn run_config_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 520.0])
            .with_resizable(true)
            .with_min_inner_size([500.0, 440.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "yoinkctl",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals {
                window_rounding: egui::Rounding::same(0.0),
                panel_fill: egui::Color32::from_rgb(18, 18, 20),
                ..egui::Visuals::dark()
            });
            
            cc.egui_ctx.set_embed_viewports(false);
            
            Ok(Box::new(ConfigApp::new(cc)))
        }),
    )
}