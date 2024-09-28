use std::thread;
use std::time::Duration;
use crossbeam::channel::Sender;
use crossbeam::channel::Receiver;


use driver::Driver;
use crate::config::SharedConfig;
use sdk::CUtl::CUtlString;
use sdk::Player::{ PlayerBase, SharedPlayerBase, Player};
use sdk::Entity::{ EntityBase, Entity };
use sdk::Vector::Vector3;

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;

use crate::sdk::Vector::Vector2;

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

                let player_base = PlayerBase::new(current_pawn, current_controller, i as u32);
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
    shared_local_player: SharedPlayerBase,
    shared_config: SharedConfig,
    player_cache_receiver: Receiver<Vec<PlayerBase>>,
    player_sender: Sender<Vec<Player>>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client_addr =  driver.read_module("libclient.so");
        let mut player_cache = Vec::new();
        let mut local_player: usize = 0;
        let mut local_idx: u32 = 0;

        loop {
            let config = {
                let config_read = shared_config.read().unwrap();
                config_read.clone()
            };

            let mut players = Vec::new();
            
            let view_matrix: [[f32; 4]; 4] =  driver.read_mem(client_addr + offsets::libclient_so::dwViewMatrix);
            
            let entity_list: usize = driver.read_mem(client_addr + offsets::libclient_so::dwEntityList);
            let cross_pawn_idx: usize = driver.read_mem(local_player + schemas::libclient_so::C_CSPlayerPawnBase::m_iIDEntIndex);
            let cross_pawn_entry: usize = driver.read_mem(entity_list + (0x8 * ((cross_pawn_idx & 0x7FFF) >> 9) + 0x10));
            let cross_pawn: usize = driver.read_mem(cross_pawn_entry + (0x78 * (cross_pawn_idx & 0x1FF)));

            let local_team_num: u32 = driver.read_mem(local_player + schemas::libclient_so::C_BaseEntity::m_iTeamNum);

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

                    let pawn_team_num: u32 = driver.read_mem(current_pawn + schemas::libclient_so::C_BaseEntity::m_iTeamNum);

                    if config.ignore_team && pawn_team_num == local_team_num {
                        continue;
                    }

                    if driver.read_mem(current_controller + schemas::libclient_so::CBasePlayerController::m_bIsLocalPlayerController){
                        let mut local_player_edit = shared_local_player.write().unwrap();
                        *local_player_edit = player_base.clone();
                        local_player = current_pawn;
                        local_idx = player_base.idx;
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
                    let feetpos_2d = feet_pos.world_to_screen(view_matrix);

                    if (pos_2d.x < 0.0 && pos_2d.y < 0.0) && (feetpos_2d.x < 0.0 && feetpos_2d.y < 0.0) {
                        continue;
                    }

                    let pawn_spotted_state: u32 = driver.read_mem(current_pawn + schemas::libclient_so::C_CSPlayerPawn::m_entitySpottedState + schemas::libclient_so::EntitySpottedState_t::m_bSpottedByMask);
                    let local_spotted_state: u32 = driver.read_mem(local_player + schemas::libclient_so::C_CSPlayerPawn::m_entitySpottedState + schemas::libclient_so::EntitySpottedState_t::m_bSpottedByMask);
                    
                    let mut bspotted = false;
                    if (pawn_spotted_state & (1 << (local_idx - 1)) != 0) || (local_spotted_state & (1 << (i - 1)) != 0) {
                        bspotted = true;
                    }

                    let mut in_cross = false;
                    if current_pawn == cross_pawn{
                        bspotted = true;
                        in_cross = true;
                    }    

                    let scene_node: usize = driver.read_mem(current_pawn + schemas::libclient_so::C_BaseEntity::m_pGameSceneNode);
                    let bone_matrix: usize = driver.read_mem(scene_node + schemas::libclient_so::CSkeletonInstance::m_modelState + 0x80);
                    let view_angle: Vector2 = driver.read_mem(local_player + schemas::libclient_so::C_BasePlayerPawn::v_angle);

                    let mut player = Player::new(
                        current_pawn, 
                        current_controller, 
                        player_base.idx,
                        name, 
                        bspotted, 
                        in_cross, 
                        health, 
                        eye_pos, 
                        pos_2d);
                    
                    player.read_bones(driver, bone_matrix, view_matrix);
                    player.read_hitboxes(view_angle, view_matrix);

                    if i < players.len() {
                        players[i] = player; 
                    } else {
                        players.push(player); 
                    }

            }
            player_sender.send(players).unwrap();
            
            thread::sleep(Duration::from_millis(3));
        }
    })
}

