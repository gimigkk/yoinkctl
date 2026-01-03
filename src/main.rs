use eframe::egui;
use std::env;
use std::process::Command;
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey, GlobalHotKeyEvent};
use xcap::Monitor;
use image::RgbaImage;

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
    let hotkey_id = hotkey.id();
    
    // FORCE REGISTER: Try to unregister first, then register
    let _ = manager.unregister(hotkey);
    
    match manager.register(hotkey) {
        Ok(_) => {
            println!("✅ Hotkey registered! Press {} to pick colors", config.hotkey);
        }
        Err(e) => {
            eprintln!("⚠️  First registration failed ({}), forcing...", e);
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            let _ = manager.unregister(hotkey);
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            manager.register(hotkey)
                .map_err(|e| {
                    eprintln!("❌ Failed to force register hotkey '{}'", config.hotkey);
                    eprintln!("   Error: {}", e);
                    format!("Hotkey conflict: {}", e)
                })?;
            
            println!("✅ Hotkey forcefully registered!");
        }
    }
    
    let exe_path = env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {}", e))?;
    let receiver = GlobalHotKeyEvent::receiver();
    let mut last_activation = std::time::Instant::now();
    
    loop {
        // Use try_recv for non-blocking check
        match receiver.try_recv() {
            Ok(event) => {
                // CRITICAL: Always drain Release events to prevent state stuck
                // This prevents the X11 key release order bug
                if event.state == global_hotkey::HotKeyState::Released {
                    continue;
                }
                
                // Only process PRESSED events for our hotkey
                if event.id != hotkey_id {
                    continue;
                }
                
                let now = std::time::Instant::now();
                
                // Minimal debounce - just enough to prevent accidental double-press
                if now.duration_since(last_activation).as_millis() < 50 {
                    continue;
                }
                
                last_activation = now;
                
                // Launch picker immediately - let the picker handle its own locking
                // This prevents the daemon from being blocked by stale lock checks
                Command::new(&exe_path)
                    .arg("pick")
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .ok();
            }
            Err(_) => {
                // No event available - sleep a bit to reduce CPU usage
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }
}

fn run_picker() -> Result<(), eframe::Error> {
    let lock_path = std::env::temp_dir().join("yoinkctl-picker.lock");
    
    // ATOMIC LOCK: Create file and write PID immediately
    let lock_acquired = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(mut file) => {
            use std::io::Write;
            let pid = std::process::id();
            write!(file, "{}", pid).ok();
            file.flush().ok();
            file.sync_all().ok();
            drop(file);
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lock exists - verify if process is alive
            if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    #[cfg(target_os = "linux")]
                    {
                        if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                            return Ok(()); // Already running
                        }
                    }
                }
            }
            
            // Stale lock - remove and retry ONCE
            std::fs::remove_file(&lock_path).ok();
            
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut file) => {
                    use std::io::Write;
                    let pid = std::process::id();
                    write!(file, "{}", pid).ok();
                    file.flush().ok();
                    file.sync_all().ok();
                    drop(file);
                    true
                }
                Err(_) => return Ok(()), // Another instance won
            }
        }
        Err(_) => return Ok(()),
    };
    
    if !lock_acquired {
        return Ok(());
    }
    
    // SPEED OPTIMIZATION: Parallel screenshot + config loading
    let screenshot_handle = std::thread::spawn(|| {
        capture_all_screens()
    });
    
    let config_handle = std::thread::spawn(|| {
        Config::load().unwrap_or_default()
    });
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_fullscreen(true)
            .with_always_on_top()
            .with_mouse_passthrough(false)
            .with_active(true),
        centered: false,
        // OPTIMIZATION: Disable hardware acceleration if not needed - faster startup
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        // Keep vsync enabled for smooth animation
        ..Default::default()
    };
    
    let result = eframe::run_native(
        "yoinkctl Picker",
        options,
        Box::new(move |cc| {
            // Retrieve pre-loaded data from parallel threads
            let (screenshot, offset) = screenshot_handle.join().unwrap_or((None, (0, 0)));
            let config = config_handle.join().unwrap_or_default();
            
            // OPTIMIZATION: Disable font rasterization delay by using default fonts
            // This speeds up first frame render significantly
            
            Ok(Box::new(ColorPicker::new_with_config(cc, screenshot, offset, config)))
        }),
    );
    
    // Always clean up lock file
    std::fs::remove_file(&lock_path).ok();
    result
}

// Helper function for parallel screenshot capture
fn capture_all_screens() -> (Option<RgbaImage>, (i32, i32)) {
    if let Ok(monitors) = Monitor::all() {
        if let Some(monitor) = monitors.first() {
            if let Ok(image) = monitor.capture_image() {
                let x_offset = monitor.x();
                let y_offset = monitor.y();
                return (Some(image), (x_offset, y_offset));
            }
        }
    }
    
    (None, (0, 0))
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