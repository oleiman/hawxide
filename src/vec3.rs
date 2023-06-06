#![allow(unused)]

use std::ops;
use std::fmt;
use std::assert;
use std::io::Write;
use crate::util::{
    random::{double, double_range},
    clamp, PI,
};

#[derive(Copy, Clone, Default)]
pub struct Vec3(pub f64, pub f64, pub f64);

pub type Point3 = Vec3;
#[allow(non_snake_case)]
#[must_use]
pub const fn Point3(x: f64, y: f64, z: f64) -> Point3 {
    Vec3(x, y, z)
}
pub type Color = Vec3;
#[allow(non_snake_case)]
#[must_use]
pub const fn Color(r: f64, g: f64, b: f64) -> Point3 {
    Vec3(r, g, b)
}

#[derive(Copy, Clone)]
pub enum Axis {
    X, Y, Z,
}

impl Vec3 {
    #[must_use]
    pub fn new() -> Vec3 { Vec3(0.0, 0.0, 0.0) }
    #[must_use]
    pub fn x(&self) -> f64 { self.0 }
    #[must_use]
    pub fn y(&self) -> f64 { self.1 }
    #[must_use]
    pub fn z(&self) -> f64 { self.2 }

    #[must_use]
    pub fn r(&self) -> f64 { self.0 }
    #[must_use]
    pub fn g(&self) -> f64 { self.1 }
    #[must_use]
    pub fn b(&self) -> f64 { self.2 }

    #[must_use]
    pub fn exp(&self) -> Self {
        Vec3(self.r().exp(), self.g().exp(), self.b().exp())
    }

    #[must_use]
    pub fn axis(&self, d: Axis) -> f64 {
        match (d) {
            Axis::X => self.x(),
            Axis::Y => self.y(),
            Axis::Z => self.z(),
        }
    }

    #[must_use]
    pub fn len(&self) -> f64 { self.len_squared().sqrt() }

    #[must_use]
    pub fn len_squared(&self) -> f64 {
        self.0 * self.0 + self.1 * self.1 + self.2 * self.2
    }

    #[must_use]
    pub fn unit_vector(&self) -> Vec3 {
        *self / self.len()
    }

    #[must_use]
    pub fn near_zero(&self) -> bool {
        const S : f64 = 1e-8;
        (self.0.abs() < S) && (self.1.abs() < S) && (self.2.abs() < S)
    }

    #[must_use]
    pub fn is_nan(&self) -> bool {
        self.0.is_nan() || self.1.is_nan() || self.2.is_nan()
    }

    #[must_use]
    pub fn random() -> Self {
        Vec3(double(), double(), double())
    }

    #[must_use]
    pub fn random_range(min : f64, max : f64) -> Self {
        Vec3(
            double_range(min, max),
            double_range(min, max),
            double_range(min, max)
        )
    }

    #[must_use]
    pub fn random_in_unit_disk() -> Vec3 {
        let mut p = Vec3(
            double_range(-1.0, 1.0),
            double_range(-1.0, 1.0),
            0.0
        );
        while p.len_squared() >= 1.0 {
            p = Vec3(
                double_range(-1.0, 1.0),
                double_range(-1.0, 1.0),
                0.0
            );
        }
        p
    }

    #[must_use]
    pub fn random_in_unit_sphere() -> Vec3 {
        let mut p = Self::random_range(-1., 1.);
        while p.len_squared() >= 1.0 {
            p = Self::random_range(-1., 1.);
        }
        p
    }

    #[must_use]
    pub fn random_unit_vector() -> Vec3 {
        Self::random_in_unit_sphere().unit_vector()
    }

    #[must_use]
    pub fn random_in_hemisphere(norm: Vec3) -> Vec3 {
        let in_unit_sphere = Self::random_in_unit_sphere();
        if dot(in_unit_sphere, norm) > 0.0 {
            in_unit_sphere
        } else {
            -in_unit_sphere
        }
    }

