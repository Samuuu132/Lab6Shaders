use raylib::prelude::*;
use super::*;

/// Shader de prueba: gris con iluminacion difusa. Sirve para verificar la
/// geometria, la camara y la rotacion antes de escribir los shaders reales.
pub fn basic(f: &FragmentIn) -> Vector3 {
    let diffuse = saturate(f.normal.dot(f.light));
    rgb(0.75, 0.75, 0.8) * (0.12 + 0.88 * diffuse)
}
