use eframe::egui;
use std::env;
use std::process::Command;
use global_hotkey::{GlobalHotKeyManager, hotkey::HotKey};

mod picker;
mod config;

use picker::ColorPicker;
use config::Config;

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
            
            // Increased debounce time to 500ms to prevent double-spawns
            if now.duration_since(last_activation).as_millis() < 500 {
                continue;
            }
            
            // Additional check: is a picker already running?
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
            
            // Spawn the picker
            Command::new(&exe_path)
                .arg("pick")
                .spawn()
                .ok();
        }
    }
}

fn run_picker() -> Result<(), eframe::Error> {
    // Use a proper file-based mutex for locking
    let lock_path = std::env::temp_dir().join("yoinkctl-picker.lock");
    
    // Try to create the lock file exclusively (atomic operation)
    // This will fail if the file already exists
    let lock_acquired = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock_path)
    {
        Ok(mut file) => {
            // Successfully created lock, write our PID
            use std::io::Write;
            let pid = std::process::id();
            writeln!(file, "{}", pid).ok();
            drop(file); // Ensure file is flushed
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Lock file exists, check if process is actually running
            if let Ok(pid_str) = std::fs::read_to_string(&lock_path) {
                if let Ok(pid) = pid_str.trim().parse::<i32>() {
                    #[cfg(target_os = "linux")]
                    {
                        if std::path::Path::new(&format!("/proc/{}", pid)).exists() {
                            // Process is running, don't launch
                            println!("Color picker already running (PID: {})", pid);
                            return Ok(());
                        }
                    }
                }
            }
            
            // Lock file is stale, remove and retry
            std::fs::remove_file(&lock_path).ok();
            
            // Small delay to ensure filesystem sync
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            // Retry once after removing stale lock
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
                    // Another process beat us to it
                    println!("Another picker instance started first");
                    return Ok(());
                }
            }
        }
        Err(_) => {
            // Other error, bail
            println!("Failed to create lock file");
            return Ok(());
        }
    };
    
    if !lock_acquired {
        return Ok(());
    }
    
    // Small delay to ensure lock file is fully written before window appears
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
    
    // Clean up lock file immediately on exit
    std::fs::remove_file(&lock_path).ok();
    result
}

fn run_config_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 480.0])
            .with_resizable(true)
            .with_min_inner_size([500.0, 400.0]),
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
            Ok(Box::new(ConfigApp::new(cc)))
        }),
    )
}

struct ConfigApp {
    config: Config,
    daemon_running: bool,
    save_message: Option<(String, std::time::Instant)>,
}

impl ConfigApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            config: Config::load().unwrap_or_default(),
            daemon_running: is_daemon_running(),
            save_message: None,
        }
    }
}

