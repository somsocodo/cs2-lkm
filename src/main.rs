#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]
extern crate libc;
extern crate nix;
use std::sync::{Arc, Mutex};

mod schema { pub mod CUtlString; }
mod cs2_dumper {pub mod offsets; pub mod libclient_so;}

mod driver;
use driver::Driver;
mod game;

fn main() {
    let mut driver = Driver::new();

    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    let _pid = driver.set_task("cs2");
    
    let driver_arc = Arc::new(Mutex::new(driver));
    
    let cache_players_handle = game::cache_players(Arc::clone(&driver_arc));

    cache_players_handle.join().unwrap();

}