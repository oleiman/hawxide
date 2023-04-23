use crate::vec3::{Point3,Color};

use std::rc::Rc;

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

pub struct SolidColor {
    color_val: Color,
}

impl SolidColor {
    pub fn new(r: f64, g: f64, b: f64) -> Self{
        SolidColor {
            color_val: Color(r, g, b),
        }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.color_val
    }
}

pub struct CheckerTexture {
    even: Rc<dyn Texture>,
    odd: Rc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(c1: &Color, c2: &Color) -> CheckerTexture {
        CheckerTexture {
            even: Rc::new(SolidColor { color_val: *c1}),
            odd: Rc::new(SolidColor {color_val: *c2}),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let sines =
            f64::sin(10.0 * p.x()) * f64::sin(10.0 * p.y()) * f64::sin(10.0 * p.z());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}
