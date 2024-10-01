#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;

extern crate egui;
extern crate egui_overlay;
extern crate egui_render_three_d;
extern crate once_cell;
extern crate rdev;
extern crate serde;
extern crate serde_json;

use features::grenades::GrenadeHelper;
use rdev::{listen, Event, EventType, Key, Button};

extern crate crossbeam;
use crossbeam::channel;
use std::sync::{Arc, RwLock};
use std::thread;

mod driver;
use driver::Driver;
mod render;
mod game;

mod sdk { pub mod CUtl; pub mod Icon; pub mod Player; pub mod Entity; pub mod Vector; pub mod WeaponClass; }
use sdk::Player::{ PlayerBase, Player};
use sdk::Entity::{ EntityBase, Entity };

mod config;
use config::{init_config, init_keystate, SharedActiveState};

mod features {pub mod combat; pub mod grenades;}
use features::{ combat, grenades };

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
    let activestate = init_keystate();
    key_listener(activestate.clone());
    
    let local_player = Arc::new(RwLock::new(PlayerBase::default()));
    let (player_cache_sender, player_cache_receiver) = channel::unbounded::<Vec<PlayerBase>>();
    let (player_sender, player_receiver) = channel::unbounded::<Vec<Player>>();

    let (world_cache_sender, world_cache_receiver) = channel::unbounded::<Vec<EntityBase>>();
    let (world_sender, world_receiver) = channel::unbounded::<Vec<Entity>>();

    let cache_players_handle = game::cache_players(
        driver.clone(), 
        player_cache_sender);

    let cache_world_handle = game::cache_world(
        driver.clone(), 
        world_cache_sender);

    let update_players_handle = game::update_players(
        driver.clone(), 
        local_player.clone(),
        config.clone(),
        activestate.clone(),
        player_cache_receiver, 
        player_sender);

    let update_world_handle = game::update_world(
            driver.clone(),
            config.clone(),
            world_cache_receiver, 
            world_sender);
        
    let combat_handle = combat::run_combat(
            driver.clone(),
            local_player.clone(),
            activestate.clone(),
            config.clone(),
            player_receiver.clone());

    render::run_overlay(
        player_receiver.clone(), 
        world_receiver.clone(),
        activestate.clone(), 
        config.clone(),
    GrenadeHelper::new(driver.clone(), local_player.clone()));

    cache_players_handle.join().unwrap();
    cache_world_handle.join().unwrap();
    update_players_handle.join().unwrap();
    update_world_handle.join().unwrap();
    combat_handle.join().unwrap();

}


fn key_listener(activestate: SharedActiveState) -> () {
    thread::spawn(move || {
        if let Err(error) = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    if key == Key::Insert {
                        let mut activestate = activestate.write().unwrap();
                        activestate.show_gui = !activestate.show_gui;
                    }
                },
                EventType::ButtonPress(button) => {
                    if button == Button::Unknown(8) {
                        let mut activestate = activestate.write().unwrap();
                        activestate.trigger = true;
                    }
                    if button == Button::Left {
                        let mut activestate = activestate.write().unwrap();
                        activestate.aim = true;
                    }
                },
                EventType::ButtonRelease(button) => {
                    if button == Button::Unknown(8) {
                        let mut activestate = activestate.write().unwrap();
                        activestate.trigger = false;
                    }
                    if button == Button::Left {
                        let mut activestate = activestate.write().unwrap();
                        activestate.aim = false;
                    }
                },
                _ => {}
            }
        }) {
            println!("Error: {:?}", error)
        }
    });
}