pub fn cache_world(
    driver: Driver, 
    world_cache_sender: Sender<Vec<EntityBase>> 
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client_addr =  driver.read_module("libclient.so");
        
        loop {
            let mut world_cache = Vec::new();

            let entity_list: usize = driver.read_mem(client_addr + offsets::libclient_so::dwEntityList);
            let max_ent_idx: i32 = driver.read_mem(entity_list + offsets::libclient_so::dwGameEntitySystem_highestEntityIndex);

            for i in 65..max_ent_idx as usize { 
                let world_entry: usize = driver.read_mem(entity_list + (0x8 * ((i & 0x7FFF) >> 9) + 0x10));
                let base_entity_addr: usize = driver.read_mem(world_entry + 0x78 * (i & 0x1FF));

                if base_entity_addr == 0 {
                    continue;
                }

                let entity_identity: usize = driver.read_mem(base_entity_addr + 0x10);

                if entity_identity == 0 {
                    continue;
                }

                let class_name_addr: usize = driver.read_mem(entity_identity + 0x20);
                let class_name: CUtlString = driver.read_mem(class_name_addr);
                
                let class_name_str = class_name.to_str();
                let len = class_name_str.len();

                if len < 9 { // cannot contain weapon || projectile
                    continue;
                }

                let bytes = class_name_str.as_bytes();

                let starts_with_weapon = &bytes[0..7] == b"weapon_";
                let ends_with_projectile = len >= 11 && &bytes[len-11..] == b"_projectile";
            
                if !starts_with_weapon && !ends_with_projectile {
                    continue;
                }

                if starts_with_weapon {
                    let owner: i32 = driver.read_mem(base_entity_addr + schemas::libclient_so::C_BaseEntity::m_hOwnerEntity);
                    if owner != -1 {
                        continue;
                    }
                }

                let mut is_projectile = false;

                if ends_with_projectile {
                    is_projectile = true;
                }

                let mut entity_base = EntityBase::new(base_entity_addr, class_name, is_projectile, false);

                if starts_with_weapon {
                    entity_base.ammo[0] = driver.read_mem(base_entity_addr + schemas::libclient_so::C_BasePlayerWeapon::m_iClip1);
                    entity_base.ammo[1] = driver.read_mem(base_entity_addr + schemas::libclient_so::C_BasePlayerWeapon::m_iClip1);
                }

                if i < world_cache.len() {
                    world_cache[i] = entity_base; 
                } else {
                    world_cache.push(entity_base); 
                }
                thread::sleep(Duration::from_millis(1));
            }

            let planted_c4: bool = driver.read_mem(client_addr + offsets::libclient_so::dwPlantedC4 - 0x8);
            if planted_c4 {
                let p_planted_c4: usize = driver.read_mem(client_addr + offsets::libclient_so::dwPlantedC4);
                let c4_base_addr: usize = driver.read_mem(p_planted_c4);

                let c4_entity_base = EntityBase::new(c4_base_addr, CUtlString::new("planted_c4"), false, true);
                world_cache.push(c4_entity_base);
            }

            world_cache_sender.send(world_cache).unwrap();

            thread::sleep(Duration::from_millis(200));
        }  
    })
}

pub fn update_world(
    driver: Driver, 
    shared_config: SharedConfig,
    world_cache_receiver: Receiver<Vec<EntityBase>>,
    world_sender: Sender<Vec<Entity>>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let client_addr =  driver.read_module("libclient.so");
        let mut world_cache = Vec::new();

        loop {
            let config = {
                let config_read = shared_config.read().unwrap();
                config_read.clone()
            };

            let mut world_list = Vec::new();

            if !config.esp_world {
                world_sender.send(world_list).unwrap();
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            
            let view_matrix: [[f32; 4]; 4] =  driver.read_mem(client_addr + offsets::libclient_so::dwViewMatrix);

            if let Ok(world_cache_channel) = world_cache_receiver.try_recv() {
                world_cache.clear();
                world_cache.extend(world_cache_channel.iter().cloned());
            }

            for (i, entity_base) in world_cache.iter().enumerate()  {
                let game_scene_node: usize = driver.read_mem(entity_base.addr + schemas::libclient_so::C_BaseEntity::m_pGameSceneNode);
                let origin: Vector3 = driver.read_mem(game_scene_node + schemas::libclient_so::CGameSceneNode::m_vecAbsOrigin);
                let origin_2d = origin.world_to_screen(view_matrix);

                if origin_2d.x < 0.0 && origin_2d.y < 0.0 {
                    continue;
                }

                let mut ammo = entity_base.ammo;
                if entity_base.is_planted_c4 {
                    let c4_blow: f32 = driver.read_mem(entity_base.addr + schemas::libclient_so::C_PlantedC4::m_flC4Blow);
                    let c4_next_beep: f32 = driver.read_mem(entity_base.addr + schemas::libclient_so::C_PlantedC4::m_flNextBeep);
                    let c4_timer_length: f32 = driver.read_mem(entity_base.addr + schemas::libclient_so::C_PlantedC4::m_flTimerLength);
                    ammo = [(c4_blow - c4_next_beep) as i32, c4_timer_length as i32];
                }

                let entity = Entity::new(entity_base.addr, entity_base.class_name, entity_base.is_projectile, entity_base.is_planted_c4, origin, origin_2d, ammo);

                if i < world_list.len() {
                    world_list[i] = entity; 
                } else {
                    world_list.push(entity); 
                }
            }

            world_sender.send(world_list).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    })
}