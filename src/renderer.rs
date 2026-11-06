use raylib::prelude::*;
use crate::fragment::Fragment;
use crate::framebuffer::{to_rgba, Framebuffer};
use crate::obj::Mesh;
use crate::rasterizer::{rasterize, ScreenTriangle};
use crate::shaders::{FragmentIn, ShaderFn};
use crate::transform::{Camera, Mat3};

/// Un objeto de la escena: malla compartida + transformacion + fragment shader.
pub struct Object<'a> {
    pub mesh: &'a Mesh,
    pub position: Vector3,
    pub scale: f32,
    pub rotation: Mat3,
    pub shader: ShaderFn,
    /// Prueba de descarte (como `discard` en GLSL): si devuelve false el
    /// fragmento no existe, ni siquiera tapa lo que esta detras. Recibe la
    /// posicion en espacio de objeto.
    pub mask: Option<fn(Vector3) -> bool>,
}

pub struct Scene<'a> {
    pub objects: Vec<Object<'a>>,
    pub light: Vector3, // direccion hacia la luz (mundo)
    pub time: f32,
}

fn vertex_stage(obj: &Object, camera: &Camera) -> Vec<ScreenTriangle> {
    let mesh = obj.mesh;

    let projected: Vec<(f32, f32, f32)> = mesh
        .vertices
        .iter()
        .map(|v| camera.project(obj.rotation.apply(*v) * obj.scale + obj.position))
        .collect();

    mesh.indices
        .iter()
        .filter(|[i0, i1, i2]| projected[*i0].2 > 0.01 && projected[*i1].2 > 0.01 && projected[*i2].2 > 0.01)
        .map(|[i0, i1, i2]| {
            let idx = [*i0, *i1, *i2];
            ScreenTriangle {
                screen: idx.map(|i| (projected[i].0, projected[i].1)),
                depth: idx.map(|i| projected[i].2),
                position: idx.map(|i| mesh.vertices[i]),
                normal: idx.map(|i| mesh.normals[i]),
            }
        })
        .collect()
}

/// Pipeline: vertices -> rasterizacion -> prueba de profundidad -> fragment shader.
/// Se sombrea solo el fragmento ganador de cada pixel.
pub fn render(fb: &mut Framebuffer, scene: &Scene, camera: &Camera) {
    fb.clear();

    let mut gbuffer: Vec<Option<(Fragment, usize)>> = vec![None; fb.width * fb.height];

    for (id, obj) in scene.objects.iter().enumerate() {
        let triangles = vertex_stage(obj, camera);
        for frag in rasterize(&triangles, fb.width, fb.height) {
            if obj.mask.is_some_and(|m| !m(frag.position)) {
                continue;
            }
            let slot = &mut gbuffer[frag.y as usize * fb.width + frag.x as usize];
            if slot.map_or(true, |(old, _)| frag.depth < old.depth) {
                *slot = Some((frag, id));
            }
        }
    }

    // El sombreado es independiente por pixel: se reparte por bandas de filas
    // entre hilos.
    let (w, h) = (fb.width, fb.height);
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let rows_per = h.div_ceil(threads);
    let gbuffer = &gbuffer;

    std::thread::scope(|s| {
        for (band, chunk) in fb.pixels.chunks_mut(rows_per * w * 4).enumerate() {
            s.spawn(move || {
                for (r, row) in chunk.chunks_mut(w * 4).enumerate() {
                    let y = band * rows_per + r;
                    for x in 0..w {
                        if let Some((frag, id)) = gbuffer[y * w + x] {
                            let color = shade(&scene.objects[id], &frag, scene, camera);
                            row[x * 4..x * 4 + 4].copy_from_slice(&to_rgba(color));
                        }
                    }
                }
            });
        }
    });
}

fn shade(obj: &Object, frag: &Fragment, scene: &Scene, camera: &Camera) -> Vector3 {
    let normal_obj = frag.normal.normalize();
    let world_pos = obj.rotation.apply(frag.position) * obj.scale + obj.position;

    let input = FragmentIn {
        point: frag.position.normalize(),
        position: frag.position,
        normal: obj.rotation.apply(normal_obj),
        normal_obj,
        light: scene.light,
        light_obj: obj.rotation.transpose().apply(scene.light),
        view: (camera.position - world_pos).normalize(),
        time: scene.time,
        rotation: obj.rotation,
    };

    (obj.shader)(&input)
}