fn is_daemon_running() -> bool {
    #[cfg(target_os = "linux")]
    {
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
        if let Some((_, instant)) = &self.save_message {
            if instant.elapsed().as_secs() > 2 {
                self.save_message = None;
            }
            ctx.request_repaint();
        }
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 20)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(30.0);
                    
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("yoinkctl").size(28.0).strong());
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("Color Picker").size(14.0).color(egui::Color32::GRAY));
                    });
                    
                    ui.add_space(30.0);
                    
                    ui.horizontal(|ui| {
                        let available = ui.available_width();
                        let max_width = 480.0;
                        let margin = ((available - max_width) / 2.0).max(20.0);
                        
                        ui.add_space(margin);
                        
                        ui.vertical(|ui| {
                            ui.set_width(available - margin * 2.0);
                            
                            ui.horizontal(|ui| {
                                let card_width = (ui.available_width() - 16.0) / 2.0;
                                let card_height = 150.0;
                                
                                ui.vertical(|ui| {
                                    ui.set_width(card_width);
                                    ui.set_height(card_height);
                                    
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_rgb(28, 28, 32))
                                        .rounding(12.0)
                                        .inner_margin(20.0)
                                        .show(ui, |ui| {
                                            ui.label(egui::RichText::new("Hotkey Daemon").size(16.0).strong());
                                            ui.add_space(8.0);
                                            
                                            if self.daemon_running {
                                                ui.label(
                                                    egui::RichText::new("● Running")
                                                        .size(13.0)
                                                        .color(egui::Color32::from_rgb(74, 222, 128))
                                                );
                                                ui.add_space(6.0);
                                                ui.label(egui::RichText::new(&self.config.hotkey).size(12.0).color(egui::Color32::GRAY));
                                            } else {
                                                ui.label(
                                                    egui::RichText::new("○ Stopped")
                                                        .size(13.0)
                                                        .color(egui::Color32::GRAY)
                                                );
                                                ui.add_space(6.0);
                                                ui.label(egui::RichText::new("Enable hotkey").size(12.0).color(egui::Color32::GRAY));
                                            }
                                            
                                            ui.add_space(8.0);
                                            
                                            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                                                if self.daemon_running {
                                                    if ui.add_sized(
                                                        [ui.available_width(), 36.0],
                                                        egui::Button::new(egui::RichText::new("Stop").color(egui::Color32::BLACK))
                                                            .fill(egui::Color32::from_rgb(239, 68, 68))
                                                            .rounding(8.0)
                                                    ).clicked() {
                                                        stop_daemon();
                                                        self.daemon_running = false;
                                                    }
                                                } else {
                                                    if ui.add_sized(
                                                        [ui.available_width(), 36.0],
                                                        egui::Button::new(egui::RichText::new("Start").color(egui::Color32::BLACK))
                                                            .fill(egui::Color32::from_rgb(59, 130, 246))
                                                            .rounding(8.0)
                                                    ).clicked() {
                                                        start_daemon();
                                                        self.daemon_running = true;
                                                    }
                                                }
                                            });
                                        });
                                });
                                
                                ui.add_space(16.0);
                                
                                ui.vertical(|ui| {
                                    ui.set_width(card_width);
                                    ui.set_height(card_height);
                                    
                                    egui::Frame::none()
                                        .fill(egui::Color32::from_rgb(28, 28, 32))
                                        .rounding(12.0)
                                        .inner_margin(20.0)
                                        .show(ui, |ui| {
                                            ui.label(egui::RichText::new("Quick Launch").size(16.0).strong());
                                            ui.add_space(8.0);
                                            ui.label(egui::RichText::new("Test without hotkey").size(12.0).color(egui::Color32::GRAY));
                                            ui.add_space(8.0);
                                            
                                            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                                                if ui.add_sized(
                                                    [ui.available_width(), 36.0],
                                                    egui::Button::new(egui::RichText::new("Launch Picker").color(egui::Color32::BLACK))
                                                        .fill(egui::Color32::from_rgb(139, 92, 246))
                                                        .rounding(8.0)
                                                ).clicked() {
                                                    let exe_path = env::current_exe()
                                                        .ok()
                                                        .and_then(|p| p.to_str().map(|s| s.to_string()))
                                                        .unwrap_or_else(|| "yoinkctl".to_string());
                                                    
                                                    Command::new(&exe_path)
                                                        .arg("pick")
                                                        .spawn()
                                                        .ok();
                                                }
                                            });
                                        });
                                });
                            });
                            
                            ui.add_space(16.0);
                            
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(28, 28, 32))
                                .rounding(12.0)
                                .inner_margin(20.0)
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new("Settings").size(16.0).strong());
                                    ui.add_space(12.0);
                                    
                                    ui.label(egui::RichText::new("Global Hotkey").size(14.0).strong());
                                    ui.add_space(8.0);
                                    
                                    ui.horizontal(|ui| {
                                        let parts: Vec<&str> = self.config.hotkey.split('+').collect();
                                        let current_key = parts.last().unwrap_or(&"A").trim().to_string();
                                        
                                        let has_super = self.config.hotkey.contains("Super");
                                        let has_shift = self.config.hotkey.contains("Shift");
                                        let has_ctrl = self.config.hotkey.contains("Ctrl");
                                        let has_alt = self.config.hotkey.contains("Alt");
                                        
                                        let mut new_super = has_super;
                                        let mut new_shift = has_shift;
                                        let mut new_ctrl = has_ctrl;
                                        let mut new_alt = has_alt;
                                        let mut new_key = current_key.clone();
                                        
                                        ui.checkbox(&mut new_super, "Super");
                                        ui.checkbox(&mut new_shift, "Shift");
                                        ui.checkbox(&mut new_ctrl, "Ctrl");
                                        ui.checkbox(&mut new_alt, "Alt");
                                        
                                        ui.label("+");
                                        
                                        egui::ComboBox::from_id_salt("key")
                                            .selected_text(&new_key)
                                            .show_ui(ui, |ui| {
                                                for key in 'A'..='Z' {
                                                    let key_str = key.to_string();
                                                    ui.selectable_value(&mut new_key, key_str.clone(), key_str);
                                                }
                                            });
                                        
                                        let mut parts = Vec::new();
                                        if new_super { parts.push("Super"); }
                                        if new_shift { parts.push("Shift"); }
                                        if new_ctrl { parts.push("Ctrl"); }
                                        if new_alt { parts.push("Alt"); }
                                        parts.push(&new_key);
                                        
                                        self.config.hotkey = parts.join("+");
                                    });
                                    
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new(&format!("Current: {}", self.config.hotkey))
                                            .size(12.0)
                                            .color(egui::Color32::from_gray(180))
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new("⚠️ Restart daemon after changing")
                                            .size(11.0)
                                            .color(egui::Color32::from_rgb(251, 191, 36))
                                    );
                                    
                                    ui.add_space(12.0);
                                    ui.separator();
                                    ui.add_space(12.0);
                                    
                                    ui.label(egui::RichText::new("Display Options").size(14.0).strong());
                                    ui.add_space(8.0);
                                    
                                    ui.checkbox(&mut self.config.show_hex, "Show HEX codes");
                                    ui.checkbox(&mut self.config.show_rgb, "Show RGB values");
                                    ui.checkbox(&mut self.config.show_hsl, "Show HSL values");
                                    
                                    ui.add_space(12.0);
                                    ui.separator();
                                    ui.add_space(12.0);
                                    
                                    ui.label("Preview Size");
                                    ui.add(egui::Slider::new(&mut self.config.preview_size, 50..=200).suffix(" px"));
                                    
                                    ui.add_space(16.0);
                                    
                                    ui.horizontal(|ui| {
                                        if ui.add_sized(
                                            [120.0, 36.0],
                                            egui::Button::new(egui::RichText::new("Save Settings").color(egui::Color32::BLACK))
                                                .fill(egui::Color32::from_rgb(34, 197, 94))
                                                .rounding(8.0)
                                        ).clicked() {
                                            match self.config.save() {
                                                Ok(_) => {
                                                    self.save_message = Some(("Settings saved!".to_string(), std::time::Instant::now()));
                                                }
                                                Err(e) => {
                                                    self.save_message = Some((format!("Error: {}", e), std::time::Instant::now()));
                                                }
                                            }
                                        }
                                        
                                        if let Some((msg, _)) = &self.save_message {
                                            ui.add_space(8.0);
                                            ui.label(egui::RichText::new(msg).color(egui::Color32::from_rgb(34, 197, 94)));
                                        }
                                    });
                                });
                            
                            ui.add_space(30.0);
                        });
                        
                        ui.add_space(margin);
                    });
                });
            });
    }
}