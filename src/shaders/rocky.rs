use raylib::prelude::*;
use super::*;
use crate::noise::{fbm, noise_v};

// ---- Parametros del planeta rocoso ----
/// Frecuencia del relieve base: mas alto = continentes mas pequenos y numerosos.
pub const TERRAIN_SCALE: f32 = 1.9;
/// Altura del nivel del mar en el campo de ruido: mas alto = mas oceano.
pub const SEA_LEVEL: f32 = 0.02;
/// Cobertura de nubes (umbral del ruido): mas alto = cielo mas despejado.
pub const CLOUD_COVER: f32 = 0.50;
/// Frecuencia de las nubes.
pub const CLOUD_SCALE: f32 = 2.4;
/// Velocidad (rad/s) a la que las nubes se desplazan respecto al terreno.
pub const CLOUD_SPEED: f32 = 0.10;
/// Altura aparente de las nubes sobre el suelo: controla cuanto se separa la
/// sombra de su nube cuando la luz es rasante.
pub const CLOUD_HEIGHT: f32 = 0.06;
/// Oscuridad maxima de la sombra de una nube sobre el suelo (0..1).
pub const SHADOW_STRENGTH: f32 = 0.6;
/// Intensidad del relieve (bump mapping) en tierra firme.
pub const BUMP_STRENGTH: f32 = 0.15;
/// Brillo del reflejo especular del oceano.
pub const OCEAN_SPECULAR: f32 = 0.9;

fn seed() -> Vector3 {
    Vector3::new(3.7, 1.3, 8.1)
}

/// Campo de altura procedural del terreno. Se usa para decidir oceano/tierra,
/// colorear por altitud y para el bump mapping.
fn height(p: Vector3) -> f32 {
    fbm(p * TERRAIN_SCALE + seed(), 5)
}

/// Mascara de nubes en [0, 1]. Las nubes viven en el espacio de objeto pero
/// giran aparte (CLOUD_SPEED), asi se mueven independientes del terreno.
pub fn cloud_mask(p: Vector3, time: f32) -> f32 {
    let q = rotate_y(p, time * CLOUD_SPEED);
    // Un poco de distorsion alarga las nubes en filamentos en vez de manchas.
    let warp = noise_v(q * 1.5 + Vector3::new(9.0, 0.0, 2.0)) * 0.35;
    let n = fbm(q * CLOUD_SCALE + Vector3::new(warp, 0.0, -warp), 4) * 0.5 + 0.5;
    smoothstep(CLOUD_COVER, CLOUD_COVER + 0.22, n)
}

pub fn rocky(f: &FragmentIn) -> Vector3 {
    let p = f.point;
    let n = f.normal_obj;
    let light = f.light_obj;
    let view = f.rotation.transpose().apply(f.view);

    // ---------- Capa 1: terreno ----------
    let h = height(p);
    let e = h - SEA_LEVEL; // elevacion sobre el mar
    let is_land = e > 0.0;
    let moisture = fbm(p * 1.6 + Vector3::new(11.0, 3.0, 7.0), 3);
    let lat = p.y.abs() + 0.05 * noise_v(p * 6.0);

    let deep = rgb(0.015, 0.06, 0.26);
    let shallow = rgb(0.06, 0.40, 0.58);
    let sand = rgb(0.80, 0.72, 0.46);
    let forest = rgb(0.07, 0.28, 0.11);
    let grass = rgb(0.22, 0.47, 0.16);
    let desert = rgb(0.76, 0.54, 0.28);
    let rock = rgb(0.43, 0.38, 0.35);
    let snow = rgb(0.95, 0.96, 1.0);
    let ice = rgb(0.86, 0.93, 1.0);

    let vegetation = mix(forest, grass, smoothstep(-0.2, 0.3, moisture));
    let mut land = mix(vegetation, desert, smoothstep(0.02, -0.25, moisture));
    land = mix(sand, land, smoothstep(0.0, 0.06, e));
    land = mix(land, rock, smoothstep(0.28, 0.42, e));
    land = mix(land, snow, smoothstep(0.45, 0.58, e));

    let ocean = mix(shallow, deep, smoothstep(0.0, -0.45, e));

    let polar = smoothstep(0.80, 0.92, lat);
    let mut albedo = if is_land { land } else { ocean };
    albedo = mix(albedo, ice, polar);
    let is_ocean = if is_land || polar > 0.5 { 0.0 } else { 1.0 };

    // ---------- Normal: bump mapping en tierra, olitas en el oceano ----------
    let mut ns = n;
    if is_land && polar < 0.5 {
        let eps = 0.01;
        let hx = height(p + Vector3::new(eps, 0.0, 0.0));
        let hy = height(p + Vector3::new(0.0, eps, 0.0));
        let hz = height(p + Vector3::new(0.0, 0.0, eps));
        ns = bump_normal(n, h, hx, hy, hz, eps, BUMP_STRENGTH);
    } else if is_ocean > 0.5 {
        let t = f.time * 0.4;
        let w = Vector3::new(
            noise_v(p * 40.0 + Vector3::new(t, 0.0, 0.0)),
            noise_v(p * 40.0 + Vector3::new(0.0, t, 5.0)),
            noise_v(p * 40.0 + Vector3::new(3.0, 5.0, t)),
        );
        ns = (n + w * 0.04).normalize();
    }

    // ---------- Sombras de nubes ----------
    // La nube que sombrea al punto p esta sobre la recta p -> luz, a una altura
    // CLOUD_HEIGHT. Se proyecta ese punto a la capa de nubes y se consulta la
    // misma mascara: la sombra se mueve con las nubes y con la direccion de la luz.
    let ndl = n.dot(light);
    let slant = CLOUD_HEIGHT / ndl.max(0.15);
    let shadow_src = (p + light * slant).normalize();
    let shadow = cloud_mask(shadow_src, f.time) * SHADOW_STRENGTH;

    // ---------- Capa 2: iluminacion (respuestas distintas por region) ----------
    let diffuse = saturate(ns.dot(light));
    let lit = (1.0 - shadow) * smoothstep(0.0, 0.1, ndl);

    let half = (light + view).normalize();
    let spec_ocean = saturate(ns.dot(half)).powf(90.0) * OCEAN_SPECULAR * is_ocean;
    let spec_ice = saturate(ns.dot(half)).powf(25.0) * 0.25 * polar;

    let ambient = 0.035;
    let mut color = albedo * (ambient + diffuse * (1.0 - shadow));
    color = color + rgb(1.0, 0.97, 0.9) * ((spec_ocean + spec_ice) * lit);

    // ---------- Capa 3: lado nocturno ----------
    // Luces de ciudades: solo en tierra baja, solo donde no llega el sol.
    let night = 1.0 - smoothstep(-0.12, 0.08, ndl);
    let city_noise = noise_v(p * 22.0) * 0.5 + 0.5;
    let city_region = smoothstep(0.1, 0.5, fbm(p * 5.0 + Vector3::new(2.0, 9.0, 4.0), 2) * 0.5 + 0.5);
    let city = smoothstep(0.62, 0.78, city_noise)
        * city_region
        * if is_land && polar < 0.3 && e < 0.22 { 1.0 } else { 0.0 };

    // ---------- Capa 4: nubes ----------
    let cloud = cloud_mask(p, f.time);
    let cloud_light = 0.05 + 0.95 * saturate(ndl * 1.1 + 0.05);
    let cloud_color = rgb(0.97, 0.97, 1.0) * cloud_light;

    color = color + rgb(1.0, 0.78, 0.38) * (city * night * (1.0 - cloud * 0.85) * 0.9);
    color = mix(color, cloud_color, cloud * 0.93);

    // Atmosfera: halo azul en el borde, mas fuerte del lado iluminado.
    let rim = (1.0 - saturate(n.dot(view))).powi(3);
    color = color + rgb(0.25, 0.5, 1.0) * (rim * (0.12 + 0.7 * saturate(ndl + 0.25)));

    color
}
