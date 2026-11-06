use raylib::prelude::*;
use super::*;
use crate::noise::{cell_hash, fbm};

// ---- Parametros de la luna ----
/// Frecuencia de los crateres grandes (mas alto = mas crateres, mas pequenos).
pub const CRATER_SCALE: f32 = 4.0;
/// Intensidad del relieve de los crateres sobre la luz.
pub const CRATER_BUMP: f32 = 0.12;

/// Altura de crateres en una capa: cada celda de la rejilla 3D contiene un
/// crater con centro y radio aleatorios. Se suman los de las 27 celdas vecinas
/// para que los crateres puedan cruzar los bordes de celda y solaparse.
fn craters(p: Vector3, scale: f32) -> f32 {
    let q = p * scale;
    let (cx, cy, cz) = (q.x.floor() as i32, q.y.floor() as i32, q.z.floor() as i32);
    let mut sum = 0.0;

    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let (ix, iy, iz) = (cx + dx, cy + dy, cz + dz);
                let r = cell_hash(ix, iy, iz);
                if r[3] < 0.35 {
                    continue; // no todas las celdas tienen crater
                }
                let center = Vector3::new(ix as f32 + r[0], iy as f32 + r[1], iz as f32 + r[2]);
                let radius = 0.18 + 0.32 * r[3];
                let u = (q - center).length() / radius;
                if u < 1.6 {
                    let bowl = if u < 1.0 { -(1.0 - u * u) } else { 0.0 };
                    let rim = 0.45 * (-((u - 1.0) / 0.18).powi(2)).exp();
                    sum += (bowl + rim) * radius;
                }
            }
        }
    }
    sum
}

fn height(p: Vector3) -> f32 {
    craters(p, CRATER_SCALE) + 0.5 * craters(p + Vector3::new(7.0, 1.0, 3.0), CRATER_SCALE * 2.7)
}

/// Luna: gris con maria oscuros (regiones bajas), crateres con borde elevado
/// y relieve calculado por bump mapping para que respondan a la luz.
pub fn moon(f: &FragmentIn) -> Vector3 {
    let p = f.point;
    let n = f.normal_obj;
    let light = f.light_obj;

    let h = height(p);
    let eps = 0.012;
    let hx = height(p + Vector3::new(eps, 0.0, 0.0));
    let hy = height(p + Vector3::new(0.0, eps, 0.0));
    let hz = height(p + Vector3::new(0.0, 0.0, eps));
    let ns = bump_normal(n, h, hx, hy, hz, eps, CRATER_BUMP);

    let mare = smoothstep(-0.05, 0.25, fbm(p * 1.4 + Vector3::new(4.0, 8.0, 1.0), 3));
    let highland = rgb(0.64, 0.62, 0.58);
    let dark = rgb(0.30, 0.29, 0.29);
    let mut albedo = mix(highland, dark, mare * 0.85);

    // Suelo de crater un poco mas oscuro y borde mas claro.
    albedo = albedo * (1.0 + 0.9 * h.clamp(-0.4, 0.4));

    let diffuse = saturate(ns.dot(light));
    albedo * (0.03 + diffuse * 0.97)
}
