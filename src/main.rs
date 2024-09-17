#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;
extern crate egui;
extern crate egui_overlay;
extern crate egui_render_three_d;
use std::sync::{Arc, Mutex};


mod sdk { pub mod CUtlString; pub mod Player; pub mod Vector;  }
mod cs2_dumper {pub mod offsets; pub mod libclient_so;}

mod driver;
use driver::Driver;
mod render;
mod game;

fn main() {
    let mut driver = Driver::new();

    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    let _pid = driver.set_task("cs2");
    
    let driver_arc = Arc::new(Mutex::new(driver));
    
    let player_list = Arc::new(Mutex::new(vec![]));

    let cache_players_handle = game::cache_players(Arc::clone(&driver_arc), Arc::clone(&player_list));
    let overlay_handle = render::run_overlay(Arc::clone(&player_list));

    cache_players_handle.join().unwrap();
    overlay_handle.join().unwrap();

}