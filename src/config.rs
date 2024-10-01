use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Config {
    pub window_size: (i32, i32),
    pub gui_visuals: bool,
    pub gui_combat: bool,
    pub gui_grenades: bool,
    pub gui_misc: bool,

    pub esp_nametags: bool,
    pub esp_hitboxes: bool,
    pub esp_hitboxes_col_vis: [u8; 4],
    pub esp_hitboxes_col_hid: [u8; 4],
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
            gui_grenades: true,
            gui_misc: true,

            esp_nametags: true,
            esp_hitboxes: true,
            esp_hitboxes_col_vis: [0, 200, 0, 1], //Color32::from_rgba_premultiplied(0, 200, 0, 1),
            esp_hitboxes_col_hid: [200, 0, 0, 1], //Color32::from_rgba_premultiplied(200, 0, 0, 1),
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

    pub fn save(&self) {
        let j = match serde_json::to_string_pretty(&self) {
            Ok(json) => json,
            Err(e) => {
                println!("Failed to serialize config to JSON: {}", e);
                return;
            }
        };

        let mut file = match File::create("config.json") {
            Ok(f) => f,
            Err(e) => {
                println!("Failed to create or open config file: {}", e);
                return;
            }
        };

        if let Err(e) = file.write_all(j.as_bytes()) {
            println!("Failed to write config to file: {}", e);
        }

        println!("successfully saved to config.json");

    }

    pub fn load(log: bool) -> Self {
        let mut file = match File::open("config.json") {
            Ok(file) => file,
            Err(e) => {
                if log {
                    println!("Failed to open config file, reverting to default: {}", e);
                }
                return Config::new();
            }
        };
    
        let mut contents = String::new();
        if let Err(e) = file.read_to_string(&mut contents) {
            if log {
                println!("Failed to read config file, reverting to default: {}", e);
            }
            return Config::new();
        }
    
        match serde_json::from_str::<Config>(&contents) {
            Ok(config) => {
                if log {
                    println!("loaded config file");
                }
                config
            }
            Err(e) => {
                if log {
                    println!("Failed to parse config file, reverting to default: {}", e);
                }
                Config::new()
            }
        }
    }

}

pub type SharedConfig = Arc<RwLock<Config>>;

pub fn init_config() -> SharedConfig {
    Arc::new(RwLock::new(Config::load(true)))
}

#[derive(Clone)]
pub struct ActiveState {
    pub show_gui: bool,
    pub trigger: bool,
    pub aim: bool,
    pub weapon_index: i16
}

impl ActiveState {
    pub fn new() -> Self {
        Self {
            show_gui: true,
            trigger: false,
            aim: false,
            weapon_index: 0
        }
    }
}

pub type SharedActiveState = Arc<RwLock<ActiveState>>;

pub fn init_keystate() -> SharedActiveState {
    Arc::new(RwLock::new(ActiveState::new()))
}