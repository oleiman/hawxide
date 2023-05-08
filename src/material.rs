use crate::ray::Ray;
use crate::hit::HitRecord;
use crate::vec3::{Color, Vec3, Point3};
use crate::vec3;
use crate::util::random;
use crate::texture::{Texture,SolidColor};
use crate::util::PI;
use crate::pdf::{PDensityFn,CosPDF,HittablePDF,NullPDF};

use std::sync::Arc;

pub struct ScatterRecord {
    pub specular_ray: Option<Ray>,
    pub attenuation: Color,
    pub pdf: Arc<dyn PDensityFn + Sync + Send>,
}

pub trait Material {
    fn scatter(&self, _ray_in: &Ray, _rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }
    fn scattering_pdf(&self, _ray_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        return 0.0;
    }
    fn emitted(&self, _ray_in: &Ray, _rec: &HitRecord,
               _u: f64, _v: f64, _p: &Point3) -> Color {
        Color(0.0, 0.0, 0.0)
    }
}

pub struct Lambertian {
    pub albedo: Arc<dyn Texture + Sync + Send>,
}

impl Lambertian {
    pub fn new(c: &Color) -> Lambertian {
        Lambertian {
            albedo: Arc::new(SolidColor::new(c.r(), c.g(), c.b())),
        }
    }
}

impl Material for Lambertian {
    #[allow(unused_variables)]
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<ScatterRecord> {
        Some(ScatterRecord{
            specular_ray: None,
            attenuation: self.albedo.value(rec.u, rec.v, &rec.p),
            pdf: Arc::new(CosPDF::new(&rec.norm)),
        })
    }

    fn scattering_pdf(&self, _ray_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = vec3::dot(&rec.norm, &scattered.dir.unit_vector());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }
}

pub struct Metal {
    pub albedo: Arc<dyn Texture + Sync + Send>,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(c: &Color, fuzz: f64) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(c.r(), c.g(), c.b())),
            fuzz,
        }
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<ScatterRecord> {
        let reflected = vec3::reflect(&ray_in.dir.unit_vector(), &rec.norm);
        let f = match self.fuzz {
            f if f <= 1.0 => f,
            _ => 1.0,
        };

        let dir = reflected + f * Vec3::random_in_unit_sphere();
        if vec3::dot(&dir, &rec.norm) <= 0.0 {
            return None;
        }

        Some(ScatterRecord{
            specular_ray: Some(Ray::new(
                &rec.p, &(dir), ray_in.time)
            ),
            attenuation: self.albedo.value(rec.u, rec.v, &rec.p),
            pdf: Arc::new(NullPDF::new()),
        })
    }
}

pub struct Dielectric {
    pub ir : f64,
}

impl Dielectric {
    fn reflectance(cos: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflectance
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        r0 + (1. - r0) * (1. - cos).powf(5.)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<ScatterRecord> {
        let refraction_ratio : f64 = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let unit_direction = ray_in.dir.unit_vector();
        let cos_theta = vec3::dot(&-unit_direction, &rec.norm).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract: bool =
            (refraction_ratio * sin_theta > 1.0) ||
            Dielectric::reflectance(cos_theta, refraction_ratio) > random::double();

        let direction = if cannot_refract {
            vec3::reflect(&unit_direction, &rec.norm)
        } else {
            vec3::refract(&unit_direction, &rec.norm, refraction_ratio)
        };

        Some(ScatterRecord {
            specular_ray: Some(Ray::new(
                &rec.p,
                &direction,
                ray_in.time
            )),
            attenuation: Color(1.0, 1.0, 1.0),
            pdf: Arc::new(NullPDF),
        })
    }
}

pub struct DiffuseLight {
    pub emit: Arc<dyn Texture + Sync + Send>,
}

impl DiffuseLight {
    pub fn new(c: &Color) -> DiffuseLight {
        DiffuseLight {
            emit: Arc::new(SolidColor::new(c.r(), c.g(), c.b())),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, _ray_in: &Ray, rec: &HitRecord,
               u: f64, v: f64, p: &Point3) -> Color {
        if rec.front_face {
        self.emit.value(u, v, p)
        } else {
            Color(0.0, 0.0, 0.0)
        }
    }
}

pub struct Isotropic {
    pub albedo: Arc<dyn Texture + Sync + Send>,
}

impl Isotropic {
    pub fn new(c: &Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(c.r(), c.g(), c.b())),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord{
            specular_ray: Some(Ray::new(
                &rec.p, &Vec3::random_in_unit_sphere(), ray_in.time,
            )),
            attenuation: self.albedo.value(rec.u, rec.v, &rec.p),
            pdf: Arc::new(NullPDF::new()),
        })
    }
}
