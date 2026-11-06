use raylib::prelude::*;
use super::*;
use super::rings::shadow_on_planet;
use crate::noise::{fbm, noise3};

// ---- Parametros del gigante gaseoso ----
/// Numero de bandas: mas alto = mas bandas, mas finas.
pub const BAND_FREQ: f32 = 12.0;
/// Cuanto deforma el ruido los bordes de las bandas (transiciones liquidas).
pub const TURBULENCE: f32 = 0.16;
/// Velocidad de la corriente (rad/s); cada banda se desplaza a un ritmo distinto.
pub const FLOW_SPEED: f32 = 0.05;
/// Tamano de la tormenta en el espacio local (ancho, alto).
pub const STORM_RX: f32 = 0.30;
pub const STORM_RY: f32 = 0.17;
/// Cuanto gira el flujo alrededor de la tormenta (radianes en el centro).
pub const STORM_TWIST: f32 = 3.2;

fn storm_center() -> Vector3 {
    Vector3::new(0.25, -0.10, 0.96).normalize()
}

/// Gradiente de 5 colores que convierte el valor de banda (0..1) en color.
fn band_palette(v: f32) -> Vector3 {
    let cream = rgb(0.94, 0.86, 0.68);
    let tan = rgb(0.80, 0.60, 0.38);
    let rust = rgb(0.62, 0.36, 0.22);
    let brown = rgb(0.38, 0.23, 0.17);
    let v = saturate(v);
    if v < 0.33 {
        mix(brown, rust, smoothstep(0.0, 0.33, v))
    } else if v < 0.66 {
        mix(rust, tan, smoothstep(0.33, 0.66, v))
    } else {
        mix(tan, cream, smoothstep(0.66, 1.0, v))
    }
}

pub fn gas_giant(f: &FragmentIn) -> Vector3 {
    let p = f.point;
    let n = f.normal_obj;
    let t = f.time;

    // ---------- Tormenta: distorsion localizada ----------
    // Se miden coordenadas locales (dx, dy) alrededor del centro de la tormenta
    // y se rotan un angulo que decae con la distancia. Ese desplazamiento se
    // aplica al punto de muestreo de las bandas: las bandas se curvan y se
    // enrollan alrededor de la mancha en vez de ser cortadas por un circulo.
    let c = storm_center();
    let east = Vector3::new(0.0, 1.0, 0.0).cross(c).normalize();
    let north = c.cross(east);
    let dx = p.dot(east) / STORM_RX;
    let dy = p.dot(north) / STORM_RY;
    let r2 = dx * dx + dy * dy;
    let facing = if p.dot(c) > 0.0 { 1.0 } else { 0.0 };
    let falloff = (-r2 * 0.55).exp() * facing;
    let theta = STORM_TWIST * falloff;
    let (s, co) = theta.sin_cos();
    let dx2 = co * dx - s * dy;
    let dy2 = s * dx + co * dy;
    let mut q = p + east * ((dx2 - dx) * STORM_RX) + north * ((dy2 - dy) * STORM_RY);

    // ---------- Corriente: cada banda avanza a distinta velocidad ----------
    let shear = 0.7 + 0.6 * (p.y * 5.0).sin();
    q = rotate_y(q, t * FLOW_SPEED * shear);

    // ---------- Capa 1: bandas con bordes liquidos ----------
    // La latitud se perturba con ruido antes de mapearla a una banda; asi los
    // limites entre bandas ondulan y se mezclan como un fluido.
    let wobble = fbm(q * 3.0 + Vector3::new(0.0, 0.0, t * 0.04), 5);
    let swirl = fbm(q * 8.0 + Vector3::new(4.0, t * 0.06, 1.0), 3);
    let lat = q.y + TURBULENCE * wobble + 0.04 * swirl;
    let irregular = 0.8 * noise3(lat * 2.5, 6.0, 1.0);
    let phase = lat * BAND_FREQ + irregular;
    let band = 0.5 + 0.38 * phase.sin() + 0.12 * (phase * 2.3 + 1.0).sin();
    let mut color = band_palette(band);

    // ---------- Capa 2: filamentos finos a lo largo de las bandas ----------
    let streak = fbm(Vector3::new(q.x * 3.0, q.y * 26.0, q.z * 3.0) + Vector3::new(0.0, 0.0, 2.0), 3);
    color = color * (1.0 + 0.16 * streak);

    // ---------- Capa 3: la tormenta ----------
    let r = r2.sqrt();
    let core = (1.0 - smoothstep(0.55, 1.0, r)) * facing;
    let collar = smoothstep(0.85, 1.05, r) * (1.0 - smoothstep(1.05, 1.5, r)) * facing;
    let ang = dy2.atan2(dx2);
    let arms = 0.5 + 0.5 * (ang * 2.0 + r * 6.0 - t * 0.4).sin();
    let storm_color = mix(rgb(0.66, 0.17, 0.09), rgb(0.90, 0.48, 0.28), arms * 0.55 * (1.0 - core * 0.4));
    color = mix(color, rgb(0.97, 0.90, 0.78), collar * 0.30);
    color = mix(color, storm_color, core * 0.92);

    // ---------- Capa 4: atmosfera y luz ----------
    // Los polos se enfrian hacia un gris azulado.
    let polar = smoothstep(0.62, 0.95, p.y.abs());
    color = mix(color, rgb(0.46, 0.50, 0.58) * (0.8 + 0.4 * band), polar * 0.55);

    let view_dot = saturate(n.dot(f.rotation.transpose().apply(f.view)));
    let ndl = n.dot(f.light_obj);
    let wrapped = saturate((ndl + 0.12) / 1.12); // terminador suave: atmosfera gruesa
    let ring_shadow = shadow_on_planet(p, f.light_obj);
    let lighting = 0.04 + 0.96 * wrapped * (1.0 - 0.75 * ring_shadow);

    color = color * lighting;
    // Neblina clara en el limbo.
    color = color + rgb(0.9, 0.78, 0.6) * ((1.0 - view_dot).powi(3) * 0.28 * wrapped);

    color
}
