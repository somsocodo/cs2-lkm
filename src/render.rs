use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui_window_glfw_passthrough;
use egui::{Context, Color32, Pos2, Order, Sense, Vec2, FontFamily::Monospace};
use crossbeam::channel::Receiver;

use config::{SharedConfig, Config};
use sdk::Player::Player;

use crate::config::SharedKeyState;

pub fn run_overlay(player_receiver: Receiver<Vec<Player>>, shared_keystate: SharedKeyState, shared_config: SharedConfig) {
    
    egui_overlay::start(Render { 
        player_receiver,
        shared_keystate,
        shared_config,
        config: Config::new()
    });
}

pub struct Render {
    pub player_receiver: Receiver<Vec<Player>>,
    pub shared_keystate: SharedKeyState,
    pub shared_config: SharedConfig,
    pub config: Config
}

impl Render {    
    fn esp_overlay(&self, egui_context: &Context) {
        egui::Area::new("overlay")
            .interactable(false)
            .fixed_pos(Pos2::new(0.,0.))
            .order(Order::Background)
            .show(egui_context, |ui| {
                ui.allocate_at_least(Vec2::new(self.config.window_size.0 as f32, self.config.window_size.1 as f32), Sense { focusable: false, drag: false, click: false });
                let painter = ui.painter();
                if let Ok(players) = self.player_receiver.recv() {
                    for player in players.iter() {
                        if self.config.esp_hitboxes {
                            self.draw_hitboxes(player, ui, painter);
                        }
                        if self.config.esp_bones {
                            self.draw_bones(player, painter);
                        }
                        if self.config.esp_nametags {
                            self.draw_nametags(player, ui, painter);
                        }      
                    }
                }
            });
    }

    fn draw_hitboxes(&self, player: &Player, ui: &egui::Ui, painter: &egui::Painter) {
        for hitbox in player.hitboxes.iter() {            
            let bb_min = hitbox.min_bounds_2d;
            let bb_max = hitbox.max_bounds_2d;
    
            if(bb_min.x < 0.0 || bb_min.y < 0.0) && (bb_max.x < 0.0 || bb_max.y < 0.0){
                continue;
            }

            let bb_min_pos = egui::Pos2::new(bb_min.x, bb_min.y);
            let bb_max_pos = egui::Pos2::new(bb_max.x, bb_max.y);
    
            if (bb_min.x - bb_max.x).abs() > ui.available_rect_before_wrap().width() as f32 {
                continue;
            }
    
            let mut color = egui::Color32::from_rgba_premultiplied(200, 0, 0, 1); 
            if player.bspotted {
                color = egui::Color32::from_rgba_premultiplied(0, 200, 0, 1); 
            }
            
            let radius_min = hitbox.min_rad_2d;
            let radius_max = hitbox.max_rad_2d;
    
            painter.circle_filled(bb_max_pos, radius_min, color);
            painter.circle_filled(bb_min_pos, radius_max, color);
            painter.line_segment([bb_min_pos, bb_max_pos], (radius_min * 2.0, color));
        }
    }

    fn draw_bones(&self, player: &Player, painter: &egui::Painter) {
        let bone_hierarchy = vec![
            (0, 1),  // Head to Neck
            (1, 2),  // Neck to Spine 3
            (2, 3),  // Spine 3 to Spine 2
            (3, 4),  // Spine 2 to Spine 1
            (4, 5),  // Spine 1 to Spine 0
            (5, 6),  // Spine 0 to Hip
    
            // Left arm
            (1, 7),  // Neck to Left Shoulder
            (7, 8),  // Left Shoulder to Left Arm
            (8, 9),  // Left Arm to Left Hand
    
            // Right arm
            (1, 10), // Neck to Right Shoulder
            (10, 11), // Right Shoulder to Right Arm
            (11, 12), // Right Arm to Right Hand
    
            // Left leg
            (6, 13), // Hip to Left Hip
            (13, 14), // Left Hip to Left Knee
            (14, 15), // Left Knee to Left Foot
    
            // Right leg
            (6, 16),  // Hip to Right Hip
            (16, 17), // Right Hip to Right Knee
            (17, 18), // Right Knee to Right Foot
        ];
    
        let color = egui::Color32::from_rgb(255, 255, 255);

        if player.hitboxes.len() > 0 {
            let head_position = player.bones_2d[0];
            let head_radius = player.hitboxes[0].max_rad_2d / 1.5;
    
            if head_position.x != -99.0 && head_position.y != -99.0 {
                painter.circle_stroke(
                    egui::Pos2::new(head_position.x, head_position.y), 
                    head_radius, 
                    (2.0, color)
                );
            }
        }
    
        for &(start_idx, end_idx) in bone_hierarchy.iter() {
            if start_idx >= player.bones_2d.len() || end_idx >= player.bones_2d.len() {
                continue;
            }
    
            let bone_start = player.bones_2d[start_idx];
            let bone_end = player.bones_2d[end_idx];
    
            if (bone_start.x == -99.0 && bone_start.y == -99.0) || 
                (bone_end.x == -99.0 && bone_end.y == -99.0) ||
                (bone_start.x == 0.0 && bone_start.y == 0.0) ||
                (bone_end.x == 0.0 && bone_end.y == 0.0) {
                continue;
            }
    
            painter.line_segment(
                [egui::Pos2::new(bone_start.x, bone_start.y), egui::Pos2::new(bone_end.x, bone_end.y)], 
                (2.0, color)
            );
        }
    }

