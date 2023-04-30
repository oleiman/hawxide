use crate::vec3::{Point3,Color};

use crate::perlin::Perlin;

use std::sync::Arc;
use image;
use image::{GenericImageView,DynamicImage};

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
    even: Arc<dyn Texture + Sync + Send>,
    odd: Arc<dyn Texture + Sync + Send>,
}

impl CheckerTexture {
    pub fn new(c1: &Color, c2: &Color) -> CheckerTexture {
        CheckerTexture {
            even: Arc::new(SolidColor { color_val: *c1}),
            odd: Arc::new(SolidColor {color_val: *c2}),
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

pub struct MarbleTexture {
    noise: Perlin,
    scale: f64,
}

impl MarbleTexture {
    pub fn new(scale : f64) -> MarbleTexture {
        MarbleTexture {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for MarbleTexture {
    // perlin interpolation can return negative numbers, so we add 1 and divide by 2
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        Color(1.0, 1.0, 1.0) *
            0.5 * (1. +
                   f64::sin(self.scale * p.z() +
                            10. * self.noise.turb(p, None)))
    }
}

pub struct ImageTexture {
    img: DynamicImage,
    width: u32,
    height: u32,
}


impl ImageTexture {
    const COLOR_SCALE: f64 = 1.0 / 255.0;
    pub fn new(fname: &str) -> Self {
        let img = image::open(fname).expect("File not found!");
        eprintln!("{} - dimensions: {:?}; color: {:?}",
                  fname, img.dimensions(), img.color());
        let (width, height) = img.dimensions();

        ImageTexture {
            img, width, height,
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Point3) -> Color {
        // Clamp input texture coords to [0,1] x [1,0]
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // flip V to image coords

        let mut i = (u * self.width as f64) as u32;
        let mut j = (v * self.height as f64) as u32;

        i = i.clamp(0, self.width - 1);
        j = j.clamp(0, self.height - 1);

        let pixel = self.img.get_pixel(i, j);

        Color(
            ImageTexture::COLOR_SCALE * pixel[0] as f64,
            ImageTexture::COLOR_SCALE * pixel[1] as f64,
            ImageTexture::COLOR_SCALE * pixel[2] as f64
        )

    }
}
