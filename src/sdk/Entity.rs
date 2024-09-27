use super::CUtl::CUtlString;
use super::Vector::Vector3;
use super::Vector::Vector2;

#[derive(Copy, Clone)]
pub struct EntityBase {
    pub addr: usize,
    pub class_name: CUtlString,
    pub ammo: [i32; 2]
}

impl EntityBase {
    pub fn new(addr: usize, class_name: CUtlString) -> Self {
        EntityBase {
            addr,
            class_name,
            ammo: [-1, -1]
        }
    }
}

impl Default for EntityBase {
    fn default() -> Self {
        Self {
            addr: 0,
            class_name: CUtlString::default(),
            ammo: [-1, -1]
        }
    }
}

#[derive(Copy, Clone)]
pub struct Entity {
    pub addr: usize,
    pub class_name: CUtlString,
    pub pos: Vector3,
    pub pos_2d: Vector2,
    pub ammo: [i32; 2],
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            addr: 0,
            class_name: CUtlString::default(),
            pos: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            pos_2d: Vector2 { x: -99.0, y: -99.0 },
            ammo: [-1, -1]
        }
    }
}

impl Entity {
    pub fn new(addr: usize, class_name: CUtlString, pos: Vector3, pos_2d: Vector2, ammo: [i32; 2]) -> Self {
        Entity {
            addr,
            class_name,
            pos,
            pos_2d,
            ammo
        }
    }
}