    fn draw_nametags(&self, player: &Player, ui: &egui::Ui, painter: &egui::Painter) {
        fn format_name(name: &str) -> String {
            let trimmed_name = name.trim();
        
            if trimmed_name.len() > 7 {
                trimmed_name.chars().take(7).collect::<String>() + "..."
            } else {
                trimmed_name.to_string()
            }
        }

        let name = format_name(&player.name.to_str());
        let font_id = egui::FontId::new(13.0, Monospace);
        let name_layout = ui.fonts(|fonts| fonts.layout_no_wrap(name.clone(), font_id.clone(), Color32::WHITE));
        let name_size = name_layout.size();

        let health_str = player.health.to_string();

        let health_layout = ui.fonts(|fonts| fonts.layout_no_wrap(health_str.clone(), font_id.clone(), Color32::WHITE));
        let health_size = health_layout.size();

        let box_padding = 3.0;
        let box_width = name_size.x + health_size.x + 2.0 + box_padding * 2.0;
        let box_height = name_size.y + box_padding * 2.0;
    
        let box_pos = Pos2::new(
            player.pos_2d.x - box_width / 2.0, 
            player.pos_2d.y - (name_size.y / 2.0) - 13.0);

        painter.rect_filled(
            egui::Rect::from_min_size(box_pos, egui::Vec2::new(box_width, box_height)),
            1.0, // Rounding for the corners
            Color32::from_rgba_unmultiplied(70, 70, 70, 150)
        );

        painter.text(
            Pos2::new(
                player.pos_2d.x - (name_size.x / 2.0) - (health_size.x / 2.0), 
                player.pos_2d.y - (name_size.y / 2.0) - 10.0),
            egui::Align2::LEFT_TOP,
            name,
            font_id.clone(),
            Color32::WHITE,
        );

        let red = (2.0 * (100.0 - player.health as f32) / 100.0).min(1.0);
        let green = (2.0 * player.health as f32 / 100.0).min(1.0);
        let color = Color32::from_rgb((red * 255.0) as u8, (green * 255.0) as u8, 0);

        painter.text(
            Pos2::new(
                player.pos_2d.x + (name_size.x / 2.0) - (health_size.x / 2.0) + 2.0,
                player.pos_2d.y - (name_size.y / 2.0) - 10.0,
                ),
            egui::Align2::LEFT_TOP,
            player.health,
            font_id.clone(),
            color,
        );
    }
}



impl EguiOverlay for Render {
    fn gui_run(
        &mut self,
        egui_context: &Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        let keystate = self.shared_keystate.read().unwrap();
        
        glfw_backend.window.set_pos(0, 35); //35 for cs2 windowed
        glfw_backend.window.set_size(self.config.window_size.0, self.config.window_size.1);

        if keystate.show_gui {
            glfw_backend.set_passthrough(false);
            let mut edit_config = self.config;
            if !glfw_backend.focused {
                glfw_backend.window.focus();
            }
            egui::Window::new("modules")
            .resizable(true)
            .show(egui_context, |ui| {
                ui.checkbox(&mut edit_config.gui_visuals, "gui_visuals");
                ui.checkbox(&mut edit_config.gui_combat, "gui_combat");
            });
            if edit_config.gui_visuals{
                egui::Window::new("visuals")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.checkbox(&mut edit_config.esp_nametags, "esp_nametags");
                    ui.checkbox(&mut edit_config.esp_hitboxes, "esp_hitboxes");
                    ui.checkbox(&mut edit_config.esp_bones, "esp_bones");
                });
            }
            if edit_config.gui_combat{
                egui::Window::new("combat")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.checkbox(&mut edit_config.aim_enabled, "aim_enabled");
                    ui.checkbox(&mut edit_config.trigger_enabled, "trigger_enabled");
                });
            }
            if edit_config.gui_misc{
                egui::Window::new("misc")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.checkbox(&mut edit_config.ignore_team, "ignore_team");
                });
            }
            if edit_config != self.config {
                self.config = edit_config;
                let mut new_config = self.shared_config.write().unwrap();
                *new_config = edit_config;
            }
        } else {
            glfw_backend.set_passthrough(true);
        }
        
        self.esp_overlay(egui_context);

        egui_context.request_repaint();
    }
}