extern crate libc;
extern crate nix;
mod driver;

fn main() {
    let mut driver = driver::Driver::new();
    driver.open_device("mem-device");
    driver.set_task("cs2");
}