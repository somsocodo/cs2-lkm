use std::thread;
use std::time::Duration;
use crossbeam::channel::Receiver;
use std::sync::{Arc, RwLock, atomic::{AtomicBool, Ordering}};
use rdev::{grab, simulate, Button, Event, EventType, SimulateError};
use driver::Driver;
use libc::INT_MAX;
use sdk::WeaponClass::{get_weapon_class, WeaponClass};
use sdk::CUtl::CUtlVector;
use sdk::Player::Player;
use sdk::Vector::{ Vector2, Vector3, get_fov };

use crate::{config::{ SharedConfig, SharedActiveState }, sdk::Player::SharedPlayerBase};

use cs2_dumper::offsets::cs2_dumper::offsets;
use cs2_dumper::libclient_so::cs2_dumper::schemas;

fn click() {
    match simulate(&EventType::ButtonPress(Button::Left)) {
        Ok(()) => simulate(&EventType::ButtonRelease(Button::Left)).unwrap(),
        Err(SimulateError) => {
            println!("Failed click.");
        }
    }
}

fn mouse_down() {
    match simulate(&EventType::ButtonPress(Button::Left)) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("Failed to press mouse button.");
        }
    }
}

fn mouse_up() {
    match simulate(&EventType::ButtonRelease(Button::Left)) {
        Ok(()) => (),
        Err(SimulateError) => {
            println!("Failed to release mouse button.");
        }
    }
}

fn mouse_grabber(will_aim: Arc<RwLock<bool>>, shared_keystate: SharedActiveState, shared_config: SharedConfig) {
    let click_scheduled = Arc::new(AtomicBool::new(false));
    let click_held = Arc::new(AtomicBool::new(false));

    thread::spawn(move || {
        if let Err(error) = grab(move |event: Event| {
            match event.event_type {
                EventType::ButtonPress(button) => {
                    if button == Button::Left {
                        {
                            let mut keystate = shared_keystate.write().unwrap();
                            keystate.aim = true;
                        }

                        let will_aim_flag = {
                            let will_aim = will_aim.read().unwrap();
                            *will_aim
                        };

                        let config = {
                            let config_read = shared_config.read().unwrap();
                            config_read.clone()
                        };

                        if will_aim_flag {
                            if !click_scheduled.load(Ordering::SeqCst) {
                                click_scheduled.store(true, Ordering::SeqCst);

                                let click_held_inner = Arc::clone(&click_held);
                                let click_scheduled_inner = Arc::clone(&click_scheduled);

                                thread::spawn(move || {
                                    thread::sleep(Duration::from_millis(config.aim_shoot_delay));
                                    mouse_down();
                                    click_held_inner.store(true, Ordering::SeqCst);

                                    if !click_scheduled_inner.load(Ordering::SeqCst) {
                                        mouse_up();
                                        click_held_inner.store(false, Ordering::SeqCst);
                                    }
                                });
                            }
                            return None;
                        }
                    }
                    Some(event)
                },
                EventType::ButtonRelease(button) => {
                    if button == Button::Left {
                        {
                            let mut keystate = shared_keystate.write().unwrap();
                            keystate.aim = false;
                        }

                        if click_held.load(Ordering::SeqCst) {
                            mouse_up();
                            click_held.store(false, Ordering::SeqCst);
                        }

                        click_scheduled.store(false, Ordering::SeqCst);
                    }
                    Some(event)
                },
                _ => Some(event),
            }
        }) {
            println!("Error: {:?}", error);
        }
    });
}

