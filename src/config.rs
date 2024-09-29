use std::sync::{Arc, RwLock};
use egui::Color32;

#[derive(Clone, Copy, PartialEq)]
pub struct Config {
    pub window_size: (i32, i32),
    pub gui_visuals: bool,
    pub gui_combat: bool,
    pub gui_misc: bool,

    pub esp_nametags: bool,
    pub esp_hitboxes: bool,
    pub esp_hitboxes_col_vis: Color32,
    pub esp_hitboxes_col_hid: Color32,
    pub esp_bones: bool,
    pub esp_world: bool,

    pub aim_enabled: bool,
    pub aim_fov: f32,
    pub aim_smoothing: f32,
    pub aim_shoot_delay: u64,
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
            esp_hitboxes_col_vis: Color32::from_rgba_premultiplied(0, 200, 0, 1),
            esp_hitboxes_col_hid: Color32::from_rgba_premultiplied(200, 0, 0, 1),
            esp_bones: true,
            esp_world: true,

            aim_enabled: true,
            aim_fov: 5.0,
            aim_smoothing: 3.5,
            aim_shoot_delay: 130,
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