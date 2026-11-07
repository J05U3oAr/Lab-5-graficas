//! Tiny built-in bitmap font: labels are drawn into the framebuffer, not onto bodies.
use crate::scene::Body;

fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 1, 2, 4, 8, 16, 16],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '+' => [0, 4, 4, 31, 4, 4, 0],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        '[' => [14, 8, 8, 8, 8, 8, 14],
        ']' => [14, 2, 2, 2, 2, 2, 14],
        _ => [0; 7],
    }
}

pub fn text(
    frame: &mut [u32],
    width: usize,
    x: usize,
    y: usize,
    label: &str,
    scale: usize,
    color: u32,
) {
    let height = frame.len() / width;
    for (i, c) in label.chars().enumerate() {
        for (row, bits) in glyph(c).iter().enumerate() {
            for column in 0..5 {
                if bits & (1 << (4 - column)) == 0 {
                    continue;
                }
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + (i * 6 + column) * scale + dx;
                        let py = y + row * scale + dy;
                        if px < width && py < height {
                            frame[py * width + px] = color;
                        }
                    }
                }
            }
        }
    }
}

pub fn draw(
    frame: &mut [u32],
    width: usize,
    body: Body,
    system: bool,
    controls: bool,
    paused: bool,
) {
    let height = frame.len() / width;
    let margin = if width >= 900 { 40 } else { 24 };
    let small = if width >= 900 { 2 } else { 1 };
    let title = if width >= 900 { 4 } else { 3 };
    let accent = if system { 0xa2c6e2 } else { body.accent() };
    text(
        frame,
        width,
        margin,
        margin,
        "ATLAS / OBSERVATORIO PROCEDURAL",
        small,
        0x9baebe,
    );
    text(
        frame,
        width,
        margin,
        margin + small * 14,
        if system {
            "SISTEMA AURELIA"
        } else {
            body.name()
        },
        title,
        0xece9df,
    );
    let category = if system {
        "06 CUERPOS / ORBITAS / LUNA"
    } else {
        body.category()
    };
    text(
        frame,
        width,
        margin,
        margin + small * 14 + title * 10,
        category,
        small,
        accent,
    );
    let bottom = if controls { height - 96 } else { height - 52 };
    if !system {
        text(
            frame,
            width,
            margin,
            bottom,
            body.caption(),
            small,
            0xaebbc5,
        );
    }
    if controls {
        text(
            frame,
            width,
            margin,
            height - 58,
            if width < 600 {
                "1-6 CUERPO  O SISTEMA  S PNG  H HUD"
            } else {
                "1-6 CUERPO   O SISTEMA   S CAPTURA   H INTERFAZ"
            },
            small,
            0xc2ccd4,
        );
        let status = if width < 600 {
            "ESPACIO PAUSA"
        } else if paused {
            "ESPACIO REANUDAR"
        } else {
            "ESPACIO PAUSAR"
        };
        text(
            frame,
            width,
            margin,
            height - 34,
            &if width < 600 {
                format!("{status}  R RESET  +/- ZOOM  ESC SALIR")
            } else {
                format!("{status}   R REINICIAR   +/- ZOOM   ESC SALIR")
            },
            small,
            0x9baebe,
        );
    }
    for x in margin..(margin + 64).min(width) {
        let y = bottom - 12;
        frame[y * width + x] = accent;
        frame[(y + 1) * width + x] = accent;
    }
}
