use raylib::prelude::*;
use std::fs;

/// Malla triangulada. Las normales son por vertice y estan en espacio de objeto.
pub struct Mesh {
    pub vertices: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub indices: Vec<[usize; 3]>,
}

fn parse_index(token: &str, vertex_count: usize) -> usize {
    let value: i64 = token
        .split('/')
        .next()
        .and_then(|v| v.parse().ok())
        .expect("Indice de cara invalido");

    if value < 0 {
        (vertex_count as i64 + value) as usize
    } else {
        (value - 1) as usize
    }
}

impl Mesh {
    /// Carga una esfera desde un .obj. Se normaliza para que quede de radio 1
    /// y las normales salen de la posicion del vertice (es una esfera centrada).
    pub fn load_sphere(path: &str) -> Mesh {
        let content = fs::read_to_string(path).expect("No se pudo leer el archivo OBJ");

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for line in content.lines() {
            let mut parts = line.split_whitespace();

            match parts.next() {
                Some("v") => {
                    let c: Vec<f32> = parts.take(3).map(|n| n.parse().unwrap()).collect();
                    vertices.push(Vector3::new(c[0], c[1], c[2]));
                }
                Some("f") => {
                    let face: Vec<usize> = parts.map(|t| parse_index(t, vertices.len())).collect();
                    for i in 1..face.len() - 1 {
                        indices.push([face[0], face[i], face[i + 1]]);
                    }
                }
                _ => {}
            }
        }

        // Cada vertice se proyecta a la esfera unitaria: ese mismo vector sirve
        // como normal y como coordenada 3D para el ruido (sin costuras de UV).
        let normals: Vec<Vector3> = vertices.iter().map(|v| v.normalize()).collect();
        let vertices = normals.clone();

        Mesh { vertices, normals, indices }
    }

    /// Disco plano (anillo) en el plano XZ, de radio interior `inner` a exterior `outer`.
    /// Es una malla generada por codigo; la apariencia sale del fragment shader.
    pub fn ring(inner: f32, outer: f32, segments: usize) -> Mesh {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..segments {
            let a = i as f32 / segments as f32 * std::f32::consts::TAU;
            let (s, c) = a.sin_cos();
            vertices.push(Vector3::new(c * inner, 0.0, s * inner));
            vertices.push(Vector3::new(c * outer, 0.0, s * outer));
        }

        for i in 0..segments {
            let j = (i + 1) % segments;
            let (a0, a1, b0, b1) = (2 * i, 2 * i + 1, 2 * j, 2 * j + 1);
            indices.push([a0, a1, b0]);
            indices.push([a1, b1, b0]);
        }

        let normals = vec![Vector3::new(0.0, 1.0, 0.0); vertices.len()];
        Mesh { vertices, normals, indices }
    }
}
