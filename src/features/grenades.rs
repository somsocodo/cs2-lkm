use driver::Driver;
use config::SharedConfig;
use sdk::Vector::{ Vector2, Vector3};
use sdk::Vector::vec_translate;
use crate::sdk::Player::{ SharedPlayerBase, PlayerBase };
use render::Render;
use sdk::WeaponClass::{get_grenade_class, GrenadeClass};

use egui::{ Color32 , Pos2, FontId, FontFamily };

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;

pub struct Grenade {
    pub name: String,
    pub action: String,
    pub grenade_class: GrenadeClass,
    pub pos: Vector3,
    pub throw_pos: Vector3
}

pub struct GrenadeHelper {
    driver: Driver,
    local_player: SharedPlayerBase,
    pub grenades: Vec<Grenade>
}

impl GrenadeHelper {
    pub fn new(driver: Driver, local_player: SharedPlayerBase) -> Self {
        Self {
            driver,
            local_player,
            grenades: Vec::new()
        }
    }

    pub fn save(&mut self, name: String, action: String){
        let local_player: PlayerBase = {
            let local_player_read = self.local_player.read().unwrap();
            local_player_read.clone()
        };

        let grenade_class = get_grenade_class(&self.driver, local_player.pawn);

        if grenade_class == GrenadeClass::Invalid {
            println!("invalid grenade class!");
            return;
        }

        let pos: Vector3 = self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::m_vOldOrigin);
        let eye_pos: Vector3 = pos + self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BaseModelEntity::m_vecViewOffset);
        let view_angle: Vector2 = self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::v_angle);
        let throw_pos: Vector3 = vec_translate(&eye_pos, &view_angle, 2000.0);

        let grenade = Grenade {
            name,
            action,
            grenade_class,
            pos,
            throw_pos,
        };

        println!("saving grenade {}", grenade.name);
        self.grenades.push(grenade);
        
    }

    pub fn draw(
        &self,
        ui: &egui::Ui,
        shared_config: SharedConfig,
        grenade: &Grenade,
        view_matrix: [[f32; 4]; 4]
    ) {
        let local_player: PlayerBase = {
            let local_player_read = self.local_player.read().unwrap();
            local_player_read.clone()
        };
    
        let player_pos: Vector3 = self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::m_vOldOrigin);
        let distance = (player_pos - grenade.pos).length();
    
        let max_distance = 400.0;
        let alpha = ((max_distance - distance) / max_distance * 255.0).clamp(0.0, 255.0) as u8;
    
        let mut fill_colour = egui::Color32::from_rgba_unmultiplied(128, 128, 128, alpha);
        let stroke_colour = egui::Color32::from_rgba_unmultiplied(70, 70, 70, alpha);
        if distance < max_distance {
            let font_id_text = FontId::new(10.0, FontFamily::Monospace);
            let font_id_icon = FontId::new(20.0, FontFamily::Name("icons".into()));
            let painter: &egui::Painter = ui.painter();
        
            if distance < 2.0 {
                let config = {
                    let config_read = shared_config.read().unwrap();
                    config_read.clone()
                };
        
                let window_center = (config.window_size.0 as f32 / 2.0, config.window_size.1 as f32 / 2.0);
                let throw_pos_2d = grenade.throw_pos.world_to_screen(view_matrix);
        
                let line_distance = ((window_center.0 - throw_pos_2d.x).powi(2) + (window_center.1 - throw_pos_2d.y).powi(2)).sqrt();
        
                fill_colour = egui::Color32::from_rgba_unmultiplied(255, 165, 0, alpha); // Orange
        
                if line_distance < 5.0 {
                    fill_colour = egui::Color32::from_rgba_unmultiplied(20, 128, 20, alpha); // Green
                }
        
                let line_start = Pos2::new(window_center.0, window_center.1);
                let line_end = Pos2::new(throw_pos_2d.x, throw_pos_2d.y);
                let stroke = egui::Stroke::new(2.0, fill_colour);
        
                painter.line_segment([line_start, line_end], stroke);
                painter.circle_filled(Pos2::new(throw_pos_2d.x, throw_pos_2d.y), 2.0, fill_colour);
                
                Render::text_shadow(painter, Pos2::new(throw_pos_2d.x, throw_pos_2d.y), egui::Align2::LEFT_BOTTOM, &grenade.action, Color32::WHITE, &font_id_text);
            }
        
            Render::draw_circle(ui, grenade.pos, view_matrix, 12.0, true, fill_colour, stroke_colour, 3.0);
            Render::draw_circle(ui, grenade.pos, view_matrix, 0.5, true, Color32::WHITE, stroke_colour, 0.0);
        
            if distance > 10.0 {
                let text_pos = Vector3 { x: grenade.pos.x, y: grenade.pos.y, z: grenade.pos.z + 15.0 };   
                let text_pos2d = text_pos.world_to_screen(view_matrix);
                let text_colour = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
                Render::text_shadow(painter, Pos2::new(text_pos2d.x, text_pos2d.y), egui::Align2::CENTER_TOP, &grenade.name, text_colour, &font_id_text);
                Render::text_shadow(painter, Pos2::new(text_pos2d.x, text_pos2d.y), egui::Align2::CENTER_BOTTOM, &grenade.grenade_class.to_icon(), text_colour, &font_id_icon);
            }
        }
    }
    
}