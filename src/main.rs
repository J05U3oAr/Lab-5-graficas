mod hud;
mod math;
mod noise;
mod renderer;
mod scene;
mod shaders;

use image::{Rgb, RgbImage};
use minifb::{Key, KeyRepeat, ScaleMode, Window, WindowOptions};
use renderer::Renderer;
use scene::{Body, Scene, BODIES};
use std::{
    error::Error,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

struct Options {
    body: Body,
    width: usize,
    height: usize,
    samples: usize,
    time: f32,
    render: bool,
    all: bool,
    system: bool,
    paused: bool,
    hud: bool,
    output: Option<PathBuf>,
    frames: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            body: Body::Vesper,
            width: 960,
            height: 720,
            samples: 1,
            time: 0.0,
            render: false,
            all: false,
            system: false,
            paused: false,
            hud: true,
            output: None,
            frames: None,
        }
    }
}

fn usage() {
    println!("ATLAS — Laboratorio de shaders en Rust\n\n\
        cargo run --release\n\
        cargo run --release -- --render --body vesper --output captures/vesper.png\n\
        cargo run --release -- --all --output docs/screenshots --width 1200 --height 900 --samples 2\n\n\
        --body NOMBRE    aurelia, nyx, thalassa, vesper, iris, borealis\n\
        --render         Guardar PNG sin abrir ventana\n\
        --all            Exportar seis cuerpos, sistema, catálogo y concurso\n\
        --system         Vista del sistema con órbitas\n\
        --width N        Ancho [320..3840], predeterminado 960\n\
        --height N       Alto [240..2160], predeterminado 720\n\
        --samples N      Antialiasing: 1 o 2 (2 = cuatro rayos por píxel)\n\
        --time SEGUNDOS  Instante inicial reproducible\n\
        --paused         Iniciar con animación detenida\n\
        --no-hud         Ocultar títulos y controles\n\
        --output RUTA    Archivo PNG, o directorio cuando se usa --all\n\
        --frames N       Cerrar ventana tras N cuadros (verificación)\n\
        --help           Mostrar esta ayuda\n\n\
        Teclas: 1-6, flechas, O sistema, Espacio pausa, S captura, H interfaz,\n\
        R reiniciar, +/- zoom, Esc salir.");
}

fn parse(args: impl Iterator<Item = String>) -> Result<Option<Options>> {
    let mut options = Options::default();
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                usage();
                return Ok(None);
            }
            "--render" => options.render = true,
            "--all" => options.all = true,
            "--system" => options.system = true,
            "--paused" => options.paused = true,
            "--no-hud" => options.hud = false,
            "--body" => {
                let value = args.next().ok_or("Falta nombre después de --body")?;
                options.body =
                    Body::parse(&value).ok_or_else(|| format!("Cuerpo desconocido: {value}"))?;
            }
            "--width" => options.width = args.next().ok_or("Falta ancho")?.parse()?,
            "--height" => options.height = args.next().ok_or("Falta alto")?.parse()?,
            "--samples" => {
                options.samples = args.next().ok_or("Falta cantidad de muestras")?.parse()?
            }
            "--time" => options.time = args.next().ok_or("Falta tiempo")?.parse()?,
            "--output" => options.output = Some(args.next().ok_or("Falta ruta de salida")?.into()),
            "--frames" => {
                options.frames = Some(args.next().ok_or("Falta número de cuadros")?.parse()?)
            }
            _ => return Err(format!("Opción desconocida: {arg}. Usa --help.").into()),
        }
    }
    if !(320..=3840).contains(&options.width) || !(240..=2160).contains(&options.height) {
        return Err("Resolución fuera de rango; usa --help.".into());
    }
    if !(1..=2).contains(&options.samples) {
        return Err("--samples debe ser 1 o 2".into());
    }
    if !options.time.is_finite() {
        return Err("--time debe ser finito".into());
    }
    if options.frames == Some(0) {
        return Err("--frames debe ser mayor a cero".into());
    }
    Ok(Some(options))
}

fn frame_image(frame: &[u32], width: usize, height: usize) -> RgbImage {
    RgbImage::from_fn(width as u32, height as u32, |x, y| {
        let c = frame[y as usize * width + x as usize];
        Rgb([(c >> 16) as u8, (c >> 8) as u8, c as u8])
    })
}

fn save(frame: &[u32], renderer: Renderer, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    frame_image(frame, renderer.width, renderer.height).save(path)?;
    println!("Guardado: {}", path.display());
    Ok(())
}

fn export(options: &Options) -> Result<()> {
    let renderer = Renderer::new(options.width, options.height, options.samples);
    let started = Instant::now();
    if options.all {
        let directory = options
            .output
            .clone()
            .unwrap_or_else(|| "docs/screenshots".into());
        let mut catalog = RgbImage::new(1920, 960);
        for (i, body) in BODIES.into_iter().enumerate() {
            let scene = Scene::new(body, options.time, false, false);
            let mut frame = renderer.render(&scene, options.time);
            if options.hud {
                hud::draw(&mut frame, renderer.width, body, false, false, true);
            }
            save(
                &frame,
                renderer,
                &directory.join(format!("{}.png", body.name().to_lowercase())),
            )?;
            let thumb = Renderer::new(640, 480, options.samples);
            let mut frame = thumb.render(&scene, options.time);
            if options.hud {
                hud::draw(&mut frame, 640, body, false, false, true);
            }
            image::imageops::replace(
                &mut catalog,
                &frame_image(&frame, 640, 480),
                (i % 3 * 640) as i64,
                (i / 3 * 480) as i64,
            );
        }
        let scene = Scene::new(Body::Vesper, options.time, true, false);
        let mut frame = renderer.render(&scene, options.time);
        if options.hud {
            hud::draw(&mut frame, renderer.width, Body::Vesper, true, false, true);
        }
        save(&frame, renderer, &directory.join("sistema.png"))?;
        catalog.save(directory.join("catalogo.png"))?;
        let scene = Scene::new(Body::Vesper, options.time, false, false);
        save(
            &renderer.render(&scene, options.time),
            renderer,
            &directory.join("concurso.png"),
        )?;
    } else {
        let scene = Scene::new(options.body, options.time, options.system, !options.paused);
        let mut frame = renderer.render(&scene, options.time);
        if options.hud {
            hud::draw(
                &mut frame,
                renderer.width,
                options.body,
                options.system,
                false,
                true,
            );
        }
        let path = options.output.clone().unwrap_or_else(|| {
            PathBuf::from(format!(
                "captures/{}.png",
                if options.system {
                    "sistema".into()
                } else {
                    options.body.name().to_lowercase()
                }
            ))
        });
        save(&frame, renderer, &path)?;
    }
    println!("Render en CPU: {:.2} s", started.elapsed().as_secs_f32());
    Ok(())
}

fn interactive(options: &Options) -> Result<()> {
    let renderer = Renderer::new(options.width, options.height, options.samples);
    let mut window = Window::new(
        "ATLAS | Rust CPU renderer",
        options.width,
        options.height,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);
    let mut body = options.body;
    let mut system = options.system;
    let mut paused = options.paused;
    let mut show_hud = options.hud;
    let mut time = options.time;
    let mut zoom = 1.0_f32;
    let mut previous = Instant::now();
    let mut count = 0;
    let mut dirty = true;
    let mut frame = Vec::new();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let dt = (now - previous).as_secs_f32().min(0.25);
        previous = now;
        if !paused {
            time += dt;
            dirty = true;
        }
        for key in window.get_keys_pressed(KeyRepeat::No) {
            dirty = true;
            match key {
                Key::Key1 => {
                    body = Body::Aurelia;
                    system = false;
                }
                Key::Key2 => {
                    body = Body::Nyx;
                    system = false;
                }
                Key::Key3 => {
                    body = Body::Thalassa;
                    system = false;
                }
                Key::Key4 => {
                    body = Body::Vesper;
                    system = false;
                }
                Key::Key5 => {
                    body = Body::Iris;
                    system = false;
                }
                Key::Key6 => {
                    body = Body::Borealis;
                    system = false;
                }
                Key::Left | Key::Right => {
                    let index = BODIES.iter().position(|b| *b == body).unwrap_or(0);
                    body = BODIES[(index + if key == Key::Right { 1 } else { 5 }) % 6];
                    system = false;
                }
                Key::O => system = !system,
                Key::Space => paused = !paused,
                Key::H => show_hud = !show_hud,
                Key::R => {
                    time = options.time;
                    zoom = 1.0;
                }
                Key::Equal | Key::NumPadPlus => zoom = (zoom * 0.9).max(0.45),
                Key::Minus | Key::NumPadMinus => zoom = (zoom / 0.9).min(2.5),
                Key::S => {
                    let mut scene = Scene::new(body, time, system, true);
                    scene.height *= zoom;
                    let mut capture = renderer.render(&scene, time);
                    if show_hud {
                        hud::draw(&mut capture, renderer.width, body, system, false, paused);
                    }
                    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
                    let name = if system {
                        "sistema".into()
                    } else {
                        body.name().to_lowercase()
                    };
                    save(
                        &capture,
                        renderer,
                        &PathBuf::from(format!("captures/{name}-{stamp}.png")),
                    )?;
                }
                _ => {}
            }
        }
        if dirty {
            let start = Instant::now();
            let mut scene = Scene::new(body, time, system, true);
            scene.height *= zoom;
            frame = renderer.render(&scene, time);
            if show_hud {
                hud::draw(&mut frame, renderer.width, body, system, true, paused);
            }
            let fps = 1.0 / start.elapsed().as_secs_f32();
            window.set_title(&format!(
                "ATLAS | {} | {:.1} FPS | {} | 1-6 cambiar, O sistema, S captura",
                if system { "SISTEMA" } else { body.name() },
                fps,
                if paused { "PAUSA" } else { "ROTACION" }
            ));
            dirty = false;
        }
        window.update_with_buffer(&frame, renderer.width, renderer.height)?;
        count += 1;
        if options.frames.is_some_and(|limit| count >= limit) {
            break;
        }
    }
    Ok(())
}

fn run() -> Result<()> {
    let Some(options) = parse(std::env::args().skip(1))? else {
        return Ok(());
    };
    if options.render || options.all {
        export(&options)
    } else {
        interactive(&options)
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cli_rejects_invalid_arguments_without_opening_a_window() {
        for args in [
            vec!["--samples", "0"],
            vec!["--width", "0"],
            vec!["--time", "NaN"],
            vec!["--body", "pluton"],
            vec!["--output"],
            vec!["--frames", "0"],
            vec!["--unknown"],
        ] {
            assert!(parse(args.into_iter().map(String::from)).is_err());
        }
    }
}
