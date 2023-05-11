use crate::vec3::{Point3,Color,Vec3};
use crate::util::random;
use crate::perlin::Perlin;

use std::sync::Arc;
use image;
use image::{GenericImageView,DynamicImage};

pub trait Texture {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color;
}

pub struct SolidColor {
    color_val: Color,
}

impl SolidColor {
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

pub struct CheckerTexture {
    even: Arc<dyn Texture + Sync + Send>,
    odd: Arc<dyn Texture + Sync + Send>,
}

impl CheckerTexture {
    pub fn new(c1: Color, c2: Color) -> Self {
        Self {
            even: SolidColor::new(c1).into(),
            odd: SolidColor::new(c2).into(),
        }
    }
}

impl From<CheckerTexture> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: CheckerTexture) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for CheckerTexture {
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

pub struct MarbleTexture {
    noise: Perlin,
    albedo: Arc<dyn Texture + Sync + Send>,
    scale: f64,
}

impl MarbleTexture {
    pub fn new(scale : f64) -> Self {
        Self::from_texture(
            scale,
            SolidColor::new(Color(1.0, 1.0, 1.0)).into()
        )
    }
    pub fn from_texture(scale: f64, albedo: Arc<dyn Texture + Sync + Send>)
                        -> Self {
        Self {
            noise: Perlin::new(),
            albedo, scale,
        }
    }
}

impl From<MarbleTexture> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: MarbleTexture) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for MarbleTexture {
    // perlin interpolation can return negative numbers, so we add 1 and divide by 2
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        self.albedo.value(u, v, p) *
            0.5 * (1. +
                   f64::sin(self.scale * p.z() +
                            10. * self.noise.turb(p, None)))
    }
}

pub struct WoodTexture {
    noise: Perlin,
    scale: Vec3,
    color: Color,
}

impl WoodTexture {
    pub fn new(scale: Vec3, color: Color) -> Self {
        Self {
            noise: Perlin::new(),
            scale, color,
        }
    }
}

impl From<WoodTexture> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: WoodTexture) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for WoodTexture {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        // let ns = self.noise.turb(&(self.scale * *p), None);
        let ns = self.noise.turb(self.scale * Point3(_u, _v, 0.0), None);
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

pub struct NoiseTexture {
    noise: Perlin,
    color: Arc<dyn Texture + Sync + Send>,
}

impl NoiseTexture {
    pub fn new(c: Color) -> Self {
        Self::from_texture(SolidColor::new(c).into())
    }

    pub fn from_texture(color: Arc<dyn Texture + Sync + Send>)
                        -> Self {
        Self {
            noise: Perlin::new(),
            color
        }
    }
}

impl From<NoiseTexture> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: NoiseTexture) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for NoiseTexture {
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

pub struct VoronoiTexture {
    vn_points: Vec<(Point3, Color)>,
}

impl VoronoiTexture {
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

impl From<VoronoiTexture> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: VoronoiTexture) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for VoronoiTexture {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        let pt = Point3(u, v, 0.0);
        let mp = self.vn_points.iter().min_by(|p1, p2| {
            (((*p1).0 - pt).len()).partial_cmp(
                &((*p2).0 - pt).len()).unwrap()
        }).unwrap();

        mp.1
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

        Self {
            img, width, height,
        }
    }
}

impl From<ImageTexture> for Arc<dyn Texture + Sync + Send> {
    fn from(tt: ImageTexture) -> Arc<dyn Texture + Sync + Send> {
        Arc::new(tt)
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
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
