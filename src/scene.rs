use crate::math::{v, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Aurelia,
    Nyx,
    Thalassa,
    Vesper,
    Iris,
    Borealis,
    Moon,
}

pub const BODIES: [Body; 6] = [
    Body::Aurelia,
    Body::Nyx,
    Body::Thalassa,
    Body::Vesper,
    Body::Iris,
    Body::Borealis,
];

impl Body {
    pub fn name(self) -> &'static str {
        match self {
            Self::Aurelia => "AURELIA",
            Self::Nyx => "NYX",
            Self::Thalassa => "THALASSA",
            Self::Vesper => "VESPER",
            Self::Iris => "IRIS",
            Self::Borealis => "BOREALIS",
            Self::Moon => "SELENE",
        }
    }
    pub fn category(self) -> &'static str {
        match self {
            Self::Aurelia => "ESTRELLA",
            Self::Nyx => "MUNDO VOLCANICO",
            Self::Thalassa => "MUNDO OCEANICO",
            Self::Vesper | Self::Iris => "GIGANTE GASEOSO",
            Self::Borealis => "MUNDO GLACIAL",
            Self::Moon => "LUNA",
        }
    }
    pub fn caption(self) -> &'static str {
        match self {
            Self::Aurelia => "CONVECCION / FILAMENTOS / MANCHAS SOLARES",
            Self::Nyx => "BASALTO / LAVA / CRATERES DE IMPACTO",
            Self::Thalassa => "OCEANOS / ARCHIPIELAGOS / LUNA SELENE",
            Self::Vesper => "CINTURONES DE JADE / TORMENTA / ANILLOS",
            Self::Iris => "CORRIENTES VIOLETA / VORTICES / HIELO",
            Self::Borealis => "FRACTURAS / ESCARCHA / AURORAS POLARES",
            Self::Moon => "CRATERES / MARES / REGOLITO",
        }
    }
    pub fn atmosphere(self) -> Vec3 {
        match self {
            Self::Aurelia => v(0.95, 0.18, 0.015),
            Self::Nyx => v(0.32, 0.10, 0.035),
            Self::Thalassa => v(0.03, 0.30, 0.56),
            Self::Vesper => v(0.24, 0.38, 0.30),
            Self::Iris => v(0.28, 0.12, 0.46),
            Self::Borealis => v(0.04, 0.38, 0.58),
            Self::Moon => v(0.0, 0.0, 0.0),
        }
    }
    pub fn accent(self) -> u32 {
        match self {
            Self::Aurelia => 0xffab4c,
            Self::Nyx => 0xff7e47,
            Self::Thalassa => 0x68d8c5,
            Self::Vesper => 0xe5c584,
            Self::Iris => 0xc7a9f1,
            Self::Borealis => 0x76dbf3,
            Self::Moon => 0xbfc8cc,
        }
    }
    pub fn has_rings(self) -> bool {
        matches!(self, Self::Vesper | Self::Iris)
    }
    pub fn parse(s: &str) -> Option<Self> {
        BODIES
            .into_iter()
            .find(|b| b.name().eq_ignore_ascii_case(s))
    }
}

#[derive(Clone, Debug)]
pub struct Sphere {
    pub body: Body,
    pub center: Vec3,
    pub radius: f32,
    pub angle: f32,
    pub tilt: f32,
}

#[derive(Clone, Debug)]
pub struct Ring {
    pub center: Vec3,
    pub normal: Vec3,
    pub planet_radius: f32,
    pub body: Body,
}

#[derive(Clone, Debug)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
    pub rings: Vec<Ring>,
    pub height: f32,
    pub target: Vec3,
    pub system: bool,
}

impl Scene {
    pub fn new(body: Body, time: f32, system: bool, orbit: bool) -> Self {
        let mut scene = Self {
            spheres: vec![],
            rings: vec![],
            height: 3.25,
            target: v(0.0, 0.0, 0.0),
            system,
        };
        if system {
            scene.height = 9.2;
            scene.add(Body::Aurelia, v(0.0, 0.0, 0.0), 0.66, time);
            for (i, body) in BODIES[1..].iter().enumerate() {
                let distance = 1.65 + i as f32 * 1.02;
                let phase = [2.6, 0.55, 3.9, 5.5, 1.35][i]
                    + if orbit {
                        time * 0.30 / distance.powf(1.5)
                    } else {
                        0.0
                    };
                let center = v(
                    phase.cos() * distance,
                    phase.sin() * distance * 0.48,
                    phase.sin() * distance * 0.35,
                );
                let radius = [0.23, 0.29, 0.44, 0.36, 0.26][i];
                scene.add(*body, center, radius, time);
                if *body == Body::Thalassa {
                    let a = if orbit { time * 0.6 } else { 0.0 };
                    scene.add(
                        Body::Moon,
                        center + v(a.cos() * 0.54, 0.24, a.sin() * 0.54),
                        0.075,
                        time,
                    );
                }
            }
        } else {
            scene.add(body, v(0.0, 0.0, 0.0), 1.0, time);
            if body.has_rings() {
                scene.height = 4.75;
            }
            if body == Body::Thalassa {
                scene.target.x = 0.14;
                let a = if orbit { time * 0.45 } else { 0.0 };
                scene.add(
                    Body::Moon,
                    v(a.cos() * 1.55, 0.66, a.sin() * 1.55 + 0.3),
                    0.20,
                    time,
                );
            }
        }
        scene
    }

    fn add(&mut self, body: Body, center: Vec3, radius: f32, time: f32) {
        self.spheres.push(Sphere {
            body,
            center,
            radius,
            angle: time * 0.12,
            tilt: 0.10,
        });
        if body.has_rings() {
            let normal = if body == Body::Vesper {
                v(-0.20, 0.82, 0.54)
            } else {
                v(0.30, 0.80, 0.48)
            };
            self.rings.push(Ring {
                center,
                normal: normal.normalized(),
                planet_radius: radius,
                body,
            });
        }
    }
}
