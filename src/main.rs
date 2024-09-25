#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;

extern crate egui;
extern crate egui_overlay;
extern crate egui_render_three_d;
extern crate rdev;
use rdev::{listen, Event, EventType, Key};

extern crate crossbeam;
use crossbeam::channel;
use std::sync::{Arc, Barrier};
use std::thread;

mod driver;
use driver::Driver;
mod render;
mod game;

mod sdk { pub mod CUtlString; pub mod Player; pub mod Vector;  }
use sdk::Player::PlayerBase;
use sdk::Player::Player;

mod config;
use config::{init_config, init_keystate, SharedConfig, SharedKeyState};

mod features {pub mod combat;}
use features::{ combat };

mod cs2_dumper {pub mod offsets; pub mod libclient_so;}

fn main() {
    let mut driver = Driver::new();

    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    let _pid = driver.set_task("cs2");

    let config = init_config();
    let keystate = init_keystate();
    key_listener(keystate.clone());
        
    let (player_cache_sender, player_cache_receiver) = channel::unbounded::<Vec<PlayerBase>>();
    let (player_sender, player_receiver) = channel::unbounded::<Vec<Player>>();
    let player_barrier = Arc::new(Barrier::new(2));

    let cache_players_handle = game::cache_players(
        driver.clone(), 
        player_cache_sender);

    let update_players_handle = game::update_players(
        driver.clone(), 
        player_cache_receiver, 
        player_sender, 
        Arc::clone(&player_barrier));
        
    let combat_handle = combat::run_combat(
            driver.clone(),
            config.clone(),
            player_receiver.clone());

    render::run_overlay(player_receiver.clone(), Arc::clone(&player_barrier), keystate.clone(), config.clone());

    cache_players_handle.join().unwrap();
    update_players_handle.join().unwrap();
    combat_handle.join().unwrap();

}


fn key_listener(keystate: SharedKeyState) -> () {
    thread::spawn(move || {
        if let Err(error) = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if key == Key::Insert {
                        let mut keystate = keystate.write().unwrap();
                        keystate.show_gui = !keystate.show_gui;
                    }
                },
                _ => {}
            }
        }) {
            println!("Error: {:?}", error)
        }
    });
}