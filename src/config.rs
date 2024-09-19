use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Config {
    pub show_gui: bool,
    pub nametags: bool,
}

impl Config {
    pub fn new() -> Self {
        Self {
            show_gui: false,
            nametags: true
        }
    }
}

pub type SharedConfig = Arc<Mutex<Config>>;

pub fn init_config() -> SharedConfig {
    Arc::new(Mutex::new(Config::new()))
}