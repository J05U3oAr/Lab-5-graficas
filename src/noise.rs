//! Continuous, deterministic 3D value noise. No lookup images or texture reads.
use crate::math::{v, Vec3};

pub fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32)
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263))
        .wrapping_add((z as u32).wrapping_mul(2_147_483_647));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

pub fn noise(p: Vec3) -> f32 {
    let cell = v(p.x.floor(), p.y.floor(), p.z.floor());
    let f = p - cell;
    let u = v(
        f.x * f.x * (3.0 - 2.0 * f.x),
        f.y * f.y * (3.0 - 2.0 * f.y),
        f.z * f.z * (3.0 - 2.0 * f.z),
    );
    let mut result = 0.0;
    for x in 0..=1 {
        for y in 0..=1 {
            for z in 0..=1 {
                let weight = (if x == 1 { u.x } else { 1.0 - u.x })
                    * (if y == 1 { u.y } else { 1.0 - u.y })
                    * (if z == 1 { u.z } else { 1.0 - u.z });
                result += weight * hash(cell.x as i32 + x, cell.y as i32 + y, cell.z as i32 + z);
            }
        }
    }
    result
}

pub fn fbm(mut p: Vec3, octaves: usize) -> f32 {
    let (mut result, mut amplitude, mut total) = (0.0, 0.5, 0.0);
    for _ in 0..octaves {
        result += noise(p) * amplitude;
        total += amplitude;
        p = p * 2.03 + v(7.1, 13.7, 3.9);
        amplitude *= 0.5;
    }
    result / total
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn noise_is_bounded_continuous_and_deterministic_across_negative_cells() {
        for i in -40..40 {
            let p = v(i as f32 * 0.13, -2.3, 0.77);
            let a = fbm(p, 5);
            assert!((0.0..=1.0).contains(&a));
            assert_eq!(a, fbm(p, 5));
            assert!((a - fbm(p + v(1e-4, 0.0, 0.0), 5)).abs() < 0.005);
        }
    }
}
