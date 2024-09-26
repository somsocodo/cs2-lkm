use std::thread;
use std::time::Duration;
use crossbeam::channel::Receiver;
use rdev::{simulate, Button, EventType, SimulateError};

use driver::Driver;
use libc::INT_MAX;
use sdk::CUtl::CUtlVector;
use sdk::Player::Player;
use sdk::Vector::Vector2;

use crate::{config::{ SharedConfig, SharedKeyState }, sdk::Player::SharedPlayerBase};

use cs2_dumper::libclient_so::cs2_dumper::schemas;

fn click() {
    match simulate(&EventType::ButtonPress(Button::Left)) {
        Ok(()) => simulate(&EventType::ButtonRelease(Button::Left)).unwrap(),
        Err(SimulateError) => {
            println!("Failed click.");
        }
    }
}

pub fn run_combat(
    driver: Driver, 
    shared_local_player: SharedPlayerBase,
    shared_keystate: SharedKeyState,
    shared_config: SharedConfig,
    player_receiver: Receiver<Vec<Player>>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut target: Player = Player::default();

        loop {
            let config = {
                let config_read = shared_config.read().unwrap();
                config_read.clone()
            };

            let keystate = {
                let keystate_read = shared_keystate.read().unwrap();
                keystate_read.clone()
            };

            let local_player = {
                let local_player_read = shared_local_player.read().unwrap();
                local_player_read.clone()
            };

            let mut closest_dist: f32 = INT_MAX as f32;
            let window_center = (config.window_size.0 as f32 / 2.0, config.window_size.1  as f32 / 2.0);

            let punch_angle: Vector2 = get_vec_punch(&driver, local_player.pawn);

            if let Ok(players) = player_receiver.recv() {
                for player in players.iter() {
                    if player.health == 0 || !player.bspotted {
                        continue;
                    }

                    if config.trigger_enabled && keystate.trigger && player.in_cross && punch_angle.x > -0.01 {
                        click();
                    }
                
                    let dist_x = (window_center.0 - player.bones_2d[0].x).abs();
                    let dist_y = (window_center.1 - player.bones_2d[0].y).abs();
                    let total_dist = dist_x + dist_y;

                    if total_dist < closest_dist {
                        closest_dist = total_dist;
                        target = player.clone();
                    }
                }
            }

            if closest_dist as i32 == INT_MAX {
                continue;
            }

            if !config.aim_enabled {
                continue;
            }

            //println!("target: {} {} | {}", target.name.to_str(), target.in_cross, closest_dist);

            thread::sleep(Duration::from_millis(1));
        }

    })
}


pub fn get_vec_punch(driver: &Driver, local_player: usize) -> Vector2 {
    let mut data = Vector2 { x: 0.0, y: 0.0 };
    let mut aim_punch_cache: [u64; 2] = [0, 0];
    
    let punch_cache: CUtlVector = driver.read_mem(local_player + schemas::libclient_so::C_CSPlayerPawn::m_aimPunchCache);

    aim_punch_cache[0] = punch_cache.count as u64;
    aim_punch_cache[1] = punch_cache.data as u64;

    if aim_punch_cache[0] == 0 || aim_punch_cache[1] == 0 {
        return data;
    }

    let aimpunch_size: u32 = (aim_punch_cache[0] & 0xFFFFFFFF) as u32;  // Extract the 32-bit size

    if aimpunch_size < 1 {
        return data;
    }

    let aimpunch_size = if aimpunch_size == 129 { 130 } else { aimpunch_size };

    data = driver.read_mem((aim_punch_cache[1] + ((aimpunch_size as u64 - 1) * 12)) as usize);

    if data.is_zero() {
        data = Vector2 { x: 0.0, y: 0.0 };
    }

    data
}