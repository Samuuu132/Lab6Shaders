use raylib::prelude::*;
use super::*;
use crate::noise::{fbm, noise3, noise_v};

// ---- Parametros del planeta de hielo ----
/// Frecuencia de las fracturas (mas alto = mas grietas, mas cortas).
pub const FRACTURE_SCALE: f32 = 3.6;
/// Umbral de las fracturas: mas bajo = grietas mas anchas.
pub const FRACTURE_THRESHOLD: f32 = 0.93;
/// Intensidad del brillo especular del hielo liso.
pub const GLOSS: f32 = 0.8;
/// Cantidad de destellos de cristales.
pub const SPARKLE: f32 = 0.9;

/// Planeta de hielo: placas de hielo grueso y delgado, fracturas azules,
/// escarcha, brillo especular y luz que se filtra (subsuelo) hacia el terminador.
pub fn ice(f: &FragmentIn) -> Vector3 {
    let p = f.point;
    let n = f.normal_obj;
    let light = f.light_obj;
    let view = f.rotation.transpose().apply(f.view);

    // Capa 1: grosor del hielo. Hielo grueso es blanco; delgado deja ver azul.
    let thick = fbm(p * 2.2 + Vector3::new(7.0, 1.0, 3.0), 5) * 0.5 + 0.5;
    let mut color = mix(rgb(0.45, 0.72, 0.92), rgb(0.90, 0.97, 1.0), smoothstep(0.3, 0.7, thick));

    // Capa 2: fracturas. Ridged deformado = lineas finas que se ramifican.
    let warp = Vector3::new(
        noise_v(p * 3.0 + Vector3::new(1.0, 0.0, 0.0)),
        noise_v(p * 3.0 + Vector3::new(0.0, 4.0, 0.0)),
        noise_v(p * 3.0 + Vector3::new(0.0, 0.0, 9.0)),
    ) * 0.18;
    // Se usan dos capas de una sola octava (lineas largas, no puntos sueltos).
    let line_a = 1.0 - noise_v(p * FRACTURE_SCALE + warp).abs();
    let line_b = 1.0 - noise_v(p * FRACTURE_SCALE * 2.3 + warp * 2.0 + Vector3::new(5.0, 1.0, 2.0)).abs();
    let crack = smoothstep(FRACTURE_THRESHOLD, 0.985, line_a)
        .max(0.7 * smoothstep(FRACTURE_THRESHOLD + 0.04, 0.99, line_b));
    color = mix(color, rgb(0.05, 0.20, 0.45), crack * 0.85);

    // Capa 3: escarcha. Detalle fino que aclara y rompe la uniformidad.
    let frost = fbm(p * 14.0, 3) * 0.5 + 0.5;
    color = mix(color, rgb(1.0, 1.0, 1.0), smoothstep(0.6, 0.9, frost) * 0.2 * (1.0 - crack));

    // Casquetes polares mas blancos.
    color = mix(color, rgb(0.97, 0.99, 1.0), smoothstep(0.7, 0.95, p.y.abs()) * 0.7);

    // Capa 4: iluminacion. Difusa suave + luz filtrada azul hacia la sombra.
    let ndl = n.dot(light);
    let diffuse = saturate((ndl + 0.2) / 1.2);
    let subsurface = rgb(0.10, 0.35, 0.65) * smoothstep(0.1, -0.25, ndl) * smoothstep(-0.8, -0.1, ndl) * 0.5;
    color = color * (0.05 + 0.95 * diffuse) + subsurface * (1.0 - crack);

    // Brillo especular: el hielo liso (grueso, sin grietas) brilla mas.
    let half = (light + view).normalize();
    let spec = saturate(n.dot(half)).powf(60.0) * GLOSS * smoothstep(0.35, 0.7, thick) * (1.0 - crack);
    // Destellos de cristales que centellean con el tiempo.
    let glitter = smoothstep(0.93, 0.99, noise3(p.x * 70.0 + f.time * 0.5, p.y * 70.0, p.z * 70.0));
    let glitter = glitter * SPARKLE * saturate(ndl * 2.0);

    let rim = (1.0 - saturate(n.dot(view))).powi(3);
    color + rgb(1.0, 1.0, 1.0) * (spec + glitter) + rgb(0.35, 0.65, 1.0) * (rim * 0.4 * saturate(ndl + 0.3))
}
