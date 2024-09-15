#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

extern crate libc;
extern crate nix;
mod driver;
mod schema { pub mod CUtlString; }
use schema::CUtlString::CUtlString;

//https://github.com/a2x/cs2-dumper
pub const dwLocalPlayerController: usize = 0x386BEF8;
pub const m_iPawnHealth: usize = 0x980; // uint32
pub const m_iszPlayerName: usize = 0x7C8; // char[128]

fn main() {
    let mut driver = driver::Driver::new();
    match driver.open_device("mem-device") {
        Ok(fd)  => println!("found device, fd: {}", fd),
        _e => panic!("unable to find device")
    };

    let _pid = driver.set_task("cs2");
 
    let client_addr =  driver.read_module("libclient.so");
    println!("found libclient.so: {:#04X?}", client_addr);

    let local_controller: usize = driver.read_mem(client_addr + dwLocalPlayerController);
    let health: i32 = driver.read_mem(local_controller + m_iPawnHealth);
    let name: CUtlString = driver.read_mem(local_controller + m_iszPlayerName);

    println!("{} | {}", name.to_str(), health);

}