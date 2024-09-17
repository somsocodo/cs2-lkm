use super::Vector::Vector3;
use super::Vector::Vector2;
use super::CUtlString::CUtlString;

pub struct Player {
    pub pawn: usize,
    pub name: CUtlString,
    pub health: i32,
    pub position: Vector3,
    pub position_2d: Vector2
}

impl Player {
    pub fn new(pawn: usize, name: CUtlString, health: i32, position: Vector3, position_2d: Vector2) -> Self {
        Player {
            pawn,
            name,
            health,
            position,
            position_2d
        }
    }
}