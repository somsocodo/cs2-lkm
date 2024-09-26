use driver::Driver;
use std::sync::{Arc, RwLock};
use super::Vector::Vector4;
use super::Vector::Vector3;
use super::Vector::Vector2;
use super::Vector::generate_transformation_matrix;
use super::Vector::apply_transformation_matrix;
use super::Vector::vec_translate;
use super::Vector::vec2_diff;
use super::CUtl::CUtlString;

#[derive(Copy, Clone)]
pub struct PlayerBase {
    pub pawn: usize,
    pub controller: usize,
    pub idx: u32
}

impl PlayerBase {
    pub fn new(pawn: usize, controller: usize, idx: u32) -> Self {
        PlayerBase {
            pawn,
            controller,
            idx
        }
    }
}

impl Default for PlayerBase {
    fn default() -> Self {
        Self {
            pawn: 0,
            controller: 0,
            idx: 0
        }
    }
}

pub type SharedPlayerBase = Arc<RwLock<PlayerBase>>;

#[derive(Copy, Clone)]
pub struct Player {
    pub pawn: usize,
    pub controller: usize,
    pub idx: u32,
    pub name: CUtlString,
    pub health: i32,
    pub bspotted: bool,
    pub in_cross: bool,
    pub pos: Vector3,
    pub pos_2d: Vector2,
    pub bones_3d: [BoneData; 30],
    pub bones_2d: [Vector2; 30],
    pub hitboxes: [HitboxData; 30]
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pawn: 0,
            controller: 0,
            idx: 0,
            name: CUtlString::default(),
            health: 0,
            bspotted: false,
            in_cross: false,
            pos: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            pos_2d: Vector2 { x: -99.0, y: -99.0 },
            bones_3d: [BoneData {
                pos: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                scale: 0.0,
                rot: Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }
            }; 30],
            bones_2d: [Vector2 { x: -99.0, y: -99.0 }; 30],
            hitboxes: [HitboxData {
                min_bounds: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                max_bounds: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                shape_radius: 0.0,
                bone_idx: 0,
                min_bounds_2d: Vector2 { x: 0.0, y: 0.0 },
                max_bounds_2d: Vector2 { x: 0.0, y: 0.0 },
                min_rad_2d: 0.0,
                max_rad_2d: 0.0
            }; 30],
        }
    }
}

