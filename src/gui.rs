use eframe::egui;
use std::env;
use std::process::Command;
use arboard::Clipboard;

use crate::config::Config;
use crate::autostart::Autostart;
use crate::history::ColorHistory;

pub struct ConfigApp {
    config: Config,
    daemon_running: bool,
    save_message: Option<(String, std::time::Instant)>,
    autostart: Autostart,
    history: ColorHistory,
    show_settings_window: bool,
    copy_message: Option<(String, std::time::Instant)>,
    hovered_index: Option<usize>,
    last_history_reload: std::time::Instant,
}

impl ConfigApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            config: Config::load().unwrap_or_default(),
            daemon_running: is_daemon_running(),
            save_message: None,
            autostart: Autostart::new(),
            history: ColorHistory::load().unwrap_or_default(),
            show_settings_window: false,
            copy_message: None,
            hovered_index: None,
            last_history_reload: std::time::Instant::now(),
        }
    }
    
    fn reload_history_if_needed(&mut self) {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_history_reload).as_secs() >= 1 {
            if let Ok(history) = ColorHistory::load() {
                self.history = history;
            }
            self.last_history_reload = now;
        }
    }
    
    fn clear_expired_messages(&mut self) {
        if let Some((_, instant)) = &self.save_message {
            if instant.elapsed().as_secs() > 2 {
                self.save_message = None;
            }
        }
        
        if let Some((_, instant)) = &self.copy_message {
            if instant.elapsed().as_secs() > 2 {
                self.copy_message = None;
            }
        }
    }
    
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical_centered(|ui| {
                ui.set_width(ui.available_width() - 60.0);
                ui.label(egui::RichText::new("yoinkctl").size(28.0).strong());
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Color Picker").size(14.0).color(egui::Color32::GRAY));
            });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.add_space(20.0);
                if ui.add_sized(
                    [40.0, 40.0],
                    egui::Button::new(egui::RichText::new("⚙").size(20.0))
                        .fill(egui::Color32::from_rgb(28, 28, 32))
                        .rounding(8.0)
                ).clicked() {
                    self.show_settings_window = !self.show_settings_window;
                }
            });
        });
    }
    
    fn render_daemon_card(&mut self, ui: &mut egui::Ui, card_width: f32, card_height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
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
            }
        );
    }
    
    fn render_quick_launch_card(&mut self, ui: &mut egui::Ui, card_width: f32, card_height: f32) {
        ui.allocate_ui_with_layout(
            egui::vec2(card_width, card_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
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
            }
        );
    }
    
    fn render_control_cards(&mut self, ui: &mut egui::Ui) -> (f32, f32) {
        let available = ui.available_width();
        let max_width = 480.0;
        let margin = ((available - max_width) / 2.0).max(20.0);
        let content_width = available - margin * 2.0;
        let gap = 16.0;
        
        ui.horizontal(|ui| {
            ui.add_space(margin);
            
            ui.spacing_mut().item_spacing.x = 0.0; // Remove default spacing
            
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, 150.0),
                egui::Layout::left_to_right(egui::Align::Min),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0; // Remove default spacing
                    
                    let card_width = (content_width - gap) / 2.0;
                    let card_height = 150.0;
                    
                    self.render_daemon_card(ui, card_width, card_height);
                    ui.add_space(gap);
                    self.render_quick_launch_card(ui, card_width, card_height);
                }
            );
            
            ui.add_space(margin);
        });
        
        (margin, content_width)
    }
    
    fn render_history_entry(&mut self, ui: &mut egui::Ui, idx: usize, entry: &crate::history::ColorEntry, bg_color: egui::Color32) {
        let response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 32.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                egui::Frame::none()
                    .fill(bg_color)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        
                        ui.horizontal(|ui| {
                            ui.add_space(20.0);
                            
                            let color = egui::Color32::from_rgb(entry.rgb.0, entry.rgb.1, entry.rgb.2);
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(16.0, 16.0),
                                egui::Sense::hover()
                            );
                            ui.painter().rect_filled(rect, 2.0, color);
                            ui.painter().rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_gray(60)));
                            
                            ui.add_space(12.0);
                            
                            ui.label(egui::RichText::new(&entry.hex)
                                .size(13.0)
                                .color(egui::Color32::from_rgb(200, 200, 255))
                                .family(egui::FontFamily::Monospace));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("│")
                                    .size(12.0)
                                    .color(egui::Color32::from_gray(60))
                                    .family(egui::FontFamily::Monospace));
                            });
                        });
                    });
            }
        ).response;
        
        if response.hovered() {
            self.hovered_index = Some(idx);
        }
        
        if response.clicked() {
            if let Ok(mut clipboard) = Clipboard::new() {
                if clipboard.set_text(&entry.hex).is_ok() {
                    self.copy_message = Some((format!("Copied {}!", entry.hex), std::time::Instant::now()));
                }
            }
        }
        
        if self.hovered_index == Some(idx) {
            ui.painter().text(
                egui::pos2(response.rect.right() - 140.0, response.rect.center().y),
                egui::Align2::LEFT_CENTER,
                "<- click to copy",
                egui::FontId::new(11.0, egui::FontFamily::Monospace),
                egui::Color32::from_rgb(100, 255, 100),
            );
        }
    }
    
    fn render_history_card(&mut self, ui: &mut egui::Ui, remaining_height: f32, margin: f32, content_width: f32) {
        ui.horizontal(|ui| {
            ui.add_space(margin);
            
            ui.allocate_ui_with_layout(
                egui::vec2(content_width, remaining_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(12, 12, 14))
                        .rounding(12.0)
                        .inner_margin(0.0)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 44)))
                        .show(ui, |ui| {
                            ui.add_space(16.0);
                            
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("┌─ [Color History] ─┐")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(100, 255, 100))
                                    .family(egui::FontFamily::Monospace));
                                
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(20.0);
                                    if ui.button(egui::RichText::new("Clear").size(11.0).family(egui::FontFamily::Monospace))
                                        .clicked() && !self.history.entries.is_empty() {
                                        self.history.clear();
                                    }
                                });
                            });
                            
                            ui.add_space(8.0);
                            
                            let scroll_height = remaining_height - 100.0;
                            
                            egui::ScrollArea::vertical()
                                .max_height(scroll_height)
                                .show(ui, |ui| {
                                    self.hovered_index = None;
                                    
                                    if self.history.entries.is_empty() {
                                        ui.add_space(8.0);
                                        ui.horizontal(|ui| {
                                            ui.add_space(20.0);
                                            ui.label(egui::RichText::new("│ No colors picked yet")
                                                .size(12.0)
                                                .color(egui::Color32::from_gray(100))
                                                .family(egui::FontFamily::Monospace));
                                        });
                                        ui.add_space(8.0);
                                    } else {
                                        let entries: Vec<_> = self.history.entries.clone();
                                        for (idx, entry) in entries.iter().enumerate() {
                                            let bg_color = if idx % 2 == 0 {
                                                egui::Color32::from_rgb(16, 16, 18)
                                            } else {
                                                egui::Color32::from_rgb(12, 12, 14)
                                            };
                                            self.render_history_entry(ui, idx, entry, bg_color);
                                        }
                                    }
                                });
                            
                            ui.add_space(8.0);
                            
                            ui.horizontal(|ui| {
                                ui.add_space(20.0);
                                ui.label(egui::RichText::new("└───────────────────┘")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(100, 255, 100))
                                    .family(egui::FontFamily::Monospace));
                            });
                            
                            if let Some((msg, _)) = &self.copy_message {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.add_space(20.0);
                                    ui.label(egui::RichText::new(msg)
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(100, 255, 100))
                                        .family(egui::FontFamily::Monospace));
                                });
                            }
                            
                            ui.add_space(16.0);
                        });
                }
            );
            
            ui.add_space(margin);
        });
    }
    
    fn draw_settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings_window {
            return;
        }
        
        let settings_id = egui::ViewportId::from_hash_of("settings_window");
        
        ctx.show_viewport_immediate(
            settings_id,
            egui::ViewportBuilder::default()
                .with_title("Settings")
                .with_inner_size([450.0, 500.0])
                .with_min_inner_size([400.0, 400.0])
                .with_resizable(true),
            |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
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
                    
                    let parts: Vec<&str> = self.config.hotkey.split('+').collect();
                    let modifier_count = parts.len() - 1;
                    
                    if modifier_count == 0 {
                        ui.label(
                            egui::RichText::new("⚠️ At least one modifier required (Super/Shift/Ctrl/Alt)")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(239, 68, 68))
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(&format!("Current: {}", self.config.hotkey))
                                .size(12.0)
                                .color(egui::Color32::from_gray(180))
                        );
                    }
                    
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("⚠️ Restart daemon after changing")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(251, 191, 36))
                    );
                    
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);
                    
                    ui.label(egui::RichText::new("Startup Options").size(14.0).strong());
                    ui.add_space(8.0);
                    
                    let mut autostart_enabled = self.autostart.is_enabled();
                    if ui.checkbox(&mut autostart_enabled, "Launch daemon at startup").changed() {
                        if autostart_enabled {
                            if let Err(e) = self.autostart.enable() {
                                self.save_message = Some((format!("Autostart error: {}", e), std::time::Instant::now()));
                            } else {
                                self.save_message = Some(("Autostart enabled!".to_string(), std::time::Instant::now()));
                            }
                        } else {
                            if let Err(e) = self.autostart.disable() {
                                self.save_message = Some((format!("Autostart error: {}", e), std::time::Instant::now()));
                            } else {
                                self.save_message = Some(("Autostart disabled!".to_string(), std::time::Instant::now()));
                            }
                        }
                    }
                    
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
                            if let Err(e) = self.config.validate_hotkey() {
                                self.save_message = Some((format!("Invalid hotkey: {}", e), std::time::Instant::now()));
                            } else {
                                match self.config.save() {
                                    Ok(_) => {
                                        self.save_message = Some(("Settings saved!".to_string(), std::time::Instant::now()));
                                    }
                                    Err(e) => {
                                        self.save_message = Some((format!("Error: {}", e), std::time::Instant::now()));
                                    }
                                }
                            }
                        }
                        
                        if let Some((msg, _)) = &self.save_message {
                            ui.add_space(8.0);
                            let color = if msg.contains("Invalid") || msg.contains("Error") {
                                egui::Color32::from_rgb(239, 68, 68)
                            } else {
                                egui::Color32::from_rgb(34, 197, 94)
                            };
                            ui.label(egui::RichText::new(msg).color(color));
                        }
                    });
                });
                
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_settings_window = false;
                }
            },
        );
    }
}

impl eframe::App for ConfigApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.reload_history_if_needed();
        self.clear_expired_messages();
        ctx.request_repaint();
        
        if self.show_settings_window {
            self.draw_settings_window(ctx);
        }
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 20)))
            .show(ctx, |ui| {
                ui.add_space(30.0);
                self.render_header(ui);
                ui.add_space(30.0);
                let (margin, content_width) = self.render_control_cards(ui);
                ui.add_space(16.0);
                
                let remaining_height = ui.available_height();
                self.render_history_card(ui, remaining_height, margin, content_width);
            });
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