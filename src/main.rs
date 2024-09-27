#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;

extern crate egui;
extern crate egui_overlay;
extern crate egui_render_three_d;
extern crate rdev;
use rdev::{listen, Event, EventType, Key, Button};

extern crate crossbeam;
use crossbeam::channel;
use std::sync::{Arc, RwLock};
use std::thread;

mod driver;
use driver::Driver;
mod render;
mod game;

mod sdk { pub mod CUtl; pub mod Player; pub mod Vector; pub mod WeaponClass; }
use sdk::Player::{ PlayerBase, Player};

mod config;
use config::{init_config, init_keystate, SharedKeyState};

mod features {pub mod combat;}
use features::{ combat };

mod cs2_dumper {pub mod offsets; pub mod libclient_so;}

fn main() {
    let mut driver = Driver::new();

    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found memory device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    match driver.open_input_device("event-mouse") {
        Ok(fd)  => println!("found mouse device, fd: {}", fd),
        _e => panic!("unable to find mouse device")
    };

    let _pid = driver.set_task("cs2");

    let config = init_config();
    let keystate = init_keystate();
    key_listener(keystate.clone());
    
    let local_player = Arc::new(RwLock::new(PlayerBase::default()));
    let (player_cache_sender, player_cache_receiver) = channel::unbounded::<Vec<PlayerBase>>();
    let (player_sender, player_receiver) = channel::unbounded::<Vec<Player>>();

    let cache_players_handle = game::cache_players(
        driver.clone(), 
        player_cache_sender);

    let update_players_handle = game::update_players(
        driver.clone(), 
        local_player.clone(),
        config.clone(),
        player_cache_receiver, 
        player_sender);
        
    let combat_handle = combat::run_combat(
            driver.clone(),
            local_player.clone(),
            keystate.clone(),
            config.clone(),
            player_receiver.clone());

    render::run_overlay(player_receiver.clone(), keystate.clone(), config.clone());

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
                EventType::ButtonPress(button) => {
                    if button == Button::Unknown(8) {
                        let mut keystate = keystate.write().unwrap();
                        keystate.trigger = true;
                    }
                    if button == Button::Left {
                        let mut keystate = keystate.write().unwrap();
                        keystate.aim = true;
                    }
                },
                EventType::ButtonRelease(button) => {
                    if button == Button::Unknown(8) {
                        let mut keystate = keystate.write().unwrap();
                        keystate.trigger = false;
                    }
                    if button == Button::Left {
                        let mut keystate = keystate.write().unwrap();
                        keystate.aim = false;
                    }
                },
                _ => {}
            }
        }) {
            println!("Error: {:?}", error)
        }
    });
}