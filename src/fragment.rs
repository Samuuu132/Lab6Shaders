use raylib::prelude::*;

/// Resultado de rasterizar un triangulo en un pixel. Lleva los atributos
/// interpolados con correccion de perspectiva para el fragment shader.
#[derive(Clone, Copy)]
pub struct Fragment {
    pub x: i32,
    pub y: i32,
    pub depth: f32,
    pub position: Vector3, // espacio de objeto
    pub normal: Vector3,   // espacio de objeto
}