impl Player {
    pub fn new(pawn: usize, controller: usize, idx: u32, name: CUtlString, bspotted: bool, in_cross:bool, health: i32, pos: Vector3, pos_2d: Vector2) -> Self {
        Player {
            pawn,
            controller,
            idx,
            name,
            health,
            bspotted,
            in_cross,
            pos,
            pos_2d,
            bones_3d: [BoneData { pos: Vector3 { x: 0.0, y: 0.0, z: 0.0 }, scale: 0.0, rot: Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }}; 30],
            bones_2d: [Vector2 { x: 0.0, y: 0.0 }; 30],
            hitboxes: [HitboxData {
                min_bounds: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                max_bounds: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                shape_radius: 0.0,
                bone_idx: 0,
                min_bounds_2d: Vector2 { x: 0.0, y: 0.0 },
                max_bounds_2d: Vector2 { x: 0.0, y: 0.0 },
                min_rad_2d: 0.0,
                max_rad_2d: 0.0
            }; 30]
        }
    }

    pub fn read_bones(&mut self, driver: Driver, bone_matrix: usize, view_matrix: [[f32; 4]; 4] ) {
        let bone_data: BoneJointDataArray = driver.read_mem(bone_matrix);

        let bone_ids = [
            6,  // BONE_HEAD = 6
            5,  // BONE_NECK = 5
            4,  // BONE_SPINE_3 = 4
            3,  // BONE_SPINE_2 = 3
            2,  // BONE_SPINE_1 = 2
            1,  // BONE_SPINE_0 = 1
            0,  // BONE_HIP = 0
            8,  // BONE_LEFT_SHOULDER = 8
            9,  // BONE_LEFT_ARM = 9
            10, // BONE_LEFT_HAND = 10
            13, // BONE_RIGHT_SHOULDER = 13
            14, // BONE_RIGHT_ARM = 14
            15, // BONE_RIGHT_HAND = 15
            22, // BONE_LEFT_HIP = 22
            23, // BONE_LEFT_KNEE = 23
            24, // BONE_LEFT_FEET = 24
            25, // BONE_RIGHT_HIP = 25
            26, // BONE_RIGHT_KNEE = 26
            27, // BONE_RIGHT_FEET = 27
        ];

        for (i, &bone_id) in bone_ids.iter().enumerate() {
            let bone_data = &bone_data.bone_array[bone_id];  // Access bone data using the bone_id

            let bone_2d = bone_data.pos.world_to_screen(view_matrix);
            if bone_2d.x == -99.0 {
                continue;
            }
            self.bones_2d[i] = bone_2d;
            self.bones_3d[i] = bone_data.clone();  // Store bone data in the corresponding index
        }
    }

    pub fn read_hitboxes(&mut self, view_angle: Vector2, view_matrix: [[f32; 4]; 4]){
        for (i, hitbox) in HITBOXES.iter().enumerate() {
            let bone = self.bones_3d[hitbox.bone_idx];
            let transformation_matrix = generate_transformation_matrix(&bone.pos, &bone.rot);
            let bbmax_trans = apply_transformation_matrix(&hitbox.max_bounds, transformation_matrix);
            let bbmin_trans = apply_transformation_matrix(&hitbox.min_bounds, transformation_matrix);
    
            let angle_diff = Vector2 {
                x: view_angle.x + 90.0,
                y: view_angle.y + 90.0,
            };
    
            let bbmax_trans_rad = vec_translate(&bbmax_trans, &angle_diff, hitbox.shape_radius);
            let bbmin_trans_rad = vec_translate(&bbmin_trans, &angle_diff, hitbox.shape_radius);
    
            let mut modifiable_hitbox = *hitbox;
    
            modifiable_hitbox.min_bounds_2d = bbmin_trans.world_to_screen(view_matrix);
            modifiable_hitbox.max_bounds_2d = bbmax_trans.world_to_screen(view_matrix);

    
            modifiable_hitbox.min_rad_2d = vec2_diff(
                &bbmin_trans_rad.world_to_screen(view_matrix),
                &bbmin_trans.world_to_screen(view_matrix),
            );
            modifiable_hitbox.max_rad_2d = vec2_diff(
                &bbmax_trans_rad.world_to_screen(view_matrix),
                &bbmax_trans.world_to_screen(view_matrix),
            );
    
            self.hitboxes[i] = modifiable_hitbox;
        }
    }
}

#[repr(C)] #[derive(Copy, Clone)]
pub struct BoneData {
    pub pos: Vector3,
    pub scale: f32,
    pub rot: Vector4
}

#[repr(C)]
struct BoneJointDataArray {
    bone_array: [BoneData; 30]
}

impl Default for BoneJointDataArray {
    fn default() -> Self {
        Self {
            bone_array: std::array::from_fn(|_| BoneData {
                pos: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
                scale: 1.0,
                rot: Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            }),
        }
    }
}

#[derive(Copy, Clone)]
pub struct HitboxData {
    pub min_bounds: Vector3,
    pub max_bounds: Vector3,
    pub shape_radius: f32,
    pub bone_idx: usize,
    pub min_bounds_2d: Vector2,
    pub max_bounds_2d: Vector2,
    pub min_rad_2d: f32,
    pub max_rad_2d: f32,
}

impl HitboxData {
    pub const fn new(min_bounds: Vector3, max_bounds: Vector3, shape_radius: f32, bone_idx: usize) -> Self {
        Self {
            min_bounds,
            max_bounds,
            shape_radius,
            bone_idx,
            min_bounds_2d: Vector2 { x: 0.0, y: 0.0 },
            max_bounds_2d: Vector2 { x: 0.0, y: 0.0 },
            min_rad_2d: 0.0,
            max_rad_2d: 0.0,
        }
    }

}

