use raylib::prelude::*;
use super::*;
use crate::noise::{fbm, noise_v, ridged};

// ---- Parametros que controlan la apariencia de la estrella ----
/// Velocidad a la que fluye la superficie (efecto de material animado).
pub const FLOW_SPEED: f32 = 0.08;
/// Frecuencia de las celdas de granulacion (mas alto = granos mas pequenos).
pub const GRANULE_SCALE: f32 = 5.0;
/// Frecuencia de las manchas solares (mas alto = mas manchas, mas pequenas).
pub const SPOT_SCALE: f32 = 1.7;
/// Umbral de las manchas: mas alto = menos manchas.
pub const SPOT_THRESHOLD: f32 = 0.36;
/// Fuerza de la distorsion que hace "hervir" la superficie.
pub const WARP_STRENGTH: f32 = 0.10;
/// Intensidad de las fulguraciones brillantes.
pub const FLARE_STRENGTH: f32 = 0.55;

/// Estrella: no usa la luz externa, ella misma emite. Cuatro capas de color:
/// 1) temperatura base, 2) granulacion por convección, 3) manchas solares,
/// 4) oscurecimiento del limbo + brillo del borde y fulguraciones.
pub fn star(f: &FragmentIn) -> Vector3 {
    let t = f.time * FLOW_SPEED;
    let p = f.point;

    // Distorsion de dominio lenta: desplaza el punto de muestreo con ruido,
    // asi los patrones se retuercen en vez de solo deslizarse.
    let warp = Vector3::new(
        noise_v(p * 2.0 + Vector3::new(t, 0.0, 0.0)),
        noise_v(p * 2.0 + Vector3::new(7.1, t, 0.0)),
        noise_v(p * 2.0 + Vector3::new(0.0, 3.3, t)),
    ) * WARP_STRENGTH;
    let q = p + warp;

    // Capa 1: temperatura. Ruido de baja frecuencia decide zonas frias/calientes.
    let temp = fbm(q * 3.0 + Vector3::new(0.0, 0.0, t), 4) * 0.5 + 0.5;
    let cool = rgb(0.80, 0.18, 0.02);
    let mid = rgb(1.00, 0.52, 0.06);
    let hot = rgb(1.00, 0.92, 0.55);
    let mut color = mix(cool, mid, smoothstep(0.25, 0.55, temp));
    color = mix(color, hot, smoothstep(0.55, 0.85, temp));

    // Capa 2: granulacion. Las crestas del ruido "ridged" son los bordes de las
    // celdas de convección: oscuras (gas que desciende); el centro es brillante.
    let ridge = ridged(q * GRANULE_SCALE + Vector3::new(0.0, t * 3.0, 0.0), 2);
    let lanes = smoothstep(0.80, 0.97, ridge);
    color = color * (0.80 + 0.45 * (1.0 - ridge));
    color = color * (1.0 - 0.35 * lanes);

    // Capa 3: manchas solares, solo cerca del ecuador, con umbra y penumbra.
    let band = 1.0 - smoothstep(0.55, 0.85, p.y.abs());
    let spot = fbm(p * SPOT_SCALE + Vector3::new(5.0, 2.0, t * 0.5), 3) * band;
    let penumbra = smoothstep(SPOT_THRESHOLD - 0.10, SPOT_THRESHOLD, spot);
    let umbra = smoothstep(SPOT_THRESHOLD, SPOT_THRESHOLD + 0.10, spot);
    color = mix(color, color * rgb(0.55, 0.28, 0.12), penumbra);
    color = mix(color, rgb(0.07, 0.025, 0.01), umbra);

    // Capa 4: limbo y fulguraciones.
    let facing = saturate(f.normal.dot(f.view));
    color = color * (0.50 + 0.50 * facing.sqrt());
    color = color + rgb(1.0, 0.45, 0.08) * ((1.0 - facing).powi(3) * 0.55);

    let flare = smoothstep(0.62, 0.9, noise_v(q * 14.0 + Vector3::new(0.0, 0.0, t * 6.0)));
    color = color + rgb(1.0, 0.85, 0.5) * (flare * FLARE_STRENGTH * (0.4 + 0.6 * temp));

    color * 1.1
}
