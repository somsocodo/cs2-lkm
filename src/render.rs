use std::sync::{Arc, Mutex};
use std::thread;
use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui_window_glfw_passthrough;
use egui::{Color32, Stroke, Pos2};
use sdk::Player::Player;

pub fn run_overlay(players: Arc<Mutex<Vec<Player>>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        egui_overlay::start(Render { frame: 0, players });
    })
}

pub struct Render {
    pub frame: u64,
    pub players: Arc<Mutex<Vec<Player>>>
}

impl EguiOverlay for Render {
    fn gui_run(
        &mut self,
        egui_context: &egui::Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {
        egui::Window::new("controls").show(egui_context, |ui| {
            ui.set_width(300.0);
            self.frame += 1;

            ui.label(format!(
                "passthrough: {}",
                glfw_backend.window.is_mouse_passthrough()
            ));

            let players_lock = self.players.lock().unwrap();
            for player in players_lock.iter() {
                println!("Rendering player: {} | ({}, {})", player.name.to_str(), player.position_2d.x, player.position_2d.y);
                let painter = ui.painter();
                painter.circle_stroke(Pos2::new(player.position_2d.x, player.position_2d.y), 2.0, Stroke::new(2.0, Color32::RED));
            }
        });

        if egui_context.wants_pointer_input() || egui_context.wants_keyboard_input() {
            glfw_backend.set_passthrough(false);
        } else {
            glfw_backend.set_passthrough(true)
        }
        egui_context.request_repaint();
    }
}