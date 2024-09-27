use std::ops::{Add, Sub, Mul, MulAssign};
use std::f32::consts::PI;

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

impl Default for Vector2 {
    fn default() -> Self {
        Vector2 { x: 0.0, y: 0.0 }
    }
}

impl Vector2 {
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
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

    pub fn normalize(&self) -> Vector3 {
        let magnitude = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if magnitude > 0.0 {
            Vector3 {
                x: self.x / magnitude,
                y: self.y / magnitude,
                z: self.z / magnitude,
            }
        } else {
            *self
        }
    }

    pub fn vec_angles(forward: Vector3, angles: &mut Vector3) {
        let tmp: f32;
        let mut yaw: f32;
        let mut pitch: f32;

        if forward.y == 0.0 && forward.x == 0.0 {
            yaw = 0.0;
            if forward.z > 0.0 {
                pitch = 270.0;
            } else {
                pitch = 90.0;
            }
        } else {
            yaw = (forward.y.atan2(forward.x) * 180.0 / PI) as f32;
            if yaw < 0.0 {
                yaw += 360.0;
            }
            tmp = (forward.x * forward.x + forward.y * forward.y).sqrt();
            pitch = (-forward.z).atan2(tmp) * 180.0 / PI;
            if pitch < 0.0 {
                pitch += 360.0;
            }
        }

        angles.x = pitch;
        angles.y = yaw;
        angles.z = 0.0;
    }

    pub fn clamp(&mut self) {
        if self.x > 89.0 && self.x <= 180.0 {
            self.x = 89.0;
        }
        if self.x > 180.0 {
            self.x = self.x - 360.0;
        }
        if self.x < -89.0 {
            self.x = -89.0;
        }

        self.y = ((self.y + 180.0) % 360.0) - 180.0;
        self.z = 0.0;
    }

    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
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

pub fn generate_transformation_matrix(pos: &Vector3, rot: &Vector4) -> [[f32; 4]; 3] {
    let mut matrix = [[0.0; 4]; 3];

    matrix[0][0] = 1.0 - 2.0 * rot.y * rot.y - 2.0 * rot.z * rot.z;
    matrix[1][0] = 2.0 * rot.x * rot.y + 2.0 * rot.w * rot.z;
    matrix[2][0] = 2.0 * rot.x * rot.z - 2.0 * rot.w * rot.y;

    matrix[0][1] = 2.0 * rot.x * rot.y - 2.0 * rot.w * rot.z;
    matrix[1][1] = 1.0 - 2.0 * rot.x * rot.x - 2.0 * rot.z * rot.z;
    matrix[2][1] = 2.0 * rot.y * rot.z + 2.0 * rot.w * rot.x;

    matrix[0][2] = 2.0 * rot.x * rot.z + 2.0 * rot.w * rot.y;
    matrix[1][2] = 2.0 * rot.y * rot.z - 2.0 * rot.w * rot.x;
    matrix[2][2] = 1.0 - 2.0 * rot.x * rot.x - 2.0 * rot.y * rot.y;

    matrix[0][3] = pos.x;
    matrix[1][3] = pos.y;
    matrix[2][3] = pos.z;

    matrix
}

pub fn apply_transformation_matrix(vec: &Vector3, matrix: [[f32; 4]; 3]) -> Vector3 {
    let mut result = Vector3 { x: 0.0, y: 0.0, z: 0.0 };

    result.x = matrix[0][0] * vec.x + matrix[0][1] * vec.y + matrix[0][2] * vec.z + matrix[0][3];
    result.y = matrix[1][0] * vec.x + matrix[1][1] * vec.y + matrix[1][2] * vec.z + matrix[1][3];
    result.z = matrix[2][0] * vec.x + matrix[2][1] * vec.y + matrix[2][2] * vec.z + matrix[2][3];

    result
}

pub fn vec2_diff(vec1: &Vector2, vec2: &Vector2) -> f32 {
    let diff_x = vec2.x - vec1.x;
    let diff_y = vec2.y - vec1.y;
    (diff_x * diff_x + diff_y * diff_y).sqrt()
}

pub fn vec_translate(origin: &Vector3, angles: &Vector2, dist: f32) -> Vector3 {
    let sp = (angles.x * (std::f32::consts::PI / 180.0)).sin();
    let sy = (angles.y * (std::f32::consts::PI / 180.0)).sin();
    let cp = (angles.x * (std::f32::consts::PI / 180.0)).cos();
    let cy = (angles.y * (std::f32::consts::PI / 180.0)).cos();

    let forward = Vector3 {
        x: cp * cy,
        y: cp * sy,
        z: -sp,
    };

    Vector3 {
        x: origin.x + forward.x * dist,
        y: origin.y + forward.y * dist,
        z: origin.z + forward.z * dist,
    }
}