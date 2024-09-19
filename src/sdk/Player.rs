use driver::Driver;
use super::Vector::Vector4;
use super::Vector::Vector3;
use super::Vector::Vector2;
use super::CUtlString::CUtlString;

#[derive(Copy, Clone)]
pub struct PlayerBase {
    pub pawn: usize,
    pub controller: usize,
    pub idx: usize
}

impl PlayerBase {
    pub fn new(pawn: usize, controller: usize, idx: usize) -> Self {
        PlayerBase {
            pawn,
            controller,
            idx
        }
    }
}

pub struct Player {
    pub name: CUtlString,
    pub health: i32,
    pub pos: Vector3,
    pub pos_2d: Vector2,
    pub bones_2d: [Vector2; 30]
}

impl Player {
    pub fn new(name: CUtlString, health: i32, pos: Vector3, pos_2d: Vector2) -> Self {
        Player {
            name,
            health,
            pos,
            pos_2d,
            bones_2d: [Vector2 { x: 0.0, y: 0.0 }; 30]
        }
    }

    pub fn read_bones(&mut self, driver: Driver, bone_matrix: usize, view_matrix: [[f32; 4]; 4] ) {
        let bone_data: BoneJointDataArray = driver.read_mem(bone_matrix);

        for (i, bone_data) in bone_data.bone_array.iter().enumerate() {

            let bone_2d = bone_data.pos.world_to_screen(view_matrix);
            if bone_2d.x == -99.0 {
                continue;
            }
            self.bones_2d[i] = bone_2d;
        }  
    }
}

#[repr(C)]
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