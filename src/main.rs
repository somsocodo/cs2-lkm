#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;
extern crate egui;
extern crate egui_overlay;
extern crate egui_render_three_d;
extern crate crossbeam;
use crossbeam::channel;
use std::sync::{Arc, Mutex, Barrier};

mod driver;
use driver::Driver;
mod render;
mod game;

mod sdk { pub mod CUtlString; pub mod Player; pub mod Vector;  }
use sdk::Player::Player;

mod cs2_dumper {pub mod offsets; pub mod libclient_so;}

fn main() {
    let mut driver = Driver::new();

    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    let _pid = driver.set_task("cs2");
    
    let driver_arc = Arc::new(Mutex::new(driver));
    
    let (player_sender, player_receiver) = channel::unbounded::<Vec<Player>>();
    let barrier = Arc::new(Barrier::new(2));

    let cache_players_handle = game::cache_players(Arc::clone(&driver_arc), player_sender, Arc::clone(&barrier));
    
    render::run_overlay(player_receiver, Arc::clone(&barrier));

    cache_players_handle.join().unwrap();
}