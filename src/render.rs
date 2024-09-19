use std::sync::{Arc, Barrier};
use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui_window_glfw_passthrough;
use egui::{Context, Color32, Pos2, Order, Sense, Vec2, FontFamily::Monospace};
use crossbeam::channel::Receiver;

use config::SharedConfig;
use sdk::Player::Player;

pub fn run_overlay(player_receiver: Receiver<Vec<Player>>, barrier: Arc<Barrier>, config: SharedConfig) {
    egui_overlay::start(Render { 
        player_receiver, 
        barrier,
        config
    });
}

pub struct Render {
    pub player_receiver: Receiver<Vec<Player>>,
    pub barrier: Arc<Barrier>,
    pub config: SharedConfig
}

impl Render {    
    fn esp_overlay(&self, egui_context: &Context, nametags: bool) {
        egui::Area::new("overlay")
            .interactable(false)
            .fixed_pos(Pos2::new(0.,0.))
            .order(Order::Background)
            .show(egui_context, |ui| {
                ui.allocate_at_least(Vec2::new(1024.0, 768.0), Sense { focusable: false, drag: false, click: false });
                let painter = ui.painter();
                if let Ok(players) = self.player_receiver.recv() {
                    for player in players.iter() {
                        if nametags {
                            self.draw_nametags(player, ui, painter);
                        }
                    }
                }
            });
            self.barrier.wait();
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
        glfw_backend.window.set_pos(0, 0);
        glfw_backend.window.set_size(1024, 768);

        let mut config = self.config.lock().unwrap();

        if config.show_gui {
            if !glfw_backend.focused {
                glfw_backend.window.focus();
            }
            
            egui::Window::new("gui")
            .resizable(false)
            .show(egui_context, |ui| {
                ui.set_width(500.0);
                ui.set_height(200.0);

                ui.checkbox(&mut config.nametags, "Nametags");
            });
        }
        
        self.esp_overlay(egui_context, config.nametags);

        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.set_passthrough(false);
        } else {
            glfw_backend.set_passthrough(true)
        }
        egui_context.request_repaint();
    }
}