pub fn run_combat(
    driver: Driver, 
    shared_local_player: SharedPlayerBase,
    shared_activestate: SharedActiveState,
    shared_config: SharedConfig,
    player_receiver: Receiver<Vec<Player>>
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut target: Player = Player::default();
        let mut aimbot_ms: i64 = 0;
        let will_aim_shared = Arc::new(RwLock::new(false));

        let client_addr =  driver.read_module("libclient.so");
        let dwSensitivity: usize = driver.read_mem(client_addr + offsets::libclient_so::dwSensitivity);
        let sensitivity_raw: f32 = driver.read_mem(dwSensitivity + offsets::libclient_so::dwSensitivity_sensitivity);
        
        mouse_grabber(Arc::clone(&will_aim_shared), shared_activestate.clone(), shared_config.clone());

        loop {
            let config = {
                let config_read = shared_config.read().unwrap();
                config_read.clone()
            };

            let activestate = {
                let activestate_read = shared_activestate.read().unwrap();
                activestate_read.clone()
            };

            let local_player = {
                let local_player_read = shared_local_player.read().unwrap();
                local_player_read.clone()
            };

            let weapon_class = get_weapon_class(&driver, local_player.pawn);

            let mut closest_dist: f32 = INT_MAX as f32;
            let window_center = (config.window_size.0 as f32 / 2.0, config.window_size.1  as f32 / 2.0);

            let punch_angle: Vector2 = get_vec_punch(&driver, local_player.pawn);

            let fov_multipler: f32 = driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::m_flFOVSensitivityAdjust);
            let sensitivity = sensitivity_raw * fov_multipler;

            let shots_fired: u32 = driver.read_mem(local_player.pawn + schemas::libclient_so::C_CSPlayerPawn::m_iShotsFired);

            if let Ok(players) = player_receiver.recv() {
                for player in players.iter() {
                    if player.health == 0 || !player.bspotted {
                        continue;
                    }

                    if config.trigger_enabled && activestate.trigger && player.in_cross && punch_angle.x > -0.01 {
                        click();
                    }
                
                    let dist_x = (window_center.0 - player.bones_2d[0].x).abs();
                    let dist_y = (window_center.1 - player.bones_2d[0].y).abs();
                    let total_dist = dist_x + dist_y;

                    if total_dist < closest_dist {
                        closest_dist = total_dist;
                        target = player.clone();
                    }
                }
            }

            if !config.aim_enabled || closest_dist as i32 == INT_MAX || target.bones_3d[0].pos.is_zero() {
                let mut will_aim_write = will_aim_shared.try_write().unwrap();
                *will_aim_write = false;
                continue;
            }

            let target_pos = target.bones_3d[0].pos;
            let mut angles: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
            let view_angle: Vector2 = driver.read_mem(local_player.pawn + schemas::libclient_so::C_BasePlayerPawn::v_angle);
            let aimbot_angle = get_target_angle(&driver, target_pos, punch_angle, shots_fired, weapon_class, local_player.pawn);
            let aimbot_fov = get_fov(view_angle, aimbot_angle);

            if aimbot_fov > config.aim_fov {
                let mut will_aim_write = will_aim_shared.try_write().unwrap();
                *will_aim_write = false;    
                continue;
            }

            if !activestate.show_gui {
                let mut will_aim_write = will_aim_shared.write().unwrap();
                *will_aim_write = true;
            }
                
            
            angles.x = view_angle.x - aimbot_angle.x;
            angles.y = view_angle.y - aimbot_angle.y;
            angles.z = 0.0;

            angles.clamp();

            let x = (angles.y / sensitivity) / 0.022;
            let y = (angles.x / sensitivity) / -0.022;

            let mut smooth_x = 0.0f32;
            let mut smooth_y = 0.0f32;    

            let ms = if config.aim_smoothing >= 1.0 {
                if x.abs() > 1.0 {
                    if smooth_x < x {
                        smooth_x += 1.0 + (x / config.aim_smoothing);
                    } else if smooth_x > x {
                        smooth_x += (x / config.aim_smoothing) - 1.0;
                    } else {
                        smooth_x = x;
                    }
                } else {
                    smooth_x = x;
                }

                if y.abs() > 1.0 {
                    if smooth_y < y {
                        smooth_y += 1.0 + (y / config.aim_smoothing);
                    } else if smooth_y > y {
                        smooth_y += (y / config.aim_smoothing) - 1.0;
                    } else {
                        smooth_y = y;
                    }
                } else {
                    smooth_y = y;
                }
                ((config.aim_smoothing / 100.0) + 1.0) as i64 * 16
            } else {
                smooth_x = x;
                smooth_y = y;
                16
            };

            let now = std::time::SystemTime::now();
            let duration = now.duration_since(std::time::UNIX_EPOCH).expect("Time went backwards");
            let current_ms = duration.as_millis() as i64;

            if !activestate.show_gui && activestate.aim && current_ms - aimbot_ms > ms {
                aimbot_ms = current_ms;
                driver.send_input(0x02, 0, smooth_x as i32).unwrap();
                driver.send_input(0x02, 1, smooth_y as i32).unwrap();
            }

            thread::sleep(Duration::from_millis(1));
        }

    })
}


fn get_vec_punch(driver: &Driver, local_player: usize) -> Vector2 {
    let mut data = Vector2 { x: 0.0, y: 0.0 };
    let mut aim_punch_cache: [u64; 2] = [0, 0];
    
    let punch_cache: CUtlVector = driver.read_mem(local_player + schemas::libclient_so::C_CSPlayerPawn::m_aimPunchCache);

    aim_punch_cache[0] = punch_cache.count as u64;
    aim_punch_cache[1] = punch_cache.data as u64;

    if aim_punch_cache[0] == 0 || aim_punch_cache[1] == 0 {
        return data;
    }

    let aimpunch_size: u32 = (aim_punch_cache[0] & 0xFFFFFFFF) as u32;

    if aimpunch_size < 1 {
        return data;
    }

    let aimpunch_size = if aimpunch_size == 129 { 130 } else { aimpunch_size };

    data = driver.read_mem((aim_punch_cache[1] + ((aimpunch_size as u64 - 1) * 12)) as usize);

    if data.is_zero() {
        data = Vector2 { x: 0.0, y: 0.0 };
    }

    data
}

fn get_target_angle(
    driver: &Driver, 
    target: Vector3, 
    punch_angle: Vector2, 
    shots_fired: u32, 
    weapon_class: WeaponClass,
    local_player: usize
) -> Vector3 {
    let feet_pos: Vector3 = driver.read_mem(local_player + schemas::libclient_so::C_BasePlayerPawn::m_vOldOrigin);

    let eye_pos_offset: Vector3 = driver.read_mem(local_player + schemas::libclient_so::C_BaseModelEntity::m_vecViewOffset);
    let eye_pos = feet_pos + eye_pos_offset;

    let mut angle = Vector3 {
        x: target.x - eye_pos.x,
        y: target.y - eye_pos.y,
        z: target.z - eye_pos.z,
    };

    angle = angle.normalize();

    Vector3::vec_angles(angle, &mut angle);

    if shots_fired > 0 {
        match weapon_class {
            WeaponClass::Sniper | WeaponClass::Shotgun => {
            },
            WeaponClass::Pistol if shots_fired < 2 => {
            },
            _ => {
                angle.x -= punch_angle.x * 2.0;
                angle.y -= punch_angle.y * 2.0;
            }
        }
    }
    angle.clamp();

    angle
}