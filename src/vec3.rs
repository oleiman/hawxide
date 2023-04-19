#![allow(unused)]

use std::ops;
use std::fmt;
use std::assert;
use std::io::Write;
use crate::util::{
    random::{random_double, random_double_in_range},
    clamp
};


#[derive(Copy, Clone)]
pub struct Vec3(pub f64, pub f64, pub f64);

pub type Point3 = Vec3;
#[allow(non_snake_case)]
pub const fn Point3(x: f64, y: f64, z: f64) -> Point3 {
    return Vec3(x, y, z);
}
pub type Color = Vec3;
#[allow(non_snake_case)]
pub const fn Color(r: f64, g: f64, b: f64) -> Point3 {
    return Vec3(r, g, b);
}

impl Vec3 {
    pub fn x(&self) -> f64 { return self.0; }
    pub fn y(&self) -> f64 { return self.1; }
    pub fn z(&self) -> f64 { return self.2; }

    pub fn r(&self) -> f64 { return self.0; }
    pub fn g(&self) -> f64 { return self.1; }
    pub fn b(&self) -> f64 { return self.2; }

    pub fn len(&self) -> f64 { return self.len_squared().sqrt(); }
    pub fn len_squared(&self) -> f64 {
        return self.0 * self.0 + self.1 * self.1 + self.2 * self.2;
    }

    pub fn near_zero(&self) -> bool {
        let s : f64 = 1e-8;
        return self.0.abs() < s && self.1.abs() < s && self.2.abs() < s;
    }

    pub fn unit_vector(&self) -> Vec3 {
        return self / self.len();
    }

    // TODO(oren): random
    pub fn random() -> Self {
        return Vec3(random_double(), random_double(), random_double());
    }

    pub fn random_in_range(min : f64, max : f64) -> Self {
        return Vec3(
            random_double_in_range(min, max),
            random_double_in_range(min, max),
            random_double_in_range(min, max)
        );
    }

    pub fn random_in_unit_disk() -> Vec3 {
        let mut p = Vec3(
            random_double_in_range(-1.0, 1.0),
            random_double_in_range(-1.0, 1.0),
            0.0
        );
        while p.len_squared() >= 1.0 {
            p = Vec3(
                random_double_in_range(-1.0, 1.0),
                random_double_in_range(-1.0, 1.0),
                0.0
            );
        }
        return p;
    }

    pub fn random_in_unit_sphere() -> Vec3 {
        let mut p = Self::random_in_range(-1., 1.);
        while p.len_squared() >= 1.0 {
            p = Self::random_in_range(-1., 1.);
        }
        return p;
    }

    pub fn random_unit_vector() -> Vec3 {
        return Self::random_in_unit_sphere().unit_vector();
    }

    pub fn random_in_hemisphere(norm: &Vec3) -> Vec3 {
        let in_unit_sphere = Self::random_in_unit_sphere();
        if dot(&in_unit_sphere, norm) > 0.0 {
            return in_unit_sphere;
        } else {
            return -in_unit_sphere;
        }
    }

}


// Maybe dot should pass by value (implicity copy)?
pub fn dot(u: &Vec3, v: &Vec3) -> f64 {
    return (u.0 * v.0) + (u.1 * v.1) + (u.2 * v.2);
}

pub fn cross(u: &Vec3, v: &Vec3) -> Vec3 {
    return Vec3(
        u.1 * v.2 - u.2 * v.1,
        u.2 * v.0 - u.0 * v.2,
        u.0 * v.1 - u.1 * v.0
    );
}

pub fn reflect(v: &Vec3, n: &Vec3) -> Vec3 {
    return v - &(2. * dot(v, n) * n);
}

pub fn refract(uv : &Vec3, n: &Vec3, etai_over_etat: f64) -> Vec3 {
    // TODO(oren)
    assert!(false);
    return Vec3(0.,0.,0.);
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

// TODO(oren): implement in terms of operator*=
impl ops::DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, t: f64) {
        *self *= (1. / t);
    }
}

impl ops::Neg for &Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        return Vec3(-self.0, -self.1, -self.2);
    }
}

impl ops::Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        return Vec3(-self.0, -self.1, -self.2);
    }
}

impl ops::Add for &Vec3 {
    type Output = Vec3;

    fn add(self, other: Self) -> Self::Output {
        return Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2);
    }
}

impl ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Self) -> Self::Output {
        return Vec3(self.0 + other.0, self.1 + other.1, self.2 + other.2);
    }
}

impl ops::Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, other: Self) -> Self::Output {
        return Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2);
    }
}

impl ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Self) -> Self::Output {
        return Vec3(self.0 - other.0, self.1 - other.1, self.2 - other.2);
    }
}

impl ops::Mul for &Vec3 {
    type Output = Vec3;

    fn mul(self, other: Self) -> Self::Output {
        return Vec3(self.0 * other.0, self.1 * other.1, self.2 * other.2);
    }
}

impl ops::Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, other: Self) -> Self::Output {
        return Vec3(self.0 * other.0, self.1 * other.1, self.2 * other.2);
    }
}

impl ops::Mul<&Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, vec: &Vec3) -> Self::Output {
        return Vec3(self * vec.0, self * vec.1, self * vec.2);
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Self::Output {
        return Vec3(self * vec.0, self * vec.1, self * vec.2);
    }
}

impl ops::Mul<&Vec3> for i32 {
    type Output = Vec3;

    fn mul(self, vec: &Vec3) -> Self::Output {
        let t = self as f64;
        return t * vec;
    }
}

impl ops::Mul<Vec3> for i32 {
    type Output = Vec3;

    fn mul(self, vec: Vec3) -> Self::Output {
        let t = self as f64;
        return t * &vec;
    }
}

impl ops::Mul<f64> for &Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs : f64) -> Vec3 {
        return _rhs * self;
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs : f64) -> Vec3 {
        return _rhs * &self;
    }
}

impl ops::Mul<i32> for &Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs : i32) -> Self::Output {
        return _rhs * self;
    }
}

impl ops::Mul<i32> for Vec3 {
    type Output = Vec3;

    fn mul(self, _rhs : i32) -> Self::Output {
        return _rhs * &self;
    }
}

impl ops::Div<f64> for &Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: f64) -> Vec3 {
        return (1. / _rhs) * self;
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: f64) -> Vec3 {
        return (1. / _rhs) * &self;
    }
}

impl ops::Div<i32> for &Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: i32) -> Vec3 {
        let t = _rhs as f64;
        return (1. / t) * self;
    }
}

impl ops::Div<i32> for Vec3 {
    type Output = Vec3;

    fn div(self, _rhs: i32) -> Vec3 {
        let t = _rhs as f64;
        return (1. / t) * &self;
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.0, self.1, self.2)
    }
}

pub fn write_color<W: Write>(writer: &mut W, col : &Color, samples_per_pixel : i32) {
    // Divide the color by the number of samples

    let r = (col.r() / samples_per_pixel as f64).sqrt();
    let g = (col.g() / samples_per_pixel as f64).sqrt();
    let b = (col.b() / samples_per_pixel as f64).sqrt();

    writeln!(writer, "{} {} {}",
        (255.999 * r.clamp(0., 0.999)) as i32,
        (255.999 * g.clamp(0., 0.999)) as i32,
        (255.999 * b.clamp(0., 0.999)) as i32
    );
}

