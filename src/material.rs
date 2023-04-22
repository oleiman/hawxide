use crate::ray::Ray;
use crate::hit::HitRecord;
use crate::vec3::{Color, Vec3};
use crate::vec3;
use crate::util::random;


pub trait Material {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<(Color,Ray)>;
}

pub struct Lambertian {
    pub albedo: Color,
}

impl Material for Lambertian {
    #[allow(unused_variables)]
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<(Color,Ray)> {
        let mut scatter_direction = rec.norm + Vec3::random_unit_vector();
        if scatter_direction.near_zero() {
            scatter_direction = rec.norm;
        }
        Some((self.albedo,
              Ray{
                  origin: rec.p,
                  dir: scatter_direction,
                  time: ray_in.time,}
        ))
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<(Color,Ray)> {
        let reflected = vec3::reflect(&ray_in.dir.unit_vector(), &rec.norm);
        let f = match self.fuzz {
            f if f <= 1.0 => f,
            _ => 1.0,
        };
        let scattered = Ray {
            origin: rec.p,
            dir: reflected + f * Vec3::random_in_unit_sphere(),
            time: ray_in.time,
        };
        if vec3::dot(&scattered.dir, &rec.norm) <= 0.0 {
            return None;
        }

        Some((self.albedo, scattered))
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
               -> Option<(Color,Ray)> {
        let attenuation = Color(1., 1., 1.);
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

        if cannot_refract {
            Some((attenuation,
                  Ray{
                      origin: rec.p,
                      dir: vec3::reflect(&unit_direction, &rec.norm),
                      time: ray_in.time,
                  }
            ))
        } else {
            Some((attenuation,
                  Ray{
                      origin: rec.p,
                      dir: vec3::refract(&unit_direction, &rec.norm, refraction_ratio),
                      time: ray_in.time,
                  }
            ))
        }
    }
}
