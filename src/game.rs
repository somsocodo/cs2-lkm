use std::sync::{Arc, Mutex, Barrier};
use std::thread;
use std::time::Duration;
use crossbeam::channel::Sender;

use driver::Driver;
use sdk::CUtlString::CUtlString;
use sdk::Player::Player;
use sdk::Vector::Vector3;

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;

pub fn cache_players(driver: Arc<Mutex<Driver>>, player_sender: Sender<Vec<Player>>, barrier: Arc<Barrier>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let driver = driver.lock().unwrap();
        let client_addr =  driver.read_module("libclient.so");
        println!("found libclient.so: {:#04X?}", client_addr);
        
        loop {
            let mut players = Vec::new();

            let view_matrix: [[f32; 4]; 4] =  driver.read_mem(client_addr + offsets::libclient_so::dwViewMatrix);

            let entity_list: usize = driver.read_mem(client_addr + offsets::libclient_so::dwEntityList);
            let list_entry: usize = driver.read_mem(entity_list + 0x10);

            for i in 0..64 { 
                let current_controller: usize = driver.read_mem(list_entry + (i * 0x78));
                if current_controller == 0 {
                    continue;
                }

                let pawn_handle: usize = driver.read_mem(current_controller + schemas::libclient_so::CCSPlayerController::m_hPlayerPawn);
                if pawn_handle == 0 {
                    continue;
                }

                let pawn_entry: usize = driver.read_mem(entity_list + (0x8 * ((pawn_handle & 0x7FFF) >> 9) + 0x10));
                let current_pawn: usize = driver.read_mem(pawn_entry + (0x78 * (pawn_handle & 0x1FF)));

                let name: CUtlString = driver.read_mem(current_controller + schemas::libclient_so::CBasePlayerController::m_iszPlayerName);
                let health: i32 = driver.read_mem(current_pawn + schemas::libclient_so::C_BaseEntity::m_iHealth);
                let feet_pos: Vector3 = driver.read_mem(current_pawn + schemas::libclient_so::C_BasePlayerPawn::m_vOldOrigin);
                let eye_pos: Vector3 = feet_pos + driver.read_mem(current_pawn + schemas::libclient_so::C_BaseModelEntity::m_vecViewOffset);

                let pos_2d = eye_pos.world_to_screen(view_matrix);

                let player = Player::new(current_pawn, name, health, eye_pos, pos_2d);
                if i < players.len() {
                    players[i] = player; 
                } else {
                    players.push(player); 
                }
            }
            player_sender.send(players).unwrap();
            barrier.wait();
            thread::sleep(Duration::from_millis(1));
        }  
    })
}