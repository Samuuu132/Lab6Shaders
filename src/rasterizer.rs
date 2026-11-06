use raylib::prelude::*;
use crate::fragment::Fragment;

/// Triangulo ya proyectado a pantalla. Cada vertice conserva su profundidad
/// (para interpolar con correccion de perspectiva) y sus atributos.
pub struct ScreenTriangle {
    pub screen: [(f32, f32); 3],
    pub depth: [f32; 3],
    pub position: [Vector3; 3],
    pub normal: [Vector3; 3],
}

fn edge(a: (f32, f32), b: (f32, f32), p: (f32, f32)) -> f32 {
    (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0)
}

/// Rasterizacion con coordenadas baricentricas (el mismo metodo del Lab 5),
/// ahora interpolando posicion y normal para el fragment shader.
pub fn rasterize(triangles: &[ScreenTriangle], width: usize, height: usize) -> Vec<Fragment> {
    let mut fragments = Vec::new();

    for t in triangles {
        let [a, b, c] = t.screen;
        let area = edge(a, b, c);
        if area == 0.0 {
            continue;
        }

        let min_x = a.0.min(b.0).min(c.0).floor().max(0.0) as i32;
        let max_x = a.0.max(b.0).max(c.0).ceil().min(width as f32 - 1.0) as i32;
        let min_y = a.1.min(b.1).min(c.1).floor().max(0.0) as i32;
        let max_y = a.1.max(b.1).max(c.1).ceil().min(height as f32 - 1.0) as i32;

        let inv_d = [1.0 / t.depth[0], 1.0 / t.depth[1], 1.0 / t.depth[2]];

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let p = (x as f32 + 0.5, y as f32 + 0.5);

                let w0 = edge(b, c, p) / area;
                let w1 = edge(c, a, p) / area;
                let w2 = edge(a, b, p) / area;

                if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                    continue;
                }

                // Correccion de perspectiva: se interpola attr/depth y 1/depth.
                let p0 = w0 * inv_d[0];
                let p1 = w1 * inv_d[1];
                let p2 = w2 * inv_d[2];
                let inv = p0 + p1 + p2;
                let depth = 1.0 / inv;

                let position = (t.position[0] * p0 + t.position[1] * p1 + t.position[2] * p2) * depth;
                let normal = (t.normal[0] * p0 + t.normal[1] * p1 + t.normal[2] * p2) * depth;

                fragments.push(Fragment { x, y, depth, position, normal });
            }
        }
    }

    fragments
}
