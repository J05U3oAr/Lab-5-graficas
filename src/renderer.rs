//! Ray casting analítico en CPU con profundidad por píxel y sombras geométricas.
//! minifb solo recibe el framebuffer terminado; no calcula ningún shader.
use crate::math::{mix, smoothstep, v, Vec3};
use crate::noise::hash;
use crate::scene::{Body, Ring, Scene, Sphere};
use crate::shaders::{ring_pattern, shade};
use std::thread;

const FAR: f32 = f32::INFINITY;
const EPSILON: f32 = 0.001;

pub fn sphere_hit(origin: Vec3, direction: Vec3, sphere: &Sphere) -> f32 {
    let oc = origin - sphere.center;
    let b = oc.dot(direction);
    let discriminant = b * b - oc.dot(oc) + sphere.radius * sphere.radius;
    if discriminant < 0.0 {
        return FAR;
    }
    let root = discriminant.sqrt();
    let near = -b - root;
    let far = -b + root;
    if near > EPSILON {
        near
    } else if far > EPSILON {
        far
    } else {
        FAR
    }
}

pub fn ring_hit(origin: Vec3, direction: Vec3, ring: &Ring) -> Option<(f32, f32)> {
    let denominator = direction.dot(ring.normal);
    if denominator.abs() < 1e-7 {
        return None;
    }
    let distance = (ring.center - origin).dot(ring.normal) / denominator;
    if distance <= EPSILON {
        return None;
    }
    let radius = (origin + direction * distance - ring.center).length() / ring.planet_radius;
    if (1.26..=2.12).contains(&radius) {
        Some((distance, radius))
    } else {
        None
    }
}

fn light_at(point: Vec3, system: bool) -> (Vec3, f32) {
    if system {
        let offset = v(0.0, 0.0, 0.0) - point;
        (offset.normalized(), offset.length())
    } else {
        (v(-3.5, 4.2, 6.0).normalized(), FAR)
    }
}

fn visibility(
    point: Vec3,
    light: Vec3,
    limit: f32,
    scene: &Scene,
    skip: Option<usize>,
    include_rings: bool,
) -> f32 {
    let origin = point + light * EPSILON * 2.0;
    for (i, sphere) in scene.spheres.iter().enumerate() {
        if Some(i) == skip || sphere.body == Body::Aurelia {
            continue;
        }
        if sphere_hit(origin, light, sphere) < limit {
            return 0.0;
        }
    }
    let mut transmission = 1.0;
    if include_rings {
        for ring in &scene.rings {
            if let Some((distance, radius)) = ring_hit(origin, light, ring) {
                if distance < limit {
                    transmission *= 1.0 - ring_pattern(radius, ring.body).1 * 0.86;
                }
            }
        }
    }
    transmission
}

fn background(x: usize, y: usize, width: usize, height: usize) -> Vec3 {
    let dx = x as f32 / width as f32 - 0.5;
    let dy = y as f32 / height as f32 - 0.5;
    let vignette = (1.0 - dx * dx - dy * dy).max(0.0);
    let mut color = v(0.0015, 0.0025, 0.006) + v(0.001, 0.0015, 0.003) * vignette;
    let seed = hash(x as i32, y as i32, 2056);
    if seed > 0.9995 {
        let tint = if hash(x as i32, y as i32, 91) > 0.3 {
            v(0.8, 0.88, 1.0)
        } else {
            v(1.0, 0.79, 0.53)
        };
        color += tint * (0.025 + (seed - 0.9995) * 400.0);
    }
    color
}

#[derive(Clone, Copy)]
pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub samples: usize,
}

impl Renderer {
    pub fn new(width: usize, height: usize, samples: usize) -> Self {
        Self {
            width,
            height,
            samples,
        }
    }

    pub fn render(&self, scene: &Scene, time: f32) -> Vec<u32> {
        let mut frame = vec![0; self.width * self.height];
        let threads = thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(self.height);
        let rows = self.height.div_ceil(threads);
        thread::scope(|scope| {
            for (chunk, buffer) in frame.chunks_mut(rows * self.width).enumerate() {
                scope.spawn(move || {
                    for (index, pixel) in buffer.iter_mut().enumerate() {
                        let x = index % self.width;
                        let y = chunk * rows + index / self.width;
                        let mut color = v(0.0, 0.0, 0.0);
                        for sy in 0..self.samples {
                            for sx in 0..self.samples {
                                let px = x as f32 + (sx as f32 + 0.5) / self.samples as f32;
                                let py = y as f32 + (sy as f32 + 0.5) / self.samples as f32;
                                let origin = v(
                                    (px / self.width as f32 - 0.5)
                                        * scene.height
                                        * self.width as f32
                                        / self.height as f32
                                        + scene.target.x,
                                    (0.5 - py / self.height as f32) * scene.height + scene.target.y,
                                    12.0,
                                );
                                color += self.trace(
                                    origin,
                                    scene,
                                    time,
                                    background(x, y, self.width, self.height),
                                );
                            }
                        }
                        *pixel = to_srgb(color / (self.samples * self.samples) as f32);
                    }
                });
            }
        });
        frame
    }

    fn trace(&self, origin: Vec3, scene: &Scene, time: f32, mut color: Vec3) -> Vec3 {
        let direction = v(0.0, 0.0, -1.0);
        // Atmospheric halos are analytic functions of distance to the silhouette.
        for sphere in &scene.spheres {
            if sphere.body == Body::Moon {
                continue;
            }
            let dx = origin.x - sphere.center.x;
            let dy = origin.y - sphere.center.y;
            let impact = (dx * dx + dy * dy).sqrt() / sphere.radius;
            if sphere.body == Body::Aurelia {
                let radial = (impact - 1.0).max(0.0);
                let theta = dy.atan2(dx);
                let rays = 0.72
                    + 0.28
                        * (theta * 31.0 + (theta * 13.0).sin() + time * 0.2)
                            .sin()
                            .powi(2);
                let glow = (-radial * 10.0).exp() * 0.9 + (-radial * 2.9).exp() * 0.085 * rays;
                color += v(1.0, 0.22, 0.012) * glow;
            } else if impact < 1.2 {
                color +=
                    sphere.body.atmosphere() * ((-(impact - 1.0).max(0.0) * 52.0).exp() * 0.12);
            }
        }
        if scene.system {
            let distance = (origin.x * origin.x + (origin.y / 0.48).powi(2)).sqrt();
            for i in 0..5 {
                let line =
                    1.0 - smoothstep(0.008, 0.025, (distance - (1.65 + i as f32 * 1.02)).abs());
                color += v(0.008, 0.013, 0.019) * line;
            }
        }
        // A per-pixel z-buffer chooses the nearest opaque surface.
        let mut depth = FAR;
        let mut nearest = None;
        for (i, sphere) in scene.spheres.iter().enumerate() {
            let distance = sphere_hit(origin, direction, sphere);
            if distance < depth {
                depth = distance;
                nearest = Some(i);
            }
        }
        if let Some(index) = nearest {
            let sphere = &scene.spheres[index];
            let point = origin + direction * depth;
            let normal = (point - sphere.center) / sphere.radius;
            // Inverse object transform attaches the procedural color to the rotating body.
            let p = normal.rotate_z(-sphere.tilt).rotate_y(-sphere.angle);
            let surface = shade(sphere.body, p, time);
            if sphere.body == Body::Aurelia {
                color = surface.color * (0.42 + 0.58 * normal.z.max(0.0));
            } else {
                let (light, limit) = light_at(point, scene.system);
                let diffuse = normal.dot(light).max(0.0);
                let vis = visibility(point, light, limit, scene, Some(index), true);
                let illumination = 0.026 + diffuse * vis * 1.12;
                color =
                    surface.color * (illumination * (1.0 - surface.emission) + surface.emission);
                if sphere.body != Body::Moon {
                    let rim = (1.0 - normal.z.max(0.0)).powf(3.6);
                    color += sphere.body.atmosphere() * (rim * (0.06 + diffuse * 0.45));
                }
                if sphere.body == Body::Thalassa {
                    let half = (light + v(0.0, 0.0, 1.0)).normalized();
                    let specular = normal.dot(half).max(0.0).powi(100) * 0.19 * vis;
                    color += v(specular, specular, specular);
                }
            }
        }
        // The two independent translucent annuli are sorted far to near.
        let mut hits = [None, None];
        for (slot, ring) in hits.iter_mut().zip(&scene.rings) {
            *slot = ring_hit(origin, direction, ring)
                .filter(|(distance, _)| *distance < depth)
                .map(|(distance, radius)| (distance, radius, ring));
        }
        if hits[0].map_or(0.0, |h| h.0) < hits[1].map_or(0.0, |h| h.0) {
            hits.swap(0, 1);
        }
        for (distance, radius, ring) in hits.into_iter().flatten() {
            let point = origin + direction * distance;
            let (ring_color, opacity) = ring_pattern(radius, ring.body);
            let (light, limit) = light_at(point, scene.system);
            let visible = visibility(point, light, limit, scene, None, false);
            let diffuse = light.dot(ring.normal).abs();
            let lit = ring_color * (0.07 + visible * (0.45 + 0.55 * diffuse));
            color = mix(color, lit, opacity);
        }
        color
    }
}

