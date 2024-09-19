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

pub fn cache_players(
    driver: Driver, 
    player_cache_sender: Sender<Vec<PlayerBase>>, 
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client_addr =  driver.read_module("libclient.so");
        println!("found libclient.so (cache): {:#04X?}", client_addr);
        
        loop {
            let mut players_cache = Vec::new();

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

                let player_base = PlayerBase::new(current_pawn, current_controller, i);
                if i < players_cache.len() {
                    players_cache[i] = player_base; 
                } else {
                    players_cache.push(player_base); 
                }
            }
            player_cache_sender.send(players_cache).unwrap();
            thread::sleep(Duration::from_millis(500));
        }  
    })
}

pub fn update_players(
    driver: Driver, 
    player_cache_receiver: Receiver<Vec<PlayerBase>>,
    player_sender: Sender<Vec<Player>>,
    player_barrier: Arc<Barrier>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client_addr =  driver.read_module("libclient.so");
        let mut player_cache = Vec::new();
        let mut local_player: usize = 0;

        loop {
            let mut players = Vec::new();
            
            let view_matrix: [[f32; 4]; 4] =  driver.read_mem(client_addr + offsets::libclient_so::dwViewMatrix);

            if let Ok(players_cache_channel) = player_cache_receiver.try_recv() {
                player_cache.clear();
                player_cache.extend(players_cache_channel.iter().cloned());
            }

            for (i, player_base) in player_cache.iter().enumerate()  {
                    let current_pawn = player_base.pawn;
                    let current_controller = player_base.controller;

                    if current_pawn == 0 {
                        continue;
                    }

                    let health: i32 = driver.read_mem(current_pawn + schemas::libclient_so::C_BaseEntity::m_iHealth);

                    if health <= 0 {
                        continue;
                    }

                    if driver.read_mem(current_controller + schemas::libclient_so::CBasePlayerController::m_bIsLocalPlayerController){
                        local_player = current_pawn;
                        continue;
                    }

                    if local_player == 0 {
                        continue;
                    }

                    let name: CUtlString = driver.read_mem(current_controller + schemas::libclient_so::CBasePlayerController::m_iszPlayerName);
                    let feet_pos: Vector3 = driver.read_mem(current_pawn + schemas::libclient_so::C_BasePlayerPawn::m_vOldOrigin);
                    let mut eye_pos: Vector3 = feet_pos + driver.read_mem(current_pawn + schemas::libclient_so::C_BaseModelEntity::m_vecViewOffset);
                    eye_pos.z += 13.5; // For nametags only
                    let pos_2d = eye_pos.world_to_screen(view_matrix);

                    let scene_node: usize = driver.read_mem(current_pawn + schemas::libclient_so::C_BaseEntity::m_pGameSceneNode);
                    let bone_matrix: usize = driver.read_mem(scene_node + schemas::libclient_so::CSkeletonInstance::m_modelState + 0x80);

                    let mut player = Player::new(name, health, eye_pos, pos_2d);
                    player.read_bones(driver, bone_matrix, view_matrix);

                    if i < players.len() {
                        players[i] = player; 
                    } else {
                        players.push(player); 
                    }

            }

            player_sender.send(players).unwrap();
            player_barrier.wait();
            thread::sleep(Duration::from_millis(1));
        }
    })
}