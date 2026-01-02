use eframe::egui;
use std::env;
use std::process::Command;
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};

mod picker;
mod config;

use picker::ColorPicker;
use config::Config;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();
    
    // Check what mode to run in
    if args.len() > 1 {
        match args[1].as_str() {
            "pick" => {
                println!("Launching color picker...");
                return run_picker();
            }
            "daemon" => {
                println!("Starting yoinkctl hotkey daemon...");
                return run_daemon();
            }
            _ => {}
        }
    }
    
    // Default: show config GUI
    println!("Launching config GUI...");
    run_config_gui()
}

fn run_daemon() -> Result<(), eframe::Error> {
    println!("🚀 yoinkctl daemon starting...");
    println!("📌 Hotkey: Meta+Shift+A");
    
    // Create hotkey manager
    let manager = GlobalHotKeyManager::new().expect("Failed to create hotkey manager");
    
    // Register Meta+Shift+A (Super+Shift+A)
    let hotkey = HotKey::new(
        Some(Modifiers::SUPER | Modifiers::SHIFT),
        Code::KeyA
    );
    
    manager.register(hotkey).expect("Failed to register hotkey");
    
    println!("✅ Hotkey registered! Press Meta+Shift+A to pick colors");
    println!("   Press Ctrl+C to stop the daemon");
    
    // Get path to self
    let exe_path = env::current_exe()
        .expect("Failed to get executable path");
    
    // Listen for hotkey events
    let receiver = global_hotkey::GlobalHotKeyEvent::receiver();
    
    // Track last activation time to debounce
    let mut last_activation = std::time::Instant::now();
    
    loop {
        if let Ok(_event) = receiver.recv() {
            // Debounce: ignore if less than 500ms since last activation
            let now = std::time::Instant::now();
            if now.duration_since(last_activation).as_millis() < 500 {
                println!("⏭️  Ignoring duplicate hotkey press");
                continue;
            }
            last_activation = now;
            
            println!("🎨 Hotkey pressed! Launching picker...");
            
            // Launch the picker in a separate process with clean environment
            // Use setsid to detach from session and prevent startup notification
            #[cfg(target_os = "linux")]
            {
                Command::new("sh")
                    .arg("-c")
                    .arg(format!("export DESKTOP_STARTUP_ID=; {} pick", exe_path.display()))
                    .env_clear()
                    .env("PATH", std::env::var("PATH").unwrap_or_default())
                    .env("HOME", std::env::var("HOME").unwrap_or_default())
                    .env("DISPLAY", std::env::var("DISPLAY").unwrap_or_default())
                    .env("WAYLAND_DISPLAY", std::env::var("WAYLAND_DISPLAY").unwrap_or_default())
                    .env("XDG_RUNTIME_DIR", std::env::var("XDG_RUNTIME_DIR").unwrap_or_default())
                    .spawn()
                    .ok();
            }
            
            #[cfg(not(target_os = "linux"))]
            {
                Command::new(&exe_path)
                    .arg("pick")
                    .env_remove("DESKTOP_STARTUP_ID")
                    .spawn()
                    .ok();
            }
        }
    }
}

fn run_picker() -> Result<(), eframe::Error> {
    // Use a lock file to prevent multiple instances
    let lock_path = std::env::temp_dir().join("yoinkctl-picker.lock");
    
    // Check if lock file exists and process is still running
    if lock_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // Check if process is still running
                #[cfg(target_os = "linux")]
                {
                    if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                        println!("Color picker already running (PID: {}), ignoring...", pid);
                        return Ok(());
                    }
                }
            }
        }
    }
    
    // Create lock file with our PID
    let pid = std::process::id();
    std::fs::write(&lock_path, pid.to_string()).ok();
    
    // Disable KDE portal integration and startup notification
    std::env::set_var("QT_QPA_PLATFORMTHEME", "");
    std::env::set_var("QT_QPA_PLATFORM", "xcb");
    std::env::remove_var("DESKTOP_STARTUP_ID");
    
    // CRITICAL: Tell KDE we've finished launching immediately
    #[cfg(target_os = "linux")]
    {
        // Clear the startup notification
        std::env::set_var("DESKTOP_STARTUP_ID", "");
        
        // Use wmctrl or xdotool to signal completion if available
        Command::new("sh")
            .arg("-c")
            .arg("kill -CONT $ 2>/dev/null || true")
            .output()
            .ok();
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_fullscreen(true)
            .with_always_on_top()
            .with_mouse_passthrough(false)
            .with_active(true), // Activate immediately
        centered: false,
        ..Default::default()
    };
    
    let result = eframe::run_native(
        "yoinkctl Picker",
        options,
        Box::new(|cc| Ok(Box::new(ColorPicker::new(cc)))),
    );
    
    // Clean up lock file when done
    std::fs::remove_file(&lock_path).ok();
    
    result
}

