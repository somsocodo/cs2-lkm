use std::sync::{Arc, Mutex, RwLock};

#[derive(Debug, Clone)]
pub struct Config {
    pub window_size: (i32, i32),
    pub show_gui: bool,
    pub esp_nametags: bool,
    pub esp_hitboxes: bool,
    pub esp_bones: bool
}

impl Config {
    pub fn new() -> Self {
        Self {
            window_size: (1024, 768),
            show_gui: true,
            esp_nametags: true,
            esp_hitboxes: true,
            esp_bones: false
        }
    }
}

pub type SharedConfig = Arc<Mutex<Config>>;

pub fn init_config() -> SharedConfig {
    Arc::new(Mutex::new(Config::new()))
}