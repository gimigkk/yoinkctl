use eframe::egui;
use xcap::Monitor;
use image::RgbaImage;
use arboard::Clipboard;
use std::process::Command;

pub struct ColorPicker {
    screenshot: Option<RgbaImage>,
    screenshot_offset: (i32, i32),
    cursor_pos: egui::Pos2,
    should_close: bool,
    made_sticky: bool,
}

impl ColorPicker {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (screenshot, offset) = capture_all_screens();
        
        Self {
            screenshot,
            screenshot_offset: offset,
            cursor_pos: egui::Pos2::ZERO,
            should_close: false,
            made_sticky: false,
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
                // Draw magnifying glass and info
                if let Some(color) = self.get_color_at_cursor() {
                    let mag_size = 150.0;
                    let mag_offset = egui::vec2(25.0, 25.0);
                    let mag_pos = self.cursor_pos + mag_offset;
                    
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
                    
                    // Text info below magnifier
                    let info_y = mag_pos.y + mag_size + 10.0;
                    
                    let text_bg = egui::Rect::from_min_size(
                        egui::pos2(mag_pos.x, info_y),
                        egui::vec2(mag_size, 45.0),
                    );
                    ui.painter().rect_filled(
                        text_bg,
                        4.0,
                        egui::Color32::from_black_alpha(200),
                    );
                    
                    let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
                    let rgb = format!("RGB({}, {}, {})", color.r(), color.g(), color.b());
                    
                    let text_center_x = mag_pos.x + mag_size / 2.0;
                    
                    ui.painter().text(
                        egui::pos2(text_center_x, info_y + 10.0),
                        egui::Align2::CENTER_TOP,
                        &hex,
                        egui::FontId::monospace(18.0),
                        egui::Color32::WHITE,
                    );
                    
                    ui.painter().text(
                        egui::pos2(text_center_x, info_y + 30.0),
                        egui::Align2::CENTER_TOP,
                        &rgb,
                        egui::FontId::monospace(13.0),
                        egui::Color32::from_gray(200),
                    );
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