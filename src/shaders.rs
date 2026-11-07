//! Shaders de color en CPU. p = posición normalizada en espacio del objeto.
//! Todas las capas son funciones matemáticas: no hay texturas ni materiales.
use crate::math::{mix, palette, smoothstep as ss, v, Vec3};
use crate::noise::{fbm, noise};
use crate::scene::Body;

#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub color: Vec3,
    pub emission: f32,
}

pub fn shade(body: Body, p: Vec3, time: f32) -> Surface {
    match body {
        Body::Aurelia => aurelia(p, time),
        Body::Nyx => nyx(p),
        Body::Thalassa => thalassa(p, time),
        Body::Vesper => vesper(p),
        Body::Iris => iris(p),
        Body::Borealis => borealis(p, time),
        Body::Moon => moon(p),
    }
}

fn aurelia(p: Vec3, time: f32) -> Surface {
    // Capa 1: convección multiescala y deformación del dominio.
    let flow = fbm(p * 3.4 + v(0.0, time * 0.04, 0.0), 3);
    let q = p * 16.0 + v(flow, flow, flow) * 2.5;
    let cells = fbm(q + v(time * 0.08, 0.0, 0.0), 4);
    let mut color = palette(
        cells,
        &[
            v(0.24, 0.005, 0.018),
            v(0.85, 0.035, 0.006),
            v(1.8, 0.32, 0.018),
            v(2.5, 1.0, 0.15),
            v(3.0, 2.1, 0.8),
        ],
    );
    // Capa 2: bordes brillantes entre gránulos de plasma.
    let filaments = (-(noise(q * 2.4) - 0.5).abs() * 26.0).exp();
    color += v(0.65, 0.30, 0.035) * filaments;
    // Capa 3: agrupaciones de manchas solares frías.
    let spots = ss(0.63, 0.76, fbm(p * 5.8 + v(41.0, 41.0, 41.0), 3));
    color = color * (1.0 - spots * 0.88);
    // Capa 4: arcos magnéticos restringidos a regiones activas.
    let arcs = (-(p.y * 19.0 + flow * 12.0).sin().abs() * 12.0).exp();
    let active = ss(0.48, 0.72, fbm(p * 3.0 - v(17.0, 17.0, 17.0), 3));
    color += v(1.2, 0.52, 0.09) * (arcs * active);
    // Capa 5: turbulencia fina dentro de las zonas luminosas.
    color = color * (0.8 + 0.4 * noise(p * 85.0));
    Surface {
        color,
        emission: 1.0,
    }
}

fn nyx(p: Vec3) -> Surface {
    // Capa 1: corteza de basalto deformada con fBm.
    let warp = fbm(p * 3.4 + v(19.0, 19.0, 19.0), 3);
    let terrain = fbm(p * 8.0 + v(warp, warp, warp) * 3.0, 5);
    let mut color = palette(
        terrain,
        &[
            v(0.012, 0.016, 0.04),
            v(0.035, 0.055, 0.09),
            v(0.10, 0.14, 0.19),
            v(0.24, 0.29, 0.32),
        ],
    );
    // Capa 2: cordilleras y estratos minerales, sin desplazar la esfera.
    let ridges = 1.0 - (fbm(p * 25.0, 3) * 2.0 - 1.0).abs();
    color = color * (0.55 + 0.65 * ridges);
    // Capa 3: red conectada de lava con temperatura variable.
    let crack_field = noise(p * 7.5 + v(warp, warp, warp) * 2.2);
    let fissures = 1.0 - ss(0.013, 0.055, (crack_field - 0.5).abs());
    let hot = palette(
        noise(p * 19.0),
        &[
            v(0.5, 0.025, 0.015),
            v(1.8, 0.12, 0.01),
            v(2.6, 0.8, 0.07),
            v(3.0, 1.5, 0.45),
        ],
    );
    color = mix(color, hot, fissures);
    // Capa 4: cuencas y bordes de cráteres por distancia angular.
    for (center, radius) in [
        (v(0.36, 0.32, 0.88), 0.24),
        (v(-0.52, -0.19, 0.83), 0.17),
        (v(0.24, -0.62, 0.75), 0.13),
        (v(-0.12, 0.7, 0.7), 0.12),
    ] {
        let d = (p - center.normalized()).length();
        let bowl = 1.0 - ss(radius * 0.66, radius, d);
        let rim = (-((d - radius) / 0.016).powi(2)).exp();
        color = color * (1.0 - bowl * 0.65) + v(0.2, 0.23, 0.27) * rim;
    }
    // Capa 5: depósitos oxidados, separados de la lava.
    let mineral = ss(0.64, 0.78, fbm(p * 13.0 + v(81.0, 81.0, 81.0), 3)) * (1.0 - fissures);
    Surface {
        color: mix(color, v(0.34, 0.13, 0.07), mineral * 0.6),
        emission: fissures * 0.88,
    }
}

fn thalassa(p: Vec3, time: f32) -> Surface {
    // Capa 1: continentes y archipiélagos por elevación procedural.
    let warp = fbm(p * 2.8 + v(8.0, 8.0, 8.0), 3);
    let elevation = fbm(p * 3.5 + v(warp, warp, warp) * 1.4, 5);
    let land = ss(0.49, 0.525, elevation);
    // Capa 2: océano profundo y plataforma costera turquesa.
    let ocean = palette(
        ss(0.30, 0.51, elevation),
        &[
            v(0.004, 0.025, 0.08),
            v(0.008, 0.10, 0.19),
            v(0.015, 0.31, 0.34),
            v(0.11, 0.61, 0.52),
        ],
    );
    // Capa 3: vegetación, desiertos cobrizos y nieve en las cumbres.
    let dry = fbm(p * 8.0 + v(51.0, 51.0, 51.0), 3);
    let mut ground = palette(
        dry,
        &[
            v(0.025, 0.13, 0.10),
            v(0.12, 0.26, 0.15),
            v(0.48, 0.25, 0.12),
            v(0.72, 0.47, 0.25),
        ],
    );
    ground = mix(ground, v(0.8, 0.87, 0.85), ss(0.64, 0.73, elevation));
    let mut color = mix(ocean, ground, land);
    // Capa 4: casquetes polares de borde irregular.
    let ice = ss(0.77, 0.89, p.y.abs() + (dry - 0.5) * 0.15);
    color = mix(color, v(0.72, 0.9, 0.94), ice);
    // Capa 5: frentes nubosos que giran independientemente del suelo.
    let q = p.rotate_y(time * 0.018);
    let clouds = ss(0.57, 0.73, fbm(q * 7.0 + v(23.0, 0.0, 4.0), 5)) * 0.88;
    Surface {
        color: mix(color, v(0.88, 0.95, 0.94), clouds),
        emission: 0.0,
    }
}

fn vesper(p: Vec3) -> Surface {
    // Capa 1: turbulencia que curva los límites de las bandas.
    let turbulence = fbm(p.component_mul(v(4.0, 10.0, 4.0)) + v(7.0, 7.0, 7.0), 4);
    let flow = p.y * 27.0 + (turbulence - 0.5) * 4.0;
    // Capa 2: cinturones anchos de jade, crema y ocre.
    let mut color = palette(
        0.5 + 0.5 * flow.sin(),
        &[
            v(0.035, 0.12, 0.16),
            v(0.13, 0.28, 0.29),
            v(0.48, 0.55, 0.44),
            v(0.83, 0.68, 0.43),
            v(0.96, 0.85, 0.64),
        ],
    );
    // Capa 3: filamentos finos de nubes altas.
    color = color * (0.87 + 0.13 * (flow * 4.8 + fbm(p * 24.0, 3) * 3.0).sin());
    // Capa 4: anticiclón elíptico con paredes de nubes en espiral.
    let (dx, dy) = ((p.x - 0.31) / 0.31, (p.y + 0.23) / 0.14);
    let radius = (dx * dx + dy * dy).sqrt();
    let spiral = 0.5 + 0.5 * (radius * 19.0 - dy.atan2(dx) * 2.0 + turbulence * 3.0).sin();
    let storm = palette(
        spiral,
        &[
            v(0.19, 0.055, 0.035),
            v(0.55, 0.19, 0.075),
            v(0.96, 0.57, 0.27),
            v(1.0, 0.85, 0.56),
        ],
    );
    color = mix(
        color,
        storm,
        (1.0 - ss(0.78, 1.22, radius)) * ss(0.1, 0.4, p.z),
    );
    // Capas 5 y 6: grano fino y bruma polar fría.
    color = color * (0.9 + 0.18 * noise(p * 100.0));
    Surface {
        color: mix(color, v(0.3, 0.49, 0.48), ss(0.72, 0.98, p.y.abs()) * 0.6),
        emission: 0.0,
    }
}

fn iris(p: Vec3) -> Surface {
    // Capa 1: corrientes violetas de alta frecuencia.
    let turbulent = fbm(p.component_mul(v(5.0, 12.0, 5.0)) + v(31.0, 31.0, 31.0), 4);
    let flow = p.y * 42.0 + turbulent * 5.0;
    let mut color = palette(
        0.5 + 0.5 * flow.sin(),
        &[
            v(0.045, 0.025, 0.13),
            v(0.18, 0.12, 0.35),
            v(0.48, 0.34, 0.61),
            v(0.81, 0.63, 0.76),
            v(0.96, 0.82, 0.71),
        ],
    );
    // Capas 2 y 3: filamentos claros y huecos de nubes profundas.
    color = color * (0.82 + 0.18 * (flow * 3.7 + turbulent * 5.0).sin());
    let gaps = ss(0.58, 0.75, fbm(p * 12.0 + v(71.0, 71.0, 71.0), 3));
    color = mix(color, v(0.055, 0.04, 0.17), gaps * 0.7);
    // Capa 4: sistemas de vórtices compactos con espirales.
    for (x, y, size) in [
        (-0.30, 0.30, 0.18),
        (0.43, -0.16, 0.15),
        (-0.06, -0.48, 0.12),
    ] {
        let (dx, dy) = ((p.x - x) / size, (p.y - y) / (size * 0.56));
        let r = (dx * dx + dy * dy).sqrt();
        let swirl = 0.5 + 0.5 * (r * 17.0 - dy.atan2(dx) * 2.0).cos();
        let mask = (1.0 - ss(0.6, 1.2, r)) * ss(0.1, 0.5, p.z);
        color = mix(
            color,
            palette(
                swirl,
                &[
                    v(0.08, 0.04, 0.22),
                    v(0.56, 0.33, 0.65),
                    v(0.99, 0.77, 0.64),
                ],
            ),
            mask,
        );
    }
    // Capa 5: bruma cian en los polos.
    Surface {
        color: mix(color, v(0.22, 0.7, 0.73), ss(0.80, 0.99, p.y.abs()) * 0.65),
        emission: 0.0,
    }
}

fn borealis(p: Vec3, time: f32) -> Surface {
    // Capa 1: grandes regiones de hielo azul profundo.
    let warp = fbm(p * 4.0 + v(12.0, 12.0, 12.0), 4);
    let terrain = fbm(p * 7.0 + v(warp, warp, warp) * 2.0, 4);
    let mut color = palette(
        terrain,
        &[
            v(0.012, 0.035, 0.12),
            v(0.035, 0.15, 0.30),
            v(0.17, 0.43, 0.57),
            v(0.61, 0.81, 0.83),
        ],
    );
    // Capa 2: fracturas ramificadas de hielo luminoso.
    let fractures = 1.0
        - ss(
            0.01,
            0.05,
            (noise(p * 9.0 + v(warp, warp, warp)) - 0.5).abs(),
        );
    color = mix(color, v(0.28, 0.88, 0.97), fractures * 0.72);
    // Capa 3: nieve depositada por el viento, concentrada en los polos.
    let frost = ss(0.50, 0.76, terrain + p.y.abs() * 0.16);
    color = mix(color, v(0.86, 0.93, 0.96), frost);
    // Capa 4: estrías cristalinas finas.
    let crystal = ((p.x + p.z) * 95.0 + warp * 16.0).sin().abs().powi(16);
    color += v(0.035, 0.065, 0.08) * crystal;
    // Capa 5: cortinas de aurora verdes animadas a alta latitud.
    let curtain = (0.5 + 0.5 * (p.z.atan2(p.x) * 24.0 + warp * 12.0 + time * 0.3).sin()).powi(4);
    let aurora = (-((p.y.abs() - 0.79) / 0.075).powi(2)).exp() * curtain;
    color += v(0.12, 0.9, 0.43) * aurora;
    Surface {
        color,
        emission: (aurora * 0.7).max(fractures * 0.12),
    }
}

