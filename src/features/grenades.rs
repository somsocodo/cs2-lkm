use driver::Driver;
use sdk::Vector::{ Vector2, Vector3};
use sdk::Vector::vec_translate;
use crate::sdk::Player::{ SharedPlayerBase, PlayerBase };

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;

struct Grenade {
    name: String,
    action: String,
    pos: Vector3,
    throw_pos: Vector3
}

pub struct GrenadeHelper {
    driver: Driver,
    local_player: SharedPlayerBase
}

impl GrenadeHelper {
    pub fn new(driver: Driver, local_player: SharedPlayerBase) -> Self {
        Self {
            driver,
            local_player
        }
    }

    pub fn save(&self, name: String, action: String){
        let local_player: PlayerBase = {
            let local_player_read = self.local_player.read().unwrap();
            local_player_read.clone()
        };

        let pos: Vector3 = self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::m_vOldOrigin);

        let view_angle: Vector2 = self.driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::v_angle);
        let throw_pos: Vector3 = vec_translate(&pos, &view_angle, 2000.0);

        println!("save_grenade {} {} {} {} {}", name, action, pos.x, pos.y, pos.z);
    }
}