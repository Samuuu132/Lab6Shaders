use raylib::prelude::*;
use crate::fragment::Fragment;
use crate::framebuffer::Framebuffer;
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
            let slot = &mut gbuffer[frag.y as usize * fb.width + frag.x as usize];
            if slot.map_or(true, |(old, _)| frag.depth < old.depth) {
                *slot = Some((frag, id));
            }
        }
    }

    for y in 0..fb.height {
        for x in 0..fb.width {
            if let Some((frag, id)) = gbuffer[y * fb.width + x] {
                let obj = &scene.objects[id];
                let normal_obj = frag.normal.normalize();
                let world_pos = obj.rotation.apply(frag.position) * obj.scale + obj.position;
                let inv = obj.rotation.transpose();

                let input = FragmentIn {
                    point: frag.position.normalize(),
                    normal: obj.rotation.apply(normal_obj),
                    normal_obj,
                    light: scene.light,
                    light_obj: inv.apply(scene.light),
                    view: (camera.position - world_pos).normalize(),
                    time: scene.time,
                    rotation: obj.rotation,
                };

                fb.set_pixel(x, y, (obj.shader)(&input));
            }
        }
    }
}
