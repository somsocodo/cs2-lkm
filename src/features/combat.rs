use std::thread;
use std::time::Duration;
use crossbeam::channel::Receiver;
use rdev::{simulate, Button, EventType, SimulateError};

use driver::Driver;
use libc::INT_MAX;
use sdk::Player::Player;

use crate::config::{ SharedKeyState, SharedConfig };

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
    shared_keystate: SharedKeyState,
    shared_config: SharedConfig,
    player_receiver: Receiver<Vec<Player>>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut target = Player::default();
        loop {
            let config = {
                let config_read = shared_config.read().unwrap();
                config_read.clone()
            };

            let keystate = {
                let keystate_read = shared_keystate.read().unwrap();
                keystate_read.clone()
            };

            let mut closest_dist: f32 = INT_MAX as f32;
            let window_center = (config.window_size.0 as f32 / 2.0, config.window_size.1  as f32 / 2.0);
            if let Ok(players) = player_receiver.recv() {
                for player in players.iter() {
                    if player.health == 0 || !player.bspotted {
                        continue;
                    }

                    if config.trigger_enabled && keystate.trigger && player.in_cross{
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

            println!("target: {} {} | {}", target.name.to_str(), target.in_cross, closest_dist);

            thread::sleep(Duration::from_millis(1));
        }

    })
}