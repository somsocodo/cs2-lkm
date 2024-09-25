use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Config {
    pub window_size: (i32, i32),
    pub esp_nametags: bool,
    pub esp_hitboxes: bool,
    pub esp_bones: bool,
    pub aim_enabled: bool
}

impl Config {
    pub fn new() -> Self {
        Self {
            window_size: (1024, 768),
            esp_nametags: true,
            esp_hitboxes: true,
            esp_bones: false,
            aim_enabled: true
        }
    }
}

pub type SharedConfig = Arc<RwLock<Config>>;

pub fn init_config() -> SharedConfig {
    Arc::new(RwLock::new(Config::new()))
}

#[derive(Clone)]
pub struct KeyState {
    pub show_gui: bool,
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            show_gui: false,
        }
    }
}

pub type SharedKeyState = Arc<RwLock<KeyState>>;

pub fn init_keystate() -> SharedKeyState {
    Arc::new(RwLock::new(KeyState::new()))
}