use crate::vec3::{Point3,Color,Vec3};
use crate::util::random;
use crate::perlin::Perlin;

use std::sync::Arc;
use image;
use image::{GenericImageView,DynamicImage};

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

pub trait FloatTexture {
    fn value(&self, u: f64, v: f64, p: Point3) -> f64;
}

pub struct SolidColor {
    color_val: Color,
}

impl SolidColor {
    #[must_use]
    pub fn new(color_val: Color) -> Self {
        Self { color_val }
    }
}

impl From<SolidColor> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: SolidColor) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.color_val
    }
}

pub struct Checker {
    even: Arc<dyn Texture + Sync + Send>,
    odd: Arc<dyn Texture + Sync + Send>,
}

impl Checker {
    #[must_use]
    pub fn new(c1: Color, c2: Color) -> Self {
        Self {
            even: SolidColor::new(c1).into(),
            odd: SolidColor::new(c2).into(),
        }
    }
}

impl From<Checker> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: Checker) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for Checker {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        let sines =
            f64::sin(10.0 * p.x()) * f64::sin(10.0 * p.y()) * f64::sin(10.0 * p.z());
        if sines < 0.0 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

pub struct Marble {
    noise: Perlin,
    albedo: Arc<dyn Texture + Sync + Send>,
    scale: f64,
}

impl Marble {

    #[must_use]
    pub fn new(scale : f64) -> Self {
        Self::from_texture(
            scale,
            SolidColor::new(Color(1.0, 1.0, 1.0)).into(),
            1.0,
        )
    }

    #[must_use]
    pub fn with_point_scaling(scale: f64, p_scale: f64) -> Self {
        Self::from_texture(
            scale,
            SolidColor::new(Color(1.0, 1.0, 1.0)).into(),
            p_scale,
        )
    }

    #[must_use]
    pub fn from_texture(scale: f64, albedo: Arc<dyn Texture + Sync + Send>, p_scale: f64)
                        -> Self {
        Self {
            noise: Perlin::with_point_scaling(p_scale),
            albedo, scale,
        }
    }
}

impl From<Marble> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: Marble) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for Marble {
    // perlin interpolation can return negative numbers, so we add 1 and divide by 2
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        self.albedo.value(u, v, p) *
            0.5 * (1. +
                   f64::sin(self.scale * p.z() +
                            10.0 * self.noise.turb(p, Some(10))))
        // Color(1.0, 1.0, 1.0) * self.noise.turb(p, None)
    }
}

pub struct Wood {
    noise: Perlin,
    scale: Vec3,
    color: Color,
}

impl Wood {
    #[must_use]
    pub fn new(scale: Vec3, color: Color) -> Self {
        Self {
            noise: Perlin::new(),
            scale, color,
        }
    }
}

impl From<Wood> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: Wood) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for Wood {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        // let ns = self.noise.turb(&(self.scale * *p), None);
        let ns = self.noise.turb(self.scale * Point3(u, v, 0.0), None);
        let c1 = self.color * 0.5 * (1.0 + f64::sin(self.scale.y() + 5.0 * ns));
        let c2 = c1 * 0.5 *
            (1.0 + f64::cos(
                self.scale.x() + 3.0 *
                    self.noise.turb(c1 * self.scale.z(), None)
            ));
        let c3 = self.color * self.noise.smooth_noise(c1 * 50.0);
        (c1 + c2 + c3) / 3.0
    }
}

pub struct Noise {
    noise: Perlin,
    color: Arc<dyn Texture + Sync + Send>,
}

impl Noise {
    #[must_use]
    pub fn new(c: Color) -> Self {
        Self::from_texture(SolidColor::new(c).into())
    }

    #[must_use]
    pub fn from_texture(color: Arc<dyn Texture + Sync + Send>)
                        -> Self {
        Self {
            noise: Perlin::new(),
            color
        }
    }
}

impl From<Noise> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: Noise) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for Noise {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        self.color.value(u, v, p) * (
            1.0 +
                f64::sin(
                    (u + self.noise.smooth_noise(Point3(u, v, 0.0) * 5.0) * 0.5) *
                        50.0
                )
        ) * 0.5
    }
}

pub struct Voronoi {
    vn_points: Vec<(Point3, Color)>,
}

impl Voronoi {
    #[must_use]
    pub fn new(_c: Color, n: u32) -> Self {
        let mut vn_points: Vec<(Point3, Color)> = vec![];
        for _ in 0..n {
            vn_points.push((
                Point3(random::double(), random::double(), 0.0),
                Color::random_range(0.0, 0.8),
            ));
        }
        Self {vn_points}
    }
}

impl From<Voronoi> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: Voronoi) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for Voronoi {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        let pt = Point3(u, v, 0.0);
        let mp = self.vn_points.iter().min_by(|p1, p2| {
            (((*p1).0 - pt).len()).partial_cmp(
                &((*p2).0 - pt).len()).unwrap()
        }).unwrap();

        mp.1
     }
}

pub struct Image {
    img: DynamicImage,
    width: u32,
    height: u32,
}


impl Image {
    const COLOR_SCALE: f64 = 1.0 / 255.0;

    #[must_use]
    pub fn new(fname: &str) -> Self {
        let img = image::open(fname).expect("File not found!");
        eprintln!("{} - dimensions: {:?}; color: {:?}",
                  fname, img.dimensions(), img.color());
        let (width, height) = img.dimensions();

        Self {
            img, width, height,
        }
    }
}

impl From<Image> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: Image) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for Image {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        // Clamp input texture coords to [0,1] x [1,0]
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // flip V to image coords

        let mut i = (u * f64::from(self.width)) as u32;
        let mut j = (v * f64::from(self.height)) as u32;

        i = i.clamp(0, self.width - 1);
        j = j.clamp(0, self.height - 1);

        let pixel = self.img.get_pixel(i, j);

        Color(
            Image::COLOR_SCALE * f64::from(pixel[0]),
            Image::COLOR_SCALE * f64::from(pixel[1]),
            Image::COLOR_SCALE * f64::from(pixel[2])
        )

    }
}

pub struct RandomBump {
    noise: Perlin,
}

impl RandomBump {
    pub fn new() -> Self{
        Self {
            noise: Perlin::new(),
        }
    }
}

impl From<RandomBump> for Arc<dyn FloatTexture + Sync + Send> {
    fn from(tt: RandomBump) -> Arc<dyn FloatTexture + Sync + Send> {
        Arc::new(tt)
    }
}

impl FloatTexture for RandomBump {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> f64 {
        f64::sin(self.noise.turb(4. * p, None))
    }
}

pub struct CheckerBump {
    even: f64,
    odd: f64,
}

impl CheckerBump {
    #[must_use]
    pub fn new(even: f64, odd: f64) -> Self {
        Self {
            even, odd,
        }
    }
}

impl From<CheckerBump> for Arc<dyn FloatTexture + Sync + Send> {
    fn from(tt: CheckerBump) -> Arc<dyn FloatTexture + Sync + Send> {
        Arc::new(tt)
    }
}

impl FloatTexture for CheckerBump {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> f64 {
        const SCALE: f64 = 10.0;
        let sines =
            f64::sin(SCALE * p.x()) * f64::sin(SCALE * p.y()) * f64::sin(SCALE * p.z());
        if sines < 0.0 {
            self.odd
        } else {
            self.even
        }
    }
}