fn run_config_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 450.0])
            .with_resizable(false),
        ..Default::default()
    };
    
    eframe::run_native(
        "yoinkctl Settings",
        options,
        Box::new(|cc| Ok(Box::new(ConfigApp::new(cc)))),
    )
}

struct ConfigApp {
    config: Config,
    daemon_running: bool,
}

impl ConfigApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Check if daemon is running
        let daemon_running = is_daemon_running();
        
        Self {
            config: Config::load().unwrap_or_default(),
            daemon_running,
        }
    }
}

fn is_daemon_running() -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        let output = Command::new("pgrep")
            .arg("-f")
            .arg("yoinkctl daemon")
            .output();
        
        if let Ok(output) = output {
            !output.stdout.is_empty()
        } else {
            false
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    false
}

fn start_daemon() {
    let exe_path = env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "yoinkctl".to_string());
    
    #[cfg(target_os = "linux")]
    {
        // Start daemon in background, detached from terminal
        Command::new("nohup")
            .arg(&exe_path)
            .arg("daemon")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok();
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        Command::new(&exe_path)
            .arg("daemon")
            .spawn()
            .ok();
    }
}

fn stop_daemon() {
    #[cfg(target_os = "linux")]
    {
        Command::new("pkill")
            .arg("-f")
            .arg("yoinkctl daemon")
            .output()
            .ok();
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            
            // Header
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("yoinkctl").size(32.0).strong());
                ui.label(egui::RichText::new("Quick color picker for your desktop").size(14.0).weak());
            });
            
            ui.add_space(30.0);
            ui.separator();
            ui.add_space(20.0);
            
            // Daemon status
            ui.heading("Hotkey Daemon");
            ui.add_space(10.0);
            
            if self.daemon_running {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🟢 Running").color(egui::Color32::GREEN));
                    ui.label("Press Meta+Shift+A to pick colors");
                });
                
                ui.add_space(10.0);
                
                if ui.button(egui::RichText::new("⏹️  Stop Daemon").size(14.0)).clicked() {
                    stop_daemon();
                    self.daemon_running = false;
                }
            } else {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("🔴 Not Running").color(egui::Color32::RED));
                });
                
                ui.add_space(10.0);
                
                if ui.button(egui::RichText::new("▶️  Start Daemon").size(14.0)).clicked() {
                    start_daemon();
                    self.daemon_running = true;
                }
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);
            
            // Quick test
            ui.heading("Quick Test");
            ui.add_space(10.0);
            
            if ui.button(egui::RichText::new("🎯  Test Color Picker Now").size(16.0))
                .clicked() 
            {
                let exe_path = env::current_exe()
                    .ok()
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| "yoinkctl".to_string());
                    
                std::process::Command::new(&exe_path)
                    .arg("pick")
                    .spawn()
                    .ok();
            }
            
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(20.0);
            
            // Settings
            ui.heading("Display Options");
            ui.add_space(10.0);
            
            ui.checkbox(&mut self.config.show_hex, "Show HEX color codes");
            ui.checkbox(&mut self.config.show_rgb, "Show RGB values");
            ui.checkbox(&mut self.config.show_hsl, "Show HSL values");
            
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("Preview size:");
                ui.add(egui::Slider::new(&mut self.config.preview_size, 50..=200).suffix("px"));
            });
            
            ui.add_space(20.0);
            
            if ui.button(egui::RichText::new("💾  Save Settings").size(14.0)).clicked() {
                match self.config.save() {
                    Ok(_) => {
                        // Visual feedback
                    }
                    Err(e) => {
                        eprintln!("Failed to save config: {}", e);
                    }
                }
            }
        });
    }
}