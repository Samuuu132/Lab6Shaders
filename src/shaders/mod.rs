use raylib::prelude::*;
use crate::transform::Mat3;

pub mod star;

/// Todo lo que un fragment shader puede consultar sobre su pixel.
pub struct FragmentIn {
    /// Punto sobre la esfera unitaria en espacio de objeto (gira con el cuerpo).
    pub point: Vector3,
    /// Normal en espacio de mundo.
    pub normal: Vector3,
    /// Normal en espacio de objeto.
    pub normal_obj: Vector3,
    /// Direccion hacia la luz en espacio de mundo y de objeto.
    pub light: Vector3,
    pub light_obj: Vector3,
    /// Direccion hacia la camara en espacio de mundo.
    pub view: Vector3,
    pub time: f32,
    /// Rotacion objeto -> mundo (por si el shader necesita pasar vectores).
    pub rotation: Mat3,
}

/// Un fragment shader recibe el fragmento y devuelve un color RGB en 0..1.
pub type ShaderFn = fn(&FragmentIn) -> Vector3;

// ---- Utilidades matematicas compartidas por los shaders ----

pub fn rgb(r: f32, g: f32, b: f32) -> Vector3 {
    Vector3::new(r, g, b)
}

pub fn mix(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    a * (1.0 - t) + b * t
}

pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