fn moon(p: Vec3) -> Surface {
    let terrain = fbm(p * 6.0 + v(83.0, 83.0, 83.0), 4);
    let mut color = palette(
        terrain,
        &[
            v(0.08, 0.10, 0.15),
            v(0.25, 0.30, 0.34),
            v(0.58, 0.64, 0.63),
        ],
    );
    for (center, radius) in [
        (v(0.1, 0.4, 0.9), 0.31),
        (v(-0.5, -0.25, 0.8), 0.22),
        (v(0.6, -0.25, 0.7), 0.15),
        (v(-0.4, 0.65, 0.6), 0.16),
    ] {
        let d = (p - center.normalized()).length();
        color = color * (1.0 - (1.0 - ss(radius * 0.6, radius, d)) * 0.55);
        color += v(0.16, 0.16, 0.16) * (-((d - radius) / 0.025).powi(2)).exp();
    }
    Surface {
        color,
        emission: 0.0,
    }
}

/// Radial bands and optical opacity for a separate annulus object.
pub fn ring_pattern(radius: f32, body: Body) -> (Vec3, f32) {
    let grain = noise(v(radius * 120.0, 13.0, 0.0));
    let bands = 0.5 + 0.24 * (radius * 165.0).sin() + 0.16 * (radius * 387.0).sin() + 0.1 * grain;
    let color = if body == Body::Vesper {
        palette(
            bands,
            &[
                v(0.12, 0.15, 0.15),
                v(0.38, 0.35, 0.24),
                v(0.73, 0.65, 0.43),
                v(0.93, 0.84, 0.65),
            ],
        )
    } else {
        palette(
            bands,
            &[
                v(0.14, 0.12, 0.23),
                v(0.38, 0.37, 0.56),
                v(0.68, 0.71, 0.83),
                v(0.87, 0.88, 0.95),
            ],
        )
    };
    let mut opacity =
        (0.4 + bands * 0.48) * ss(1.26, 1.31, radius) * (1.0 - ss(2.04, 2.12, radius));
    opacity *= 1.0 - 0.98 * (-((radius - 1.72) / 0.028).powi(6)).exp();
    opacity *= 1.0 - 0.78 * (-((radius - 1.97) / 0.013).powi(4)).exp();
    (color, opacity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::BODIES;
    #[test]
    fn all_shaders_remain_finite_at_poles_and_over_rotations() {
        for body in BODIES.into_iter().chain([Body::Moon]) {
            for i in 0..100 {
                let a = i as f32 * 0.17;
                let p = v(a.cos(), (a * 2.0).sin(), a.sin()).normalized();
                for t in [0.0, 1.0, 1000.0] {
                    let s = shade(body, p, t);
                    assert!(
                        s.color.x.is_finite() && s.color.y.is_finite() && s.color.z.is_finite()
                    );
                    assert!((0.0..=1.0).contains(&s.emission));
                }
            }
            for p in [v(0.0, 1.0, 0.0), v(0.0, -1.0, 0.0)] {
                assert!(shade(body, p, 0.0).color.x.is_finite());
            }
        }
    }
}
