use eframe::egui;
use xcap::Monitor;
use image::RgbaImage;
use arboard::Clipboard;
use std::process::Command;
use crate::config::Config;

pub struct ColorPicker {
    screenshot: Option<RgbaImage>,
    screenshot_offset: (i32, i32),
    cursor_pos: egui::Pos2,
    magnifier_pos: egui::Pos2, // Smoothed position for magnifier
    magnifier_offset: egui::Vec2, // Smoothed offset for magnifier positioning
    should_close: bool,
    made_sticky: bool,
    config: Config,
}

impl ColorPicker {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (screenshot, offset) = capture_all_screens();
        
        Self {
            screenshot,
            screenshot_offset: offset,
            cursor_pos: egui::Pos2::ZERO,
            magnifier_pos: egui::Pos2::ZERO, // Start at same position
            magnifier_offset: egui::vec2(30.0, 30.0), // Start with default offset
            should_close: false,
            made_sticky: false,
            config: Config::load().unwrap_or_default(),
        }
    }
    
    fn make_window_sticky(&mut self) {
        #[cfg(target_os = "linux")]
        {
            std::thread::spawn(|| {
                // Give the window time to fully initialize
                std::thread::sleep(std::time::Duration::from_millis(500));
                
                // Check session type
                let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
                
                if session_type == "wayland" {
                    println!("Detected Wayland, using KWin DBus scripting...");
                    
                    // Proper KWin script based on official API docs
                    let script = r#"
var clients = workspace.clientList();
for (var i = 0; i < clients.length; i++) {
    var client = clients[i];
    if (client.caption.indexOf("yoinkctl") !== -1 || 
        client.resourceClass.indexOf("yoinkctl") !== -1) {
        client.onAllDesktops = true;
        console.log("Made yoinkctl sticky: " + client.caption);
    }
}
"#;
                    
                    // Save script to temp file (more reliable than inline)
                    let script_path = "/tmp/yoinkctl_sticky.js";
                    std::fs::write(script_path, script).ok();
                    
                    // Try qdbus6 first (KDE 6)
                    let result = Command::new("qdbus6")
                        .args(&[
                            "org.kde.KWin",
                            "/Scripting",
                            "org.kde.kwin.Scripting.loadScript",
                            script_path,
                        ])
                        .output();
                    
                    if result.is_ok() {
                        println!("✓ Script loaded via qdbus6");
                        
                        // Run the script
                        if let Ok(out) = result {
                            if let Ok(script_id) = String::from_utf8(out.stdout) {
                                let script_id = script_id.trim();
                                if !script_id.is_empty() {
                                    Command::new("qdbus6")
                                        .args(&[
                                            "org.kde.KWin",
                                            &format!("/{}", script_id),
                                            "org.kde.kwin.Script.run",
                                        ])
                                        .output()
                                        .ok();
                                    println!("✓ Script executed");
                                }
                            }
                        }
                    } else {
                        // Fallback to qdbus (KDE 5)
                        let result = Command::new("qdbus")
                            .args(&[
                                "org.kde.KWin",
                                "/Scripting",
                                "org.kde.kwin.Scripting.loadScript",
                                script_path,
                            ])
                            .output();
                        
                        if let Ok(out) = result {
                            println!("✓ Script loaded via qdbus");
                            if let Ok(script_id) = String::from_utf8(out.stdout) {
                                let script_id = script_id.trim();
                                if !script_id.is_empty() {
                                    Command::new("qdbus")
                                        .args(&[
                                            "org.kde.KWin",
                                            &format!("/{}", script_id),
                                            "org.kde.kwin.Script.run",
                                        ])
                                        .output()
                                        .ok();
                                    println!("✓ Script executed");
                                }
                            }
                        }
                    }
                    
                    // Clean up
                    std::fs::remove_file(script_path).ok();
                    
                } else {
                    // X11 path
                    println!("Detected X11, using wmctrl...");
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    
                    Command::new("wmctrl")
                        .args(&["-r", "yoinkctl Picker", "-b", "add,sticky"])
                        .output()
                        .ok();
                    
                    let wmctrl_list = Command::new("wmctrl")
                        .arg("-l")
                        .output();
                        
                    if let Ok(output) = wmctrl_list {
                        if let Ok(list) = String::from_utf8(output.stdout) {
                            for line in list.lines() {
                                if line.contains("yoinkctl") {
                                    if let Some(win_id) = line.split_whitespace().next() {
                                        Command::new("wmctrl")
                                            .args(&["-i", "-r", win_id, "-b", "add,sticky"])
                                            .output()
                                            .ok();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    println!("✓ Attempted to make window sticky via wmctrl");
                }
            });
        }
        
        self.made_sticky = true;
    }
    
    fn get_color_at_cursor(&self) -> Option<egui::Color32> {
        let screenshot = self.screenshot.as_ref()?;
        
        let x = (self.cursor_pos.x as i32 + self.screenshot_offset.0).max(0) as u32;
        let y = (self.cursor_pos.y as i32 + self.screenshot_offset.1).max(0) as u32;
        
        if x >= screenshot.width() || y >= screenshot.height() {
            return None;
        }
        
        let pixel = screenshot.get_pixel(x, y);
        Some(egui::Color32::from_rgba_premultiplied(
            pixel[0], pixel[1], pixel[2], 255
        ))
    }
    
    fn copy_to_clipboard(&self, color: egui::Color32) {
        let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
        
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Err(e) = clipboard.set_text(&hex) {
                eprintln!("Failed to copy to clipboard: {}", e);
            } else {
                println!("Copied to clipboard: {}", hex);
            }
        }
    }
    
    // Calculate target offset for magnifier to keep it on screen
    fn calculate_magnifier_offset(&self, mag_size: f32, info_height: f32, screen_rect: egui::Rect) -> egui::Vec2 {
        let margin = 30.0; // X and Y offset when cursor goes down/right
        let small_margin = 10.0; // Y offset when cursor goes up (reduced margin for top positioning)
        let total_height = mag_size + info_height + 10.0; // mag + gap + info box
        
        // Default offset (bottom-right of cursor)
        let mut offset_x = margin;
        let mut offset_y = margin;
        
        // Check right edge (use magnifier_pos for calculations)
        if self.magnifier_pos.x + margin + mag_size > screen_rect.max.x {
            // Move to left side of cursor
            offset_x = -(mag_size + margin);
        }
        
        // Check bottom edge
        if self.magnifier_pos.y + margin + total_height > screen_rect.max.y {
            // Move to top side of cursor with smaller margin
            offset_y = -(mag_size + small_margin);
        }
        
        // Check top edge (when moved up)
        if self.magnifier_pos.y + offset_y < screen_rect.min.y {
            offset_y = -self.magnifier_pos.y + small_margin + screen_rect.min.y;
        }
        
        // Check left edge (when moved left)
        if self.magnifier_pos.x + offset_x < screen_rect.min.x {
            offset_x = -self.magnifier_pos.x + margin + screen_rect.min.x;
        }
        
        egui::vec2(offset_x, offset_y)
    }
}

impl eframe::App for ColorPicker {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Make window sticky after first frame (window needs to exist first)
        if !self.made_sticky {
            self.make_window_sticky();
        }
        
        // Close immediately if flagged
        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        // Hide default cursor
        ctx.set_cursor_icon(egui::CursorIcon::None);
        
        // Get current cursor position
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
            self.cursor_pos = pos;
        }
        
        // Smooth magnifier following with easing and distance clamping
        let max_distance = 150.0; // Maximum distance magnifier can be from cursor
        let smoothing = 0.15; // Lower = smoother but slower, higher = faster but less smooth
        
        let target_pos = self.cursor_pos;
        let current_distance = self.magnifier_pos.distance(target_pos);
        
        // If distance is too large, snap closer
        if current_distance > max_distance {
            let direction = (target_pos - self.magnifier_pos) / current_distance;
            self.magnifier_pos = target_pos - direction * max_distance;
        }
        
        // Smooth interpolation (lerp with easing)
        self.magnifier_pos.x += (target_pos.x - self.magnifier_pos.x) * smoothing;
        self.magnifier_pos.y += (target_pos.y - self.magnifier_pos.y) * smoothing;
        
        // Check for escape key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.should_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        // Check for mouse click
        if ctx.input(|i| i.pointer.primary_clicked()) {
            if let Some(color) = self.get_color_at_cursor() {
                self.copy_to_clipboard(color);
                self.should_close = true;
                // Close immediately and exit the process
                std::process::exit(0);
            }
        }
        
        // Draw fullscreen overlay with NO background
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // Get the actual screen rect from the UI
                let screen_rect = ui.max_rect();
                
                // Draw magnifying glass and info
                if let Some(color) = self.get_color_at_cursor() {
                    let mag_size = self.config.preview_size as f32;
                    
                    // Calculate info height based on what's shown
                    let mut line_count = 0;
                    if self.config.show_hex { line_count += 1; }
                    if self.config.show_rgb { line_count += 1; }
                    if self.config.show_hsl { line_count += 1; }
                    let info_height = if line_count > 0 { 15.0 + (line_count as f32 * 20.0) } else { 0.0 };
                    
                    // Calculate target offset based on screen position
                    let target_offset = self.calculate_magnifier_offset(mag_size, info_height, screen_rect);
                    
                    // Smooth offset interpolation (faster than position for snappier feel)
                    let offset_smoothing = 0.25;
                    self.magnifier_offset.x += (target_offset.x - self.magnifier_offset.x) * offset_smoothing;
                    self.magnifier_offset.y += (target_offset.y - self.magnifier_offset.y) * offset_smoothing;
                    
                    // Apply smoothed offset to magnifier position
                    let mag_pos = self.magnifier_pos + self.magnifier_offset;
                    
                    // Draw zoomed pixels (11x11 grid)
                    let zoom = 5;
                    let pixel_size = mag_size / 11.0;
                    
                    for dy in -zoom..=zoom {
                        for dx in -zoom..=zoom {
                            let px = (self.cursor_pos.x as i32 + dx + self.screenshot_offset.0).max(0) as u32;
                            let py = (self.cursor_pos.y as i32 + dy + self.screenshot_offset.1).max(0) as u32;
                            
                            if let Some(screenshot) = &self.screenshot {
                                if px < screenshot.width() && py < screenshot.height() {
                                    let pixel = screenshot.get_pixel(px, py);
                                    let pixel_color = egui::Color32::from_rgb(pixel[0], pixel[1], pixel[2]);
                                    
                                    let cell_pos = mag_pos + egui::vec2(
                                        (dx + zoom) as f32 * pixel_size,
                                        (dy + zoom) as f32 * pixel_size,
                                    );
                                    
                                    let cell_rect = egui::Rect::from_min_size(
                                        cell_pos,
                                        egui::vec2(pixel_size, pixel_size),
                                    );
                                    
                                    ui.painter().rect_filled(cell_rect, 0.0, pixel_color);
                                    
                                    // Highlight center pixel
                                    if dx == 0 && dy == 0 {
                                        ui.painter().rect_stroke(
                                            cell_rect,
                                            0.0,
                                            egui::Stroke::new(2.0, egui::Color32::RED),
                                        );
                                    }
                                }
                            }
                        }
                    }
                    
                    // White border around magnifier
                    let mag_rect = egui::Rect::from_min_size(mag_pos, egui::vec2(mag_size, mag_size));
                    ui.painter().rect_stroke(
                        mag_rect,
                        4.0,
                        egui::Stroke::new(3.0, egui::Color32::WHITE),
                    );
                    
                    // Text info below magnifier (adjust position based on where magnifier is)
                    if info_height > 0.0 {
                        let info_y = if self.magnifier_offset.y < 0.0 {
                            // Magnifier is above cursor, put info above magnifier
                            mag_pos.y - info_height - 10.0
                        } else {
                            // Magnifier is below cursor, put info below magnifier
                            mag_pos.y + mag_size + 10.0
                        };
                        
                        // Calculate maximum text width
                        let mut max_text_width = 0.0f32;
                        let padding = 16.0;
                        
                        if self.config.show_hex {
                            let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
                            let galley = ui.painter().layout_no_wrap(
                                hex,
                                egui::FontId::monospace(16.0),
                                egui::Color32::WHITE,
                            );
                            max_text_width = max_text_width.max(galley.size().x);
                        }
                        
                        if self.config.show_rgb {
                            let rgb = format!("RGB({}, {}, {})", color.r(), color.g(), color.b());
                            let galley = ui.painter().layout_no_wrap(
                                rgb,
                                egui::FontId::monospace(13.0),
                                egui::Color32::WHITE,
                            );
                            max_text_width = max_text_width.max(galley.size().x);
                        }
                        
                        if self.config.show_hsl {
                            // Convert RGB to HSL for measurement
                            let r = color.r() as f32 / 255.0;
                            let g = color.g() as f32 / 255.0;
                            let b = color.b() as f32 / 255.0;
                            
                            let max_val = r.max(g).max(b);
                            let min_val = r.min(g).min(b);
                            let delta = max_val - min_val;
                            
                            let l = (max_val + min_val) / 2.0;
                            let s = if delta == 0.0 {
                                0.0
                            } else {
                                delta / (1.0 - (2.0 * l - 1.0).abs())
                            };
                            
                            let h = if delta == 0.0 {
                                0.0
                            } else if max_val == r {
                                60.0 * (((g - b) / delta) % 6.0)
                            } else if max_val == g {
                                60.0 * (((b - r) / delta) + 2.0)
                            } else {
                                60.0 * (((r - g) / delta) + 4.0)
                            };
                            
                            let h = if h < 0.0 { h + 360.0 } else { h };
                            let hsl = format!("HSL({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0);
                            
                            let galley = ui.painter().layout_no_wrap(
                                hsl,
                                egui::FontId::monospace(13.0),
                                egui::Color32::WHITE,
                            );
                            max_text_width = max_text_width.max(galley.size().x);
                        }
                        
                        let text_box_width = max_text_width + padding * 2.0;
                        let text_box_x = mag_pos.x + (mag_size - text_box_width) / 2.0;
                        
                        let text_bg = egui::Rect::from_min_size(
                            egui::pos2(text_box_x, info_y),
                            egui::vec2(text_box_width, info_height),
                        );
                        ui.painter().rect_filled(
                            text_bg,
                            4.0,
                            egui::Color32::from_black_alpha(200),
                        );
                        
                        let text_center_x = text_box_x + text_box_width / 2.0;
                        let mut current_y = info_y + 10.0;
                        
                        if self.config.show_hex {
                            let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
                            ui.painter().text(
                                egui::pos2(text_center_x, current_y),
                                egui::Align2::CENTER_TOP,
                                &hex,
                                egui::FontId::monospace(16.0),
                                egui::Color32::WHITE,
                            );
                            current_y += 20.0;
                        }
                        
                        if self.config.show_rgb {
                            let rgb = format!("RGB({}, {}, {})", color.r(), color.g(), color.b());
                            ui.painter().text(
                                egui::pos2(text_center_x, current_y),
                                egui::Align2::CENTER_TOP,
                                &rgb,
                                egui::FontId::monospace(13.0),
                                egui::Color32::from_gray(200),
                            );
                            current_y += 20.0;
                        }
                        
                        if self.config.show_hsl {
                            // Convert RGB to HSL
                            let r = color.r() as f32 / 255.0;
                            let g = color.g() as f32 / 255.0;
                            let b = color.b() as f32 / 255.0;
                            
                            let max = r.max(g).max(b);
                            let min = r.min(g).min(b);
                            let delta = max - min;
                            
                            let l = (max + min) / 2.0;
                            let s = if delta == 0.0 {
                                0.0
                            } else {
                                delta / (1.0 - (2.0 * l - 1.0).abs())
                            };
                            
                            let h = if delta == 0.0 {
                                0.0
                            } else if max == r {
                                60.0 * (((g - b) / delta) % 6.0)
                            } else if max == g {
                                60.0 * (((b - r) / delta) + 2.0)
                            } else {
                                60.0 * (((r - g) / delta) + 4.0)
                            };
                            
                            let h = if h < 0.0 { h + 360.0 } else { h };
                            
                            let hsl = format!("HSL({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0);
                            ui.painter().text(
                                egui::pos2(text_center_x, current_y),
                                egui::Align2::CENTER_TOP,
                                &hsl,
                                egui::FontId::monospace(13.0),
                                egui::Color32::from_gray(200),
                            );
                        }
                    }
                }
                
                // Draw crosshair at cursor
                let crosshair_size = 20.0;
                ui.painter().line_segment(
                    [
                        self.cursor_pos + egui::vec2(-crosshair_size, 0.0),
                        self.cursor_pos + egui::vec2(crosshair_size, 0.0),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
                ui.painter().line_segment(
                    [
                        self.cursor_pos + egui::vec2(0.0, -crosshair_size),
                        self.cursor_pos + egui::vec2(0.0, crosshair_size),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                );
            });
        
        // Request repaint for smooth cursor tracking
        ctx.request_repaint();
    }
}

fn capture_all_screens() -> (Option<RgbaImage>, (i32, i32)) {
    // Try to capture the primary monitor
    if let Ok(monitors) = Monitor::all() {
        if let Some(monitor) = monitors.first() {
            if let Ok(image) = monitor.capture_image() {
                // Get the monitor position offset
                let x_offset = monitor.x();
                let y_offset = monitor.y();
                return (Some(image), (x_offset, y_offset));
            }
        }
    }
    (None, (0, 0))
}