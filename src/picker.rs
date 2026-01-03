use eframe::egui;
use xcap::Monitor;
use image::RgbaImage;
use arboard::Clipboard;
use crate::config::Config;
use crate::history::ColorHistory;

pub struct ColorPicker {
    screenshot: Option<RgbaImage>,
    screenshot_offset: (i32, i32),
    cursor_pos: egui::Pos2,
    magnifier_pos: egui::Pos2,
    magnifier_offset: egui::Vec2,
    should_close: bool,
    config: Config,
    initialized: bool,
}

impl ColorPicker {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (screenshot, offset) = capture_all_screens();
        
        Self {
            screenshot,
            screenshot_offset: offset,
            cursor_pos: egui::Pos2::ZERO,
            magnifier_pos: egui::Pos2::ZERO,
            magnifier_offset: egui::vec2(30.0, 30.0),
            should_close: false,
            config: Config::load().unwrap_or_default(),
            initialized: false,
        }
    }
    
    // OPTIMIZED: Accept pre-captured screenshot to avoid duplicate capture
    pub fn new_with_screenshot(_cc: &eframe::CreationContext<'_>, screenshot: Option<RgbaImage>, offset: (i32, i32)) -> Self {
        Self {
            screenshot,
            screenshot_offset: offset,
            cursor_pos: egui::Pos2::ZERO,
            magnifier_pos: egui::Pos2::ZERO,
            magnifier_offset: egui::vec2(30.0, 30.0),
            should_close: false,
            config: Config::load().unwrap_or_default(),
            initialized: false,
        }
    }
    
    // OPTIMIZED: Accept pre-loaded config AND screenshot for fastest startup
    pub fn new_with_config(_cc: &eframe::CreationContext<'_>, screenshot: Option<RgbaImage>, offset: (i32, i32), config: Config) -> Self {
        Self {
            screenshot,
            screenshot_offset: offset,
            cursor_pos: egui::Pos2::ZERO,
            magnifier_pos: egui::Pos2::ZERO,
            magnifier_offset: egui::vec2(30.0, 30.0),
            should_close: false,
            config,
            initialized: false,
        }
    }
    
    #[inline]
    fn get_color_at_cursor(&self) -> Option<egui::Color32> {
        let screenshot = self.screenshot.as_ref()?;
        
        let x = (self.cursor_pos.x as i32 + self.screenshot_offset.0).max(0) as u32;
        let y = (self.cursor_pos.y as i32 + self.screenshot_offset.1).max(0) as u32;
        
        if x >= screenshot.width() || y >= screenshot.height() {
            return None;
        }
        
        let pixel = screenshot.get_pixel(x, y);
        Some(egui::Color32::from_rgba_premultiplied(pixel[0], pixel[1], pixel[2], 255))
    }
    
    // OPTIMIZED: Non-blocking clipboard operations
    fn copy_to_clipboard(&self, color: egui::Color32) {
        let hex = format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b());
        
        // Spawn background thread for all I/O operations
        let hex_clone = hex.clone();
        let color_rgb = (color.r(), color.g(), color.b());
        
        std::thread::spawn(move || {
            // Save to history
            match ColorHistory::load() {
                Ok(mut history) => {
                    history.add_color(hex_clone.clone(), color_rgb);
                }
                Err(_) => {
                    let mut history = ColorHistory::default();
                    history.add_color(hex_clone.clone(), color_rgb);
                }
            }
            
            // Copy to clipboard
            if let Ok(mut clipboard) = Clipboard::new() {
                clipboard.set_text(&hex_clone).ok();
            }
        });
    }
    
    #[inline]
    fn calculate_magnifier_offset(&self, mag_size: f32, info_height: f32, screen_rect: egui::Rect) -> egui::Vec2 {
        let margin = 30.0;
        let small_margin = 10.0;
        let total_height = mag_size + info_height + 10.0;
        
        let mut offset_x = margin;
        let mut offset_y = margin;
        
        // Adjust horizontal position
        if self.magnifier_pos.x + margin + mag_size > screen_rect.max.x {
            offset_x = -(mag_size + margin);
        }
        if self.magnifier_pos.x + offset_x < screen_rect.min.x {
            offset_x = -self.magnifier_pos.x + margin + screen_rect.min.x;
        }
        
        // Adjust vertical position
        if self.magnifier_pos.y + margin + total_height > screen_rect.max.y {
            offset_y = -(mag_size + small_margin);
        }
        if self.magnifier_pos.y + offset_y < screen_rect.min.y {
            offset_y = -self.magnifier_pos.y + small_margin + screen_rect.min.y;
        }
        
        egui::vec2(offset_x, offset_y)
    }

    fn draw_magnifier(&self, ui: &mut egui::Ui, mag_pos: egui::Pos2, mag_size: f32) {
        let zoom = 5;
        let pixel_size = mag_size / 11.0;
        let mag_rect = egui::Rect::from_min_size(mag_pos, egui::vec2(mag_size, mag_size));
        
        // Draw blurred shadow FIRST (before content)
        draw_blurred_shadow(ui, mag_rect, 4.0, 20.0, egui::vec2(4.0, 4.0));
        
        // Draw magnifier content
        if let Some(screenshot) = &self.screenshot {
            // OPTIMIZED: Pre-calculate bounds to reduce repeated calculations
            let center_x = self.cursor_pos.x as i32 + self.screenshot_offset.0;
            let center_y = self.cursor_pos.y as i32 + self.screenshot_offset.1;
            let width = screenshot.width() as i32;
            let height = screenshot.height() as i32;
            
            for dy in -zoom..=zoom {
                for dx in -zoom..=zoom {
                    let px = (center_x + dx).clamp(0, width - 1) as u32;
                    let py = (center_y + dy).clamp(0, height - 1) as u32;
                    
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
        
        // Draw white border on top
        ui.painter().rect_stroke(
            mag_rect,
            4.0,
            egui::Stroke::new(3.0, egui::Color32::WHITE),
        );
    }

    fn draw_color_info(&self, ui: &mut egui::Ui, color: egui::Color32, mag_pos: egui::Pos2, mag_size: f32, info_height: f32) {
        let padding = 16.0;
        
        // OPTIMIZED: Pre-allocate with exact capacity
        let format_count = self.config.show_hex as usize + self.config.show_rgb as usize + self.config.show_hsl as usize;
        let mut formats = Vec::with_capacity(format_count);
        
        if self.config.show_hex {
            formats.push((
                format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b()),
                egui::FontId::monospace(16.0),
                egui::Color32::WHITE,
            ));
        }
        if self.config.show_rgb {
            formats.push((
                format!("RGB({}, {}, {})", color.r(), color.g(), color.b()),
                egui::FontId::monospace(13.0),
                egui::Color32::from_gray(200),
            ));
        }
        if self.config.show_hsl {
            let (h, s, l) = rgb_to_hsl(color.r(), color.g(), color.b());
            formats.push((
                format!("HSL({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0),
                egui::FontId::monospace(13.0),
                egui::Color32::from_gray(200),
            ));
        }
        
        if formats.is_empty() {
            return;
        }
        
        // Find max width
        let max_text_width = formats.iter()
            .map(|(text, font, _)| ui.painter().layout_no_wrap(text.clone(), font.clone(), egui::Color32::WHITE).size().x)
            .fold(0.0f32, f32::max);
        
        let text_box_width = max_text_width + padding * 2.0;
        let text_box_x = mag_pos.x + (mag_size - text_box_width) / 2.0;
        
        let info_y = if self.magnifier_offset.y < 0.0 {
            mag_pos.y - info_height - 10.0
        } else {
            mag_pos.y + mag_size + 10.0
        };
        
        let text_bg = egui::Rect::from_min_size(
            egui::pos2(text_box_x, info_y),
            egui::vec2(text_box_width, info_height),
        );
        
        // Draw blurred shadow
        draw_blurred_shadow(ui, text_bg, 4.0, 15.0, egui::vec2(3.0, 3.0));
        
        // Draw background
        ui.painter().rect_filled(
            text_bg,
            4.0,
            egui::Color32::from_black_alpha(200),
        );
        
        // Draw text
        let text_center_x = text_box_x + text_box_width / 2.0;
        let mut current_y = info_y + 10.0;
        
        for (text, font, color) in formats {
            ui.painter().text(
                egui::pos2(text_center_x, current_y),
                egui::Align2::CENTER_TOP,
                text,
                font,
                color,
            );
            current_y += 20.0;
        }
    }

    fn draw_crosshair(&self, ui: &mut egui::Ui) {
        let crosshair_size = 20.0;
        let shadow_layers = 12;
        
        // OPTIMIZED: Pre-calculate line endpoints
        let h_start = self.cursor_pos + egui::vec2(-crosshair_size, 0.0);
        let h_end = self.cursor_pos + egui::vec2(crosshair_size, 0.0);
        let v_start = self.cursor_pos + egui::vec2(0.0, -crosshair_size);
        let v_end = self.cursor_pos + egui::vec2(0.0, crosshair_size);
        
        // Blurred shadow for horizontal line
        for i in 0..shadow_layers {
            let progress = i as f32 / shadow_layers as f32;
            let alpha = (200.0 * (1.0 - progress.powf(0.8))) as u8;
            let stroke_width = 3.0 + progress * 4.0;
            
            ui.painter().line_segment(
                [h_start, h_end],
                egui::Stroke::new(stroke_width, egui::Color32::from_black_alpha(alpha / shadow_layers as u8)),
            );
        }
        
        // Blurred shadow for vertical line
        for i in 0..shadow_layers {
            let progress = i as f32 / shadow_layers as f32;
            let alpha = (200.0 * (1.0 - progress.powf(0.8))) as u8;
            let stroke_width = 3.0 + progress * 4.0;
            
            ui.painter().line_segment(
                [v_start, v_end],
                egui::Stroke::new(stroke_width, egui::Color32::from_black_alpha(alpha / shadow_layers as u8)),
            );
        }
        
        // Crosshair
        let crosshair_color = egui::Color32::WHITE;
        ui.painter().line_segment([h_start, h_end], egui::Stroke::new(2.0, crosshair_color));
        ui.painter().line_segment([v_start, v_end], egui::Stroke::new(2.0, crosshair_color));
    }

    #[inline]
    fn update_cursor_position(&mut self, ctx: &egui::Context) {
        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos().or(i.pointer.latest_pos())) {
            self.cursor_pos = pos;
            if !self.initialized {
                self.initialized = true;
            }
        }
    }

    #[inline]
    fn update_magnifier_position(&mut self) {
        const MAX_DISTANCE: f32 = 150.0;
        const SMOOTHING: f32 = 0.15;
        
        let target_pos = self.cursor_pos;
        let current_distance = self.magnifier_pos.distance(target_pos);
        
        // Clamp to max distance
        if current_distance > MAX_DISTANCE {
            let direction = (target_pos - self.magnifier_pos) / current_distance;
            self.magnifier_pos = target_pos - direction * MAX_DISTANCE;
        }
        
        // Smooth movement
        self.magnifier_pos.x += (target_pos.x - self.magnifier_pos.x) * SMOOTHING;
        self.magnifier_pos.y += (target_pos.y - self.magnifier_pos.y) * SMOOTHING;
    }

    #[inline]
    fn handle_input(&mut self, ctx: &egui::Context) -> bool {
        // OPTIMIZED: Check click first (more common action)
        if ctx.input(|i| i.pointer.primary_clicked()) {
            if let Some(color) = self.get_color_at_cursor() {
                self.copy_to_clipboard(color);
                return true;
            }
        }
        
        // Check for escape key
        ctx.input(|i| i.key_pressed(egui::Key::Escape))
    }
}

impl eframe::App for ColorPicker {
    #[inline]
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.should_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        ctx.set_cursor_icon(egui::CursorIcon::None);
        
        // Update positions
        self.update_cursor_position(ctx);
        self.update_magnifier_position();
        
        // Handle input
        if self.handle_input(ctx) {
            self.should_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                let screen_rect = ui.max_rect();
                
                if let Some(color) = self.get_color_at_cursor() {
                    let mag_size = self.config.preview_size as f32;
                    
                    // OPTIMIZED: Calculate line count using boolean arithmetic
                    let line_count = self.config.show_hex as usize 
                        + self.config.show_rgb as usize 
                        + self.config.show_hsl as usize;
                    let info_height = if line_count > 0 { 
                        15.0 + (line_count as f32 * 20.0) 
                    } else { 
                        0.0 
                    };
                    
                    // Calculate and smooth magnifier offset
                    let target_offset = self.calculate_magnifier_offset(mag_size, info_height, screen_rect);
                    const OFFSET_SMOOTHING: f32 = 0.25;
                    self.magnifier_offset.x += (target_offset.x - self.magnifier_offset.x) * OFFSET_SMOOTHING;
                    self.magnifier_offset.y += (target_offset.y - self.magnifier_offset.y) * OFFSET_SMOOTHING;
                    
                    let mag_pos = self.magnifier_pos + self.magnifier_offset;
                    
                    // Draw components
                    self.draw_magnifier(ui, mag_pos, mag_size);
                    
                    if info_height > 0.0 {
                        self.draw_color_info(ui, color, mag_pos, mag_size, info_height);
                    }
                }
                
                self.draw_crosshair(ui);
            });
        
        ctx.request_repaint();
    }
}

// Helper function to draw a blurred shadow
fn draw_blurred_shadow(ui: &mut egui::Ui, rect: egui::Rect, rounding: f32, blur_radius: f32, offset: egui::Vec2) {
    let shadow_layers = 16;
    let max_alpha = 160;
    
    for i in 0..shadow_layers {
        let progress = i as f32 / shadow_layers as f32;
        let current_blur = blur_radius * progress;
        let alpha = (max_alpha as f32 * (1.0 - progress.powf(0.7))) as u8;
        
        let expanded_rect = rect.expand(current_blur).translate(offset);
        
        ui.painter().rect_filled(
            expanded_rect,
            rounding + current_blur * 0.2,
            egui::Color32::from_black_alpha(alpha / shadow_layers as u8),
        );
    }
}

// Helper function to convert RGB to HSL
#[inline]
fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    
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
    
    (h, s, l)
}

pub fn capture_all_screens() -> (Option<RgbaImage>, (i32, i32)) {
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