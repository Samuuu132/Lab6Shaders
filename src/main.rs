mod fragment;
mod noise;
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
    Star,
    Rocky,
    Gas,
}

const BODIES: [(Body, &str); 3] = [
    (Body::Star, "Estrella"),
    (Body::Rocky, "Planeta rocoso"),
    (Body::Gas, "Gigante gaseoso"),
];

/// Mallas compartidas. Todos los cuerpos usan la esfera; solo los anillos
/// necesitan un disco generado por codigo.
struct Meshes {
    sphere: Mesh,
    ring: Mesh,
}

fn build_scene<'a>(body: Body, meshes: &'a Meshes, time: f32) -> Scene<'a> {
    let sphere = &meshes.sphere;
    let spin = Mat3::rot_y(time * 0.3);
    let light = Vector3::new(-0.6, 0.35, 0.7).normalize();

    let objects = match body {
        Body::Star => vec![Object {
            mesh: sphere,
            position: Vector3::ZERO,
            scale: 1.0,
            rotation: spin,
            shader: shaders::star::star,
            mask: None,
        }],
        Body::Rocky => {
            // Eje inclinado + rotacion lenta. La luna orbita y siempre muestra la misma cara.
            let tilt = Mat3::rot_z(0.35);
            let planet_rot = tilt.mul(&Mat3::rot_y(time * 0.12));
            let a = time * 0.3;
            let moon_pos = Vector3::new(a.cos() * 1.65, 0.28 * a.sin(), -a.sin() * 1.65 - 0.2);
            vec![
                Object {
                    mesh: sphere,
                    position: Vector3::ZERO,
                    scale: 1.0,
                    rotation: planet_rot,
                    shader: shaders::rocky::rocky,
                    mask: None,
                },
                Object {
                    mesh: sphere,
                    position: moon_pos,
                    scale: 0.26,
                    rotation: Mat3::rot_y(a),
                    shader: shaders::moon::moon,
                    mask: None,
                },
            ]
        }
        Body::Gas => {
            // Planeta y anillos comparten la inclinacion; solo el planeta gira.
            let tilt = Mat3::rot_z(-0.25).mul(&Mat3::rot_x(0.30));
            vec![
                Object {
                    mesh: sphere,
                    position: Vector3::ZERO,
                    scale: 0.58,
                    rotation: tilt.mul(&Mat3::rot_y(time * 0.10)),
                    shader: shaders::gas::gas_giant,
                    mask: None,
                },
                Object {
                    mesh: &meshes.ring,
                    position: Vector3::ZERO,
                    scale: 0.58,
                    rotation: tilt,
                    shader: shaders::rings::ring,
                    mask: Some(shaders::rings::mask),
                },
            ]
        }
    };

    Scene { objects, light, time }
}

fn main() {
    let meshes = Meshes {
        sphere: Mesh::load_sphere("models/sphere.obj"),
        ring: Mesh::ring(shaders::rings::RING_INNER, shaders::rings::RING_OUTER, 256),
    };
    println!(
        "Esfera: {} vertices, {} triangulos",
        meshes.sphere.vertices.len(),
        meshes.sphere.indices.len()
    );

    let camera = Camera::new(Vector3::new(0.0, 0.0, 3.2), 45.0, WIDTH as f32, HEIGHT as f32);
    let mut fb = Framebuffer::new(WIDTH, HEIGHT, Color::new(4, 4, 10, 255));

    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--save-images") {
        std::fs::create_dir_all("images").expect("No se pudo crear la carpeta images");
        for (i, (body, name)) in BODIES.iter().enumerate() {
            render(&mut fb, &build_scene(*body, &meshes, 1.0), &camera);
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
        render(&mut fb, &build_scene(BODIES[current].0, &meshes, time), &camera);
        texture.update_texture(&fb.pixels).expect("No se pudo actualizar la textura");

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        d.draw_texture(&texture, 0, 0, Color::WHITE);
        d.draw_text(&format!("[{}] {}", current + 1, BODIES[current].1), 10, 10, 20, Color::WHITE);
        d.draw_fps(WIDTH as i32 - 90, 10);
    }
}