fn to_srgb(color: Vec3) -> u32 {
    fn channel(c: f32) -> u32 {
        ((1.0 - (-c.max(0.0) * 1.22).exp())
            .clamp(0.0, 1.0)
            .powf(1.0 / 2.2)
            * 255.0
            + 0.5) as u32
    }
    (channel(color.x) << 16) | (channel(color.y) << 8) | channel(color.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ray_intersections_handle_misses_inside_rays_and_parallel_planes() {
        let scene = Scene::new(Body::Vesper, 0.0, false, false);
        let sphere = &scene.spheres[0];
        assert!((sphere_hit(v(0.0, 0.0, 3.0), v(0.0, 0.0, -1.0), sphere) - 2.0).abs() < 1e-5);
        assert_eq!(sphere_hit(v(2.0, 0.0, 3.0), v(0.0, 0.0, -1.0), sphere), FAR);
        assert!((sphere_hit(v(0.0, 0.0, 0.0), v(0.0, 0.0, 1.0), sphere) - 1.0).abs() < 1e-5);
        let ring = Ring {
            center: v(0.0, 0.0, 0.0),
            normal: v(0.0, 0.0, 1.0),
            planet_radius: 1.0,
            body: Body::Vesper,
        };
        assert!(ring_hit(v(0.0, 0.0, 3.0), v(0.0, 0.0, -1.0), &ring).is_none());
        assert!(ring_hit(v(1.5, 0.0, 3.0), v(1.0, 0.0, 0.0), &ring).is_none());
        assert!(ring_hit(v(1.5, 0.0, 3.0), v(0.0, 0.0, -1.0), &ring).is_some());
    }
    #[test]
    fn planet_casts_a_shadow_on_its_separate_ring() {
        let scene = Scene::new(Body::Vesper, 0.0, false, false);
        assert_eq!(
            visibility(
                v(0.0, 0.0, -2.0),
                v(0.0, 0.0, 1.0),
                FAR,
                &scene,
                None,
                false
            ),
            0.0
        );
        assert_eq!(
            visibility(
                v(3.0, 0.0, -2.0),
                v(0.0, 0.0, 1.0),
                FAR,
                &scene,
                None,
                false
            ),
            1.0
        );
    }
    #[test]
    fn opaque_depth_is_independent_of_sphere_submission_order() {
        let renderer = Renderer::new(80, 60, 1);
        let scene = Scene::new(Body::Thalassa, 0.0, false, false);
        let mut reversed = scene.clone();
        reversed.spheres.reverse();
        assert_eq!(
            renderer.render(&scene, 0.0),
            renderer.render(&reversed, 0.0)
        );
    }

    #[test]
    fn procedural_rings_cast_translucent_shadows() {
        let mut scene = Scene::new(Body::Vesper, 0.0, false, false);
        scene.rings[0].normal = v(0.0, 0.0, 1.0);
        let transmission = visibility(v(1.5, 0.0, -2.0), v(0.0, 0.0, 1.0), FAR, &scene, None, true);
        assert!(transmission > 0.0 && transmission < 1.0);
    }

    #[test]
    fn time_moves_bodies_and_changes_the_rendered_system() {
        let renderer = Renderer::new(120, 90, 1);
        let first = Scene::new(Body::Vesper, 0.0, true, true);
        let later = Scene::new(Body::Vesper, 8.0, true, true);
        assert!((first.spheres[1].center - later.spheres[1].center).length() > 0.1);
        assert_ne!(renderer.render(&first, 0.0), renderer.render(&later, 8.0));
    }
}
