//! Small, explicit vector math used by the CPU renderer.
use std::ops::{Add, AddAssign, Div, Mul, Sub};

#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub const fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl Vec3 {
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn normalized(self) -> Self {
        self / self.length().max(1e-8)
    }
    pub fn rotate_y(self, a: f32) -> Self {
        let (s, c) = a.sin_cos();
        v(c * self.x + s * self.z, self.y, -s * self.x + c * self.z)
    }
    pub fn rotate_z(self, a: f32) -> Self {
        let (s, c) = a.sin_cos();
        v(c * self.x - s * self.y, s * self.x + c * self.y, self.z)
    }
    pub fn component_mul(self, rhs: Self) -> Self {
        v(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        v(self.x + b.x, self.y + b.y, self.z + b.z)
    }
}
impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        v(self.x - b.x, self.y - b.y, self.z - b.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        v(self.x * b, self.y * b, self.z * b)
    }
}
impl Div<f32> for Vec3 {
    type Output = Self;
    fn div(self, b: f32) -> Self {
        self * (1.0 / b)
    }
}
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

pub fn mix(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a * (1.0 - t) + b * t
}
pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub fn palette(t: f32, colors: &[Vec3]) -> Vec3 {
    let position = t.clamp(0.0, 1.0) * (colors.len() - 1) as f32;
    let index = (position as usize).min(colors.len() - 2);
    mix(colors[index], colors[index + 1], position - index as f32)
}
