use std::sync::{Arc, RwLock};

#[derive(Clone, Copy, PartialEq)]
pub struct Config {
    pub window_size: (i32, i32),
    pub gui_visuals: bool,
    pub gui_combat: bool,
    pub gui_misc: bool,
    pub esp_nametags: bool,
    pub esp_hitboxes: bool,
    pub esp_bones: bool,
    pub aim_enabled: bool,
    pub aim_smoothing: f32,
    pub trigger_enabled: bool,
    pub ignore_team: bool
}

impl Config {
    pub fn new() -> Self {
        Self {
            window_size: (1024, 768),
            gui_visuals: true,
            gui_combat: true,
            gui_misc: true,
            esp_nametags: true,
            esp_hitboxes: true,
            esp_bones: true,
            aim_enabled: true,
            aim_smoothing: 3.5,
            trigger_enabled: true,
            ignore_team: true
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
    pub trigger: bool,
    pub aim: bool
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            show_gui: false,
            trigger: false,
            aim: false
        }
    }
}

pub type SharedKeyState = Arc<RwLock<KeyState>>;

pub fn init_keystate() -> SharedKeyState {
    Arc::new(RwLock::new(KeyState::new()))
}