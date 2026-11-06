use raylib::prelude::*;
use super::*;
use crate::noise::{fbm, noise_v, ridged};

// ---- Parametros del planeta bioluminiscente ----
/// Frecuencia de las manchas luminosas (mas alto = mas manchas, mas pequenas).
pub const SPORE_SCALE: f32 = 7.0;
/// Umbral de las manchas: mas alto = menos manchas.
pub const SPORE_THRESHOLD: f32 = 0.60;
/// Frecuencia de las venas luminosas.
pub const VEIN_SCALE: f32 = 3.2;
/// Velocidad del pulso de las venas.
pub const PULSE_SPEED: f32 = 1.1;

/// Planeta alienigena: suelo violeta y verde azulado con venas y esporas que
/// brillan con luz propia. El brillo domina del lado nocturno y se atenua de dia,
/// asi el planeta cambia por completo entre ambos lados.
pub fn alien(f: &FragmentIn) -> Vector3 {
    let p = f.point;
    let n = f.normal_obj;
    let t = f.time;

    // Capa 1: suelo. Dos paletas mezcladas por ruido a escala continental.
    let zone = fbm(p * 2.0 + Vector3::new(3.0, 8.0, 2.0), 4) * 0.5 + 0.5;
    let soil = mix(rgb(0.22, 0.08, 0.32), rgb(0.07, 0.30, 0.30), smoothstep(0.35, 0.65, zone));

    // Capa 2: rocas y relieve fino: oscurece y aclara el suelo.
    let rough = fbm(p * 9.0, 4);
    let ground = soil * (0.75 + 0.5 * (rough * 0.5 + 0.5));

    // Capa 3: venas luminosas (crestas ridged deformadas) que laten.
    let warp = noise_v(p * 2.5 + Vector3::new(0.0, t * 0.05, 4.0)) * 0.3;
    let vein = smoothstep(0.84, 0.96, ridged(p * VEIN_SCALE + Vector3::new(warp, 0.0, warp), 3));
    let pulse = 0.65 + 0.35 * (t * PULSE_SPEED + noise_v(p * 4.0) * 5.0).sin();

    // Capa 4: esporas, manchas aisladas que aparecen y desaparecen.
    let spore_n = noise_v(p * SPORE_SCALE + Vector3::new(0.0, 0.0, t * 0.12)) * 0.5 + 0.5;
    let spore = smoothstep(SPORE_THRESHOLD, SPORE_THRESHOLD + 0.1, spore_n);

    let glow_color = mix(rgb(0.05, 0.95, 0.55), rgb(0.95, 0.15, 0.95), smoothstep(0.3, 0.7, zone));
    let glow = saturate(vein * pulse + spore * 0.9);

    // Iluminacion: de dia el brillo se ve tenue; de noche manda.
    let ndl = n.dot(f.light_obj);
    let day = smoothstep(-0.15, 0.3, ndl);
    let diffuse = 0.05 + 0.95 * saturate((ndl + 0.1) / 1.1);
    let mut color = ground * diffuse * (1.0 - 0.75 * glow);
    color = color + glow_color * glow * (1.4 - 0.9 * day);

    // Atmosfera verdosa en el borde.
    let rim = (1.0 - saturate(n.dot(f.rotation.transpose().apply(f.view)))).powi(3);
    color + rgb(0.3, 1.0, 0.7) * (rim * (0.15 + 0.35 * day))
}
