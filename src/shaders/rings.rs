use raylib::prelude::*;
use super::*;
use crate::noise::noise3;

// ---- Parametros de los anillos (en unidades del radio del planeta) ----
pub const RING_INNER: f32 = 1.35;
pub const RING_OUTER: f32 = 2.40;
/// Por debajo de esta densidad el fragmento se descarta (hueco entre anillos).
pub const RING_THRESHOLD: f32 = 0.30;
/// Posicion radial (0..1) de la division principal, tipo Cassini.
pub const GAP_POSITION: f32 = 0.60;
/// Frecuencia de los anillos finos.
pub const RING_FREQUENCY: f32 = 70.0;

/// Densidad del anillo a una distancia `r` del centro del planeta. Depende solo
/// del radio, asi que da lo mismo cuanto gire el planeta.
pub fn density(r: f32) -> f32 {
    if r < RING_INNER || r > RING_OUTER {
        return 0.0;
    }
    let u = (r - RING_INNER) / (RING_OUTER - RING_INNER);

    let coarse = noise3(u * 7.0, 1.3, 0.7);
    let fine = noise3(u * RING_FREQUENCY, 4.1, 2.2);
    let mut d = 0.62 + 0.35 * coarse + 0.28 * fine;

    // Division principal y bordes que se desvanecen.
    d *= smoothstep(0.0, 0.035, (u - GAP_POSITION).abs());
    d *= smoothstep(0.0, 0.05, u) * (1.0 - smoothstep(0.92, 1.0, u));
    d
}

pub fn mask(position: Vector3) -> bool {
    density(position.length()) > RING_THRESHOLD
}

/// Cuanto cubren los anillos la luz que llega al punto `p` del planeta
/// (espacio de objeto del planeta: los anillos estan en el plano y = 0).
pub fn shadow_on_planet(p: Vector3, light: Vector3) -> f32 {
    if light.y.abs() < 1e-4 || p.y * light.y >= 0.0 {
        return 0.0; // el rayo hacia la luz no cruza el plano de los anillos
    }
    let s = -p.y / light.y;
    let hit = p + light * s;
    let d = density(Vector3::new(hit.x, 0.0, hit.z).length());
    smoothstep(RING_THRESHOLD, RING_THRESHOLD + 0.2, d)
}

/// Anillos: color segun densidad, iluminacion por ambas caras y la sombra
/// que el planeta proyecta sobre ellos.
pub fn ring(f: &FragmentIn) -> Vector3 {
    let pos = f.position;
    let r = pos.length();
    let d = density(r);
    let u = (r - RING_INNER) / (RING_OUTER - RING_INNER);

    let dust = rgb(0.52, 0.44, 0.36);
    let ice = rgb(0.92, 0.85, 0.72);
    let mut base = mix(dust, ice, smoothstep(0.25, 0.85, d));
    // Tinte que cambia suavemente de adentro hacia afuera.
    base = mix(base, rgb(0.75, 0.55, 0.40), 0.35 * (1.0 - u));

    // Sombra del planeta: distancia minima del rayo hacia la luz al centro.
    let l = f.light_obj;
    let b = pos.dot(l);
    let closest = (pos.dot(pos) - b * b).max(0.0).sqrt();
    let shadow = if b < 0.0 { smoothstep(1.0, 1.05, closest) } else { 1.0 };

    let face = l.y.abs(); // los anillos se iluminan por ambos lados
    base * (0.08 + 0.92 * face * shadow)
}
