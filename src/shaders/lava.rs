use raylib::prelude::*;
use super::*;
use crate::noise::{fbm, noise_v, ridged};

// ---- Parametros del planeta de lava ----
/// Frecuencia de las grietas de lava (mas alto = red de grietas mas densa).
pub const CRACK_SCALE: f32 = 2.6;
/// Grosor de las grietas: mas bajo = grietas mas anchas.
pub const CRACK_THRESHOLD: f32 = 0.80;
/// Nivel de los mares de lava en el campo de altura (mas alto = mas lava).
pub const LAVA_SEA: f32 = -0.22;
/// Velocidad del pulso de calor.
pub const PULSE_SPEED: f32 = 1.4;

/// Planeta de lava: corteza oscura de basalto sobre lava emisiva. La lava emite
/// su propio color (no depende de la luz); la corteza si se ilumina.
pub fn lava(f: &FragmentIn) -> Vector3 {
    let p = f.point;
    let n = f.normal_obj;
    let t = f.time;

    // Capa 1: corteza. Dos tonos de basalto repartidos por ruido.
    let h = fbm(p * 3.0 + Vector3::new(1.0, 2.0, 3.0), 5);
    let crust = mix(rgb(0.07, 0.055, 0.055), rgb(0.25, 0.19, 0.16), smoothstep(-0.3, 0.4, h));

    // Capa 2: grietas. Las crestas del ruido ridged, deformadas por otro ruido,
    // forman una red de fisuras; mares de lava donde la altura es baja.
    let warp = Vector3::new(
        noise_v(p * 2.0 + Vector3::new(0.0, t * 0.03, 0.0)),
        noise_v(p * 2.0 + Vector3::new(5.0, 0.0, t * 0.03)),
        noise_v(p * 2.0 + Vector3::new(2.0, 7.0, 0.0)),
    ) * 0.25;
    let ridge = ridged(p * CRACK_SCALE + warp, 3);
    let crack = smoothstep(CRACK_THRESHOLD, 0.95, ridge);
    let sea = smoothstep(LAVA_SEA, LAVA_SEA - 0.12, h);
    let heat = saturate(crack + sea);

    // Capa 3: color de la lava segun la temperatura; el pulso la hace respirar.
    let pulse = 0.78 + 0.22 * (t * PULSE_SPEED + noise_v(p * 3.0) * 6.0).sin();
    let energy = saturate(heat * pulse);
    let lava_color = mix(rgb(0.55, 0.03, 0.0), rgb(1.0, 0.42, 0.04), smoothstep(0.2, 0.7, energy));
    let lava_color = mix(lava_color, rgb(1.0, 0.92, 0.55), smoothstep(0.75, 1.0, energy));

    // Corteza iluminada con terminador suave, mas el resplandor de la lava cercana.
    let ndl = saturate(n.dot(f.light_obj));
    let lit = 0.05 + 0.95 * ndl;
    let glow = rgb(0.35, 0.08, 0.0) * smoothstep(0.35, 0.75, ridge) * 0.6;
    let mut color = (crust + glow) * lit;

    // Capa 4: lava emisiva y halo de calor en el borde.
    color = mix(color, lava_color * (0.9 + 0.5 * heat), smoothstep(0.05, 0.5, heat));
    let rim = (1.0 - saturate(n.dot(f.rotation.transpose().apply(f.view)))).powi(3);
    color + rgb(1.0, 0.3, 0.05) * (rim * 0.45)
}
