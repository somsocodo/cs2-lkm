#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;
extern crate egui;
extern crate egui_overlay;
extern crate egui_render_three_d;
extern crate crossbeam;
use crossbeam::channel;
use std::sync::{Arc, Barrier};

mod driver;
use driver::Driver;
mod render;
mod game;

mod sdk { pub mod CUtlString; pub mod Player; pub mod Vector;  }
use sdk::Player::PlayerBase;
use sdk::Player::Player;

mod cs2_dumper {pub mod offsets; pub mod libclient_so;}

fn main() {
    let mut driver = Driver::new();

    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    let _pid = driver.set_task("cs2");
        
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


    render::run_overlay(player_receiver, Arc::clone(&player_barrier));

    cache_players_handle.join().unwrap();
    update_players_handle.join().unwrap();

}