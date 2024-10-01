use driver::Driver;
use sdk::Vector::{ Vector2, Vector3};
use sdk::Vector::vec_translate;
use crate::sdk::Player::{ SharedPlayerBase, PlayerBase };
use sdk::WeaponClass::{get_grenade_class, GrenadeClass};

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
}