    #[must_use]
    pub fn random_to_sphere(r: f64, dist_squared: f64) -> Vec3 {
        let r1 = double();
        let r2 = double();
        let z = 1.0 + r2 * (f64::sqrt(1.0 - r*r/dist_squared) - 1.0);

        let phi = 2.0 * PI * r1;
        let x = f64::cos(phi) * f64::sqrt(1.0 - z * z);
        let y = f64::sin(phi) * f64::sqrt(1.0 - z * z);

        Vec3(x, y, z)
    }
}

#[must_use]
pub fn dot(u: Vec3, v: Vec3) -> f64 {
    (u.0 * v.0) + (u.1 * v.1) + (u.2 * v.2)
}

#[must_use]
pub fn cross(u: Vec3, v: Vec3) -> Vec3 {
    Vec3(
        u.1 * v.2 - u.2 * v.1,
        u.2 * v.0 - u.0 * v.2,
        u.0 * v.1 - u.1 * v.0
    )
}

#[must_use]
pub fn reflect(v: Vec3, n: Vec3) -> Vec3 {
    v - (2. * dot(v, n) * n)
}

#[must_use]
pub fn refract(uv : Vec3, n: Vec3, etai_over_etat: f64) -> Vec3 {
    let cos_theta = f64::min(dot(-uv, n), 1.0);
    let r_out_perp = etai_over_etat * (uv + cos_theta * n);
    let r_out_parallel = -(1.0 - r_out_perp.len_squared()).abs().sqrt() * n;
    r_out_perp + r_out_parallel
}

impl ops::Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < 3, "Index {} out of range for Vec3", idx);
        match idx {
            0 => &self.0,
            1 => &self.1,
            _ => &self.2,
        }
    }
}

impl ops::IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        assert!(idx < 3, "Index {} out of range for Vec3", idx);
        match idx {
            0 => &mut self.0,
            1 => &mut self.1,
            _ => &mut self.2,
        }
    }

}

// TODO(oren): would be nice if these could take references
impl ops::AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
        self.1 += other.1;
        self.2 += other.2;
    }
}

impl ops::MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, t: f64) {
        self.0 *= t;
        self.1 *= t;
        self.2 *= t;
    }
}

impl ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, t: f64) {
        *self *= (1. / t);
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3(-self.0, -self.1, -self.2)
    }
}

impl ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Self) -> Self::Output {
        Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Self) -> Self::Output {
        Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl ops::Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, other: Self) -> Self::Output {
        Vec3(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Self::Output {
        Vec3(self * vec.0, self * vec.1, self * vec.2)
    }
}

impl ops::Mul<Vec3> for i32 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Self::Output {
        f64::from(self) * vec
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs : f64) -> Vec3 {
        rhs * self
    }
}

impl ops::Mul<i32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs : i32) -> Self::Output {
        rhs * self
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: f64) -> Vec3 {
        (1. / rhs) * self
    }
}

impl ops::Div<i32> for Vec3 {
    type Output = Vec3;

    fn div(self, rhs: i32) -> Vec3 {
        (1. / f64::from(rhs)) * self
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.0, self.1, self.2)
    }
}

impl From<Vec3> for [f64;3] {
    fn from(v: Vec3) -> [f64;3] {
        [v.0, v.1, v.2]
    }
}

impl From<[f64;3]> for Vec3 {
    fn from(a: [f64;3]) -> Vec3 {
        Vec3(a[0], a[1], a[2])
    }
}

#[allow(clippy::cast_possible_truncation)]
pub fn write_color<W: Write>(writer: &mut W, col : Color, samples_per_pixel : i32) {
    // Divide the color by the number of samples
    let scale = 1.0 / f64::from(samples_per_pixel);

    // assert!(col.r() >= 0. && col.g() >= 0. && col.b() >= 0.);
    // assert!(!col.r().is_nan() && !col.g().is_nan() && !col.b().is_nan());

    let r = (scale * col.r()).sqrt();
    let g = (scale * col.g()).sqrt();
    let b = (scale * col.b()).sqrt();

    // assert!(!r.is_infinite() && !g.is_infinite() && !b.is_infinite());
    // assert!(!r.is_nan() && !g.is_nan() && !b.is_nan());

    writeln!(writer, "{} {} {}",
        (256. * r.clamp(0., 0.999)) as u8,
        (256. * g.clamp(0., 0.999)) as u8,
        (256. * b.clamp(0., 0.999)) as u8,
    );
}

