use crate::vec3::{Vec3,Point3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::util::PI;

use std::sync::Arc;

pub struct Cylinder {
    radius: f64,
    y_min: f64,
    y_max: f64,
    phi_max: f64,
    mat: Arc<dyn Material + Sync + Send>
}

impl Cylinder {
    pub fn new(radius: f64, y_min: f64, y_max: f64,
               mat: Arc<dyn Material + Sync + Send>) -> Self {
        Self {
            radius, y_min, y_max, mat,
            phi_max: 2.0 * PI,
        }
    }
}

impl Hittable for Cylinder {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let a: f64 = r.dir.x() * r.dir.x() + r.dir.z() * r.dir.z();
        let half_b: f64 = r.dir.x() * r.origin.x() + r.dir.z() * r.origin.z();
        let c: f64 =
            r.origin.x() * r.origin.x() +
            r.origin.z() * r.origin.z() -
            self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        let sqrtd = discriminant.sqrt();

        if sqrtd.is_nan() {
            return None;
        }

        let (r1, r2) = (
            (-half_b - sqrtd) / a,
            (-half_b + sqrtd) / a,
        );

        let mut t_hit = if t_min <= r1 && r1 <= t_max {
            Some(r1)
        } else if t_min <= r2 && r2 <= t_max {
            Some(r2)
        } else {
            None
        }?;

        let p = r.at(t_hit);

        let p = if p.y() < self.y_min || p.y() > self.y_max {
            if t_hit == r2 || r2 < t_min || r2 > t_max {
                None
            } else {
                t_hit = r2;
                let p_hit = r.at(t_hit);
                if p_hit.y() < self.y_min || p_hit.y() > self.y_max {
                    None
                } else {
                    Some(p_hit)
                }
            }
        } else {
            Some(p)
        }?;

        let outward_norm = (p - Point3(0.0, p.y(), 0.0)).unit_vector();
        let phi = f64::atan2(p.z(), p.x()) + PI;
        if phi > self.phi_max {
            return None;
        }

        let u = phi / self.phi_max;
        let v = (p.y() - self.y_min) / (self.y_max - self.y_min);
        let dpdu = Vec3(-self.phi_max * p.z(), 0.0, self.phi_max * p.x());
        let dpdv = Vec3(0.0, self.y_max - self.y_min, 0.0);


        Some(HitRecord::with_dps(
            r, p, outward_norm, t_hit, u, v, self.mat.clone(), dpdu, dpdv
        ))

    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: Point3(-self.radius, self.y_min, -self.radius),
            max: Point3(self.radius, self.y_max, self.radius),
        })
    }

}

impl From<Cylinder> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Cylinder) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}
