use std::sync::{Arc, Barrier};
use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui_window_glfw_passthrough;
use egui::{Context, Color32, Stroke, Pos2, Order, Sense, Vec2};
use crossbeam::channel::Receiver;

use sdk::Player::Player;

pub fn run_overlay(player_receiver: Receiver<Vec<Player>>, barrier: Arc<Barrier>) {
 
    egui_overlay::start(Render { player_receiver, barrier });
}

pub struct Render {
    pub player_receiver: Receiver<Vec<Player>>,
    pub barrier: Arc<Barrier>
}

impl Render {    
    fn esp_overlay(&self, egui_context: &Context) {
        egui::Area::new("overlay")
            .interactable(false)
            .fixed_pos(Pos2::new(0.,0.))
            .order(Order::Background)
            .show(egui_context, |ui| {
                ui.allocate_at_least(Vec2::new(1024.0, 768.0), Sense { focusable: false, drag: false, click: false });
                let painter = ui.painter();
                if let Ok(players) = self.player_receiver.recv() {
                    for player in players.iter() {
                        self.draw_nametags(player, ui, painter);
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
        let font_id = egui::FontId::proportional(16.0);
        let name_layout = ui.fonts(|fonts| fonts.layout_no_wrap(name.clone(), font_id.clone(), Color32::WHITE));
        let name_size = name_layout.size();

        painter.text(
            Pos2::new(player.position_2d.x - (name_size.x / 2.0) , player.position_2d.y - 20.0),
            egui::Align2::LEFT_TOP,
            name,
            font_id.clone(),
            Color32::WHITE,
        );

        let health_text = format!("{}", player.health);
        let health_layout = ui.fonts(|fonts| fonts.layout_no_wrap(health_text.clone(), font_id.clone(), Color32::WHITE));
        let health_size = health_layout.size(); // Get health text size

        let red = (2.0 * (100.0 - player.health as f32) / 100.0).min(1.0);
        let green = (2.0 * player.health as f32 / 100.0).min(1.0);
        let color = Color32::from_rgb((red * 255.0) as u8, (green * 255.0) as u8, 0);

        painter.text(
            Pos2::new(
                player.position_2d.x + (name_size.x / 2.0) + 2.0,
                player.position_2d.y - 20.0,
                ),
            egui::Align2::LEFT_TOP,
            health_text,
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
        glfw_backend.window.set_pos(0, 35);
        glfw_backend.window.set_size(1024, 768);

        /* 
        egui::Window::new("controls").show(egui_context, |ui| {
            ui.set_width(300.0);
        }); */

        self.esp_overlay(egui_context);

        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.set_passthrough(false);
        } else {
            glfw_backend.set_passthrough(true)
        }
        egui_context.request_repaint();
    }
}