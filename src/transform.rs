use raylib::prelude::*;

/// Matriz 3x3 para rotaciones (la inversa de una rotacion es su transpuesta).
#[derive(Clone, Copy)]
pub struct Mat3 {
    m: [[f32; 3]; 3],
}

impl Mat3 {
    pub fn identity() -> Self {
        Mat3 { m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] }
    }

    pub fn rot_x(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Mat3 { m: [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]] }
    }

    pub fn rot_y(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Mat3 { m: [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]] }
    }

    pub fn rot_z(a: f32) -> Self {
        let (s, c) = a.sin_cos();
        Mat3 { m: [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]] }
    }

    pub fn mul(&self, o: &Mat3) -> Mat3 {
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    r[i][j] += self.m[i][k] * o.m[k][j];
                }
            }
        }
        Mat3 { m: r }
    }

    pub fn apply(&self, v: Vector3) -> Vector3 {
        Vector3::new(
            self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * v.z,
            self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * v.z,
            self.m[2][0] * v.x + self.m[2][1] * v.y + self.m[2][2] * v.z,
        )
    }

    pub fn transpose(&self) -> Mat3 {
        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                r[i][j] = self.m[j][i];
            }
        }
        Mat3 { m: r }
    }
}

/// Camara fija mirando hacia -Z con proyeccion en perspectiva.
#[derive(Clone, Copy)]
pub struct Camera {
    pub position: Vector3,
    pub focal: f32,
    pub width: f32,
    pub height: f32,
}

impl Camera {
    pub fn new(position: Vector3, fov_y_degrees: f32, width: f32, height: f32) -> Self {
        let focal = height / (2.0 * (fov_y_degrees.to_radians() / 2.0).tan());
        Camera { position, focal, width, height }
    }

    /// Devuelve (x_pantalla, y_pantalla, profundidad). La profundidad es la
    /// distancia a lo largo del eje de la camara (siempre positiva delante).
    pub fn project(&self, world: Vector3) -> (f32, f32, f32) {
        let d = self.position.z - world.z;
        let x = self.width / 2.0 + (world.x - self.position.x) / d * self.focal;
        let y = self.height / 2.0 - (world.y - self.position.y) / d * self.focal;
        (x, y, d)
    }
}