pub const HITBOXES: [HitboxData; 19] = [
    HitboxData::new(Vector3 { x: -1.0, y: 1.8, z: 0.0 }, Vector3 { x: 3.5, y: 0.2, z: 0.0 }, 4.3, 0),
    HitboxData::new(Vector3 { x: 0.0, y: -0.4, z: 0.0 }, Vector3 { x: 1.4, y: -0.2, z: 0.0 }, 3.5, 1),
    HitboxData::new(Vector3 { x: -2.7, y: 1.1, z: -3.2 }, Vector3 { x: -2.7, y: 1.1, z: 3.2 }, 6.0, 6),
    HitboxData::new(Vector3 { x: 1.4, y: 0.8, z: 3.1 }, Vector3 { x: 1.4, y: 0.8, z: -3.1 }, 6.0, 5),
    HitboxData::new(Vector3 { x: 3.8, y: 0.8, z: -2.4 }, Vector3 { x: 3.8, y: 0.4, z: 2.4 }, 6.5, 4),
    HitboxData::new(Vector3 { x: 4.8, y: 0.15, z: -4.1 }, Vector3 { x: 4.8, y: 0.15, z: 4.1 }, 6.2, 3),
    HitboxData::new(Vector3 { x: 2.5, y: -0.6, z: -6.0 }, Vector3 { x: 2.5, y: -0.6, z: 6.0 }, 5.0, 2),
    HitboxData::new(Vector3 { x: 1.3, y: -0.2, z: 0.0 }, Vector3 { x: 16.5, y: -0.7, z: 0.0 }, 5.0, 13),
    HitboxData::new(Vector3 { x: -1.3, y: 0.0, z: -0.6 }, Vector3 { x: -16.5, y: 0.0, z: -0.7 }, 5.0, 16),
    HitboxData::new(Vector3 { x: 0.1, y: -0.4, z: 0.2 }, Vector3 { x: 17.0, y: -0.4, z: 0.7 }, 4.0, 14),
    HitboxData::new(Vector3 { x: -0.1, y: 0.0, z: -0.2 }, Vector3 { x: -17.0, y: 0.4, z: -0.7 }, 4.0, 17),
    HitboxData::new(Vector3 { x: 0.0, y: -3.43, z: -0.52 }, Vector3 { x: 8.0, y: 0.74, z: 0.33 }, 2.6, 15),
    HitboxData::new(Vector3 { x: -7.98, y: -0.75, z: -0.27 }, Vector3 { x: -0.02, y: 3.44, z: 0.58 }, 2.6, 18),
    HitboxData::new(Vector3 { x: 0.0, y: 0.3, z: 0.0 }, Vector3 { x: 3.59, y: 1.15, z: 0.11 }, 2.3, 9),
    HitboxData::new(Vector3 { x: 0.0, y: -0.3, z: 0.02 }, Vector3 { x: -3.44, y: -1.17, z: -0.09 }, 2.3, 12),
    HitboxData::new(Vector3 { x: 0.0, y: 0.0, z: 0.0 }, Vector3 { x: 11.2, y: 0.0, z: 0.0 }, 3.3, 7),
    HitboxData::new(Vector3 { x: 0.0, y: 0.0, z: 0.0 }, Vector3 { x: 10.0, y: 0.0, z: 0.0 }, 3.0, 8),
    HitboxData::new(Vector3 { x: 0.0, y: 0.0, z: 0.0 }, Vector3 { x: -11.2, y: 0.0, z: 0.0 }, 3.3, 10),
    HitboxData::new(Vector3 { x: 0.0, y: 0.0, z: 0.0 }, Vector3 { x: -10.0, y: 0.0, z: -0.5 }, 3.0, 11)
];