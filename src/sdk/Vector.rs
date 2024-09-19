use std::ops::{Add, Sub, Mul, MulAssign};

#[derive(Copy, Clone)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32
}

#[derive(Copy, Clone)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Default for Vector3 {
    fn default() -> Self {
        Vector3 { x: 0.0, y: 0.0, z: 0.0 }
    }
}

impl Vector3 {
    pub fn world_to_screen(&self, vm: [[f32; 4]; 4]) -> Vector2 {
        // TODO
        let display_w: f32 = 1024.0;
        let display_h: f32 = 768.0;

        let screen_w = (vm[3][0] * self.x) + (vm[3][1] * self.y) + (vm[3][2] * self.z) + vm[3][3];

        if screen_w > 0.001 {
            let screen_x = (vm[0][0] * self.x) + (vm[0][1] * self.y) + (vm[0][2] * self.z) + vm[0][3];
            let screen_y = (vm[1][0] * self.x) + (vm[1][1] * self.y) + (vm[1][2] * self.z) + vm[1][3];

            let x = (display_w / 2.0) + (display_w / 2.0) * screen_x / screen_w;
            let y = (display_h / 2.0) - (display_h / 2.0) * screen_y / screen_w;
            Vector2 { x, y }
        } else {
            Vector2 { x: -99.0, y: -99.0 }
        }
    }
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Vector3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}