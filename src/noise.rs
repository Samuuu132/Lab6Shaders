use raylib::prelude::*;

// Ruido simplex 3D (variante de la familia OpenSimplex: rejilla simplectica en
// vez de cubica, sin los artefactos axiales del ruido Perlin clasico).
// Se evalua directamente sobre el punto 3D de la esfera, asi no hay costuras
// de UV en los polos ni en el meridiano.

const GRAD3: [[f32; 3]; 12] = [
    [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0], [1.0, -1.0, 0.0], [-1.0, -1.0, 0.0],
    [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], [1.0, 0.0, -1.0], [-1.0, 0.0, -1.0],
    [0.0, 1.0, 1.0], [0.0, -1.0, 1.0], [0.0, 1.0, -1.0], [0.0, -1.0, -1.0],
];

/// Tabla de permutacion fija (barajada con un LCG en tiempo de compilacion),
/// duplicada para evitar el modulo al indexar.
const fn build_perm() -> [u8; 512] {
    let mut base = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        base[i] = i as u8;
        i += 1;
    }

    let mut seed: u32 = 0x9E3779B9;
    let mut i = 255;
    while i > 0 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let j = ((seed >> 8) as usize) % (i + 1);
        let tmp = base[i];
        base[i] = base[j];
        base[j] = tmp;
        i -= 1;
    }

    let mut perm = [0u8; 512];
    let mut i = 0;
    while i < 512 {
        perm[i] = base[i & 255];
        i += 1;
    }
    perm
}

static PERM: [u8; 512] = build_perm();

fn corner(gi: usize, x: f32, y: f32, z: f32) -> f32 {
    let t = 0.6 - x * x - y * y - z * z;
    if t < 0.0 {
        return 0.0;
    }
    let g = GRAD3[gi];
    let t2 = t * t;
    t2 * t2 * (g[0] * x + g[1] * y + g[2] * z)
}

/// Ruido simplex 3D en el rango aproximado [-1, 1].
pub fn noise3(x: f32, y: f32, z: f32) -> f32 {
    const F3: f32 = 1.0 / 3.0;
    const G3: f32 = 1.0 / 6.0;

    let s = (x + y + z) * F3;
    let i = (x + s).floor();
    let j = (y + s).floor();
    let k = (z + s).floor();

    let t = (i + j + k) * G3;
    let x0 = x - (i - t);
    let y0 = y - (j - t);
    let z0 = z - (k - t);

    // Orden en que se recorren los ejes dentro del tetraedro.
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else if y0 < z0 {
        (0, 0, 1, 0, 1, 1)
    } else if x0 < z0 {
        (0, 1, 0, 0, 1, 1)
    } else {
        (0, 1, 0, 1, 1, 0)
    };

    let x1 = x0 - i1 as f32 + G3;
    let y1 = y0 - j1 as f32 + G3;
    let z1 = z0 - k1 as f32 + G3;
    let x2 = x0 - i2 as f32 + 2.0 * G3;
    let y2 = y0 - j2 as f32 + 2.0 * G3;
    let z2 = z0 - k2 as f32 + 2.0 * G3;
    let x3 = x0 - 1.0 + 3.0 * G3;
    let y3 = y0 - 1.0 + 3.0 * G3;
    let z3 = z0 - 1.0 + 3.0 * G3;

    let ii = (i as i32 & 255) as usize;
    let jj = (j as i32 & 255) as usize;
    let kk = (k as i32 & 255) as usize;

    let p = |a: usize, b: usize, c: usize| PERM[a + PERM[b + PERM[c] as usize] as usize] as usize % 12;

    let g0 = p(ii, jj, kk);
    let g1 = p(ii + i1 as usize, jj + j1 as usize, kk + k1 as usize);
    let g2 = p(ii + i2 as usize, jj + j2 as usize, kk + k2 as usize);
    let g3 = p(ii + 1, jj + 1, kk + 1);

    32.0 * (corner(g0, x0, y0, z0)
        + corner(g1, x1, y1, z1)
        + corner(g2, x2, y2, z2)
        + corner(g3, x3, y3, z3))
}

pub fn noise_v(p: Vector3) -> f32 {
    noise3(p.x, p.y, p.z)
}

/// Movimiento browniano fractal: suma de octavas con frecuencia x2 y amplitud x0.5.
/// Resultado normalizado a [-1, 1].
pub fn fbm(p: Vector3, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut norm = 0.0;
    let mut q = p;
    for _ in 0..octaves {
        sum += amp * noise_v(q);
        norm += amp;
        q = q * 2.03;
        amp *= 0.5;
    }
    sum / norm
}

/// Ruido "ridged": crestas afiladas donde el ruido cruza por cero. Devuelve [0, 1]
/// con los maximos sobre las lineas de cruce (sirve para bordes de celdas o grietas).
pub fn ridged(p: Vector3, octaves: u32) -> f32 {
    let mut sum = 0.0;
    let mut amp = 0.5;
    let mut norm = 0.0;
    let mut q = p;
    for _ in 0..octaves {
        sum += amp * (1.0 - noise_v(q).abs());
        norm += amp;
        q = q * 2.07;
        amp *= 0.5;
    }
    sum / norm
}
