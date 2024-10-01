use driver::Driver;
use sdk::Vector::{ Vector2, Vector3};
use sdk::Vector::vec_translate;
use crate::sdk::Player::{ SharedPlayerBase, PlayerBase };
use render::Render;
use sdk::WeaponClass::{get_grenade_class, GrenadeClass};

use egui::{ Color32 , Pos2};

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
        let view_angle: Vector2 = self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::v_angle);
        let throw_pos: Vector3 = vec_translate(&pos, &view_angle, 2000.0);

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
    
        if distance < 2.0 {
            fill_colour = egui::Color32::from_rgba_unmultiplied(20, 128, 20, alpha);
    
            let painter = ui.painter();
            let throw_pos_2d = grenade.throw_pos.world_to_screen(view_matrix);
    
            painter.circle_filled(Pos2::new(throw_pos_2d.x, throw_pos_2d.y), 2.0, fill_colour);
        }
    
        if distance < max_distance {
            Render::draw_circle(ui, grenade.pos, view_matrix, 12.0, true, fill_colour, stroke_colour, 3.0);
            Render::draw_circle(ui, grenade.pos, view_matrix, 0.5, true, Color32::WHITE, stroke_colour, 0.5);    
        }
    }
    
}