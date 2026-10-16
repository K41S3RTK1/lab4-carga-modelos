mod framebuffer;
mod math;
mod obj;
mod render;
use framebuffer::Framebuffer;
use obj::Model;
use raylib::prelude::*;
use std::{error::Error, path::PathBuf};
fn main() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut model_path = root.join("assets/nave.obj");
    let mut export = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--model" => model_path = PathBuf::from(args.next().ok_or("Falta ruta para --model")?),
            "--render" => {
                export = Some(PathBuf::from(
                    args.next().ok_or("Falta ruta para --render")?,
                ))
            }
            "--help" | "-h" => {
                println!(
                    "cargo run --release -- [--model nave.obj] [--render salida.png]\nVentana: flechas para rotar, R para reiniciar, S para guardar, Esc para salir."
                );
                return Ok(());
            }
            _ => return Err(format!("Argumento desconocido: {arg}").into()),
        }
    }
    let model = Model::load(&model_path)?;
    println!(
        "Modelo: {} vértices, {} triángulos",
        model.vertices.len(),
        model.faces.len()
    );
    let mut fb = Framebuffer::new(1200, 900);
    render::render(&model, &mut fb, 0.0);
    if let Some(path) = export {
        fb.save(&path)?;
        println!("Imagen: {}", path.display());
        return Ok(());
    }
    let (mut rl, thread) = raylib::init()
        .size(fb.width as i32, fb.height as i32)
        .title("Lab 4 | Quetzal 01 | Rust software renderer")
        .build();
    rl.set_target_fps(60);
    let initial = Image::gen_image_color(fb.width as i32, fb.height as i32, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &initial)?;
    texture.update_texture(&fb.pixels)?;
    let mut angle = 0.0;
    while !rl.window_should_close() {
        let previous = angle;
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            angle -= rl.get_frame_time();
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            angle += rl.get_frame_time();
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            angle = 0.0;
        }
        if previous != angle {
            render::render(&model, &mut fb, angle);
            texture.update_texture(&fb.pixels)?;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_S) {
            fb.save(&root.join("entrega/modelo_renderizado.png"))?;
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        // Raylib únicamente muestra el framebuffer ya calculado por CPU.
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_text("QUETZAL / 01", 28, 24, 24, Color::new(214, 229, 240, 255));
        d.draw_text(
            "Flechas: rotar   R: reiniciar   S: PNG   Esc: salir",
            28,
            866,
            18,
            Color::new(160, 177, 195, 255),
        );
    }
    Ok(())
}
