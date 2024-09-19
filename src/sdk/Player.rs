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
    pub pos_2d: Vector2
}

impl Player {
    pub fn new(name: CUtlString, health: i32, pos: Vector3, pos_2d: Vector2) -> Self {
        Player {
            name,
            health,
            pos,
            pos_2d
        }
    }
}