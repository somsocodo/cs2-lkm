use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use driver::Driver;

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;
use schema::CUtlString::CUtlString;

pub fn cache_players(driver: Arc<Mutex<Driver>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let driver = driver.lock().unwrap();
        let client_addr =  driver.read_module("libclient.so");
        println!("found libclient.so: {:#04X?}", client_addr);

        loop {
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

                println!("{} | {}", name.to_str(), health);
            }   
            thread::sleep(Duration::from_millis(10));
        }  
    })
}