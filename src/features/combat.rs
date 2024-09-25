use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use crossbeam::channel::Sender;
use crossbeam::channel::Receiver;


use driver::Driver;
use sdk::CUtlString::CUtlString;
use sdk::Player::Player;
use sdk::Player::PlayerBase;
use sdk::Vector::Vector3;

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;

use crate::config::SharedConfig;
use crate::sdk::Vector::Vector2;


pub fn run_combat(
    driver: Driver, 
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


            if let Ok(players) = player_receiver.recv() {
                for player in players.iter() {
                    
                    let window_center = (config.window_size.0 as f32 / 2.0, config.window_size.1  as f32 / 2.0);

                    let dist_x = (window_center.0 - player.bones_2d[0].x).abs();
                    let dist_y = (window_center.1 - player.bones_2d[0].y).abs();
              
                    let t_dist_x = (window_center.0 - target.bones_2d[0].x).abs();
                    let t_dist_y = (window_center.1 - target.bones_2d[0].y).abs();
                    println!("{} | {}", player.name.to_str(), dist_x+dist_y);
                }
            }

            thread::sleep(Duration::from_millis(1));
        }

    })
}