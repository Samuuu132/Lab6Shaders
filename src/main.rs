mod fragment;
mod framebuffer;
mod obj;
mod rasterizer;
mod renderer;
mod shaders;
mod transform;

use raylib::prelude::*;
use framebuffer::Framebuffer;
use obj::Mesh;
use renderer::{render, Object, Scene};
use transform::{Camera, Mat3};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

/// Cuerpos celestes disponibles. Cada uno se selecciona con una tecla.
#[derive(Clone, Copy, PartialEq)]
enum Body {
    Basic,
}

const BODIES: [(Body, &str); 1] = [(Body::Basic, "Esfera de prueba")];

fn build_scene<'a>(body: Body, sphere: &'a Mesh, time: f32) -> Scene<'a> {
    let spin = Mat3::rot_y(time * 0.3);
    let light = Vector3::new(-0.6, 0.35, 0.7).normalize();

    let objects = match body {
        Body::Basic => vec![Object {
            mesh: sphere,
            position: Vector3::ZERO,
            scale: 1.0,
            rotation: spin,
            shader: shaders::basic::basic,
        }],
    };

    Scene { objects, light, time }
}

fn main() {
    let sphere = Mesh::load_sphere("models/sphere.obj");
    println!("Esfera: {} vertices, {} triangulos", sphere.vertices.len(), sphere.indices.len());

    let camera = Camera::new(Vector3::new(0.0, 0.0, 3.2), 45.0, WIDTH as f32, HEIGHT as f32);
    let mut fb = Framebuffer::new(WIDTH, HEIGHT, Color::new(4, 4, 10, 255));

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--save-images") {
        std::fs::create_dir_all("images").expect("No se pudo crear la carpeta images");
        for (i, (body, name)) in BODIES.iter().enumerate() {
            render(&mut fb, &build_scene(*body, &sphere, 1.0), &camera);
            let path = format!("images/{}_{}.png", i + 1, name.to_lowercase().replace(' ', "_"));
            fb.save_png(&path);
            println!("Guardado {path}");
        }
        return;
    }

    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Lab 6 - Shaders procedurales")
        .build();
    rl.set_target_fps(60);

    let blank = Image::gen_image_color(WIDTH as i32, HEIGHT as i32, Color::BLACK);
    let mut texture = rl
        .load_texture_from_image(&thread, &blank)
        .expect("No se pudo crear la textura");

    let keys = [
        KeyboardKey::KEY_ONE,
        KeyboardKey::KEY_TWO,
        KeyboardKey::KEY_THREE,
        KeyboardKey::KEY_FOUR,
        KeyboardKey::KEY_FIVE,
        KeyboardKey::KEY_SIX,
        KeyboardKey::KEY_SEVEN,
    ];
    let mut current = 0;

    while !rl.window_should_close() {
        for (i, key) in keys.iter().enumerate().take(BODIES.len()) {
            if rl.is_key_pressed(*key) {
                current = i;
            }
        }

        let time = rl.get_time() as f32;
        render(&mut fb, &build_scene(BODIES[current].0, &sphere, time), &camera);
        texture.update_texture(&fb.pixels).expect("No se pudo actualizar la textura");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_text(&format!("[{}] {}", current + 1, BODIES[current].1), 10, 10, 20, Color::WHITE);
        d.draw_fps(WIDTH as i32 - 90, 10);
    }
}
