use crate::vec3::{Vec3,Point3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::util::PI;

use std::sync::Arc;

pub struct Disk {
    radius: f64,
    inner_radius: f64,
    height: f64,
    phi_max: f64,
    norm: Vec3,
    mat: Arc<dyn Material + Sync + Send>,
}

impl Disk {
    pub fn new(radius: f64, inner_radius: f64, height: f64, mat: Arc<dyn Material + Sync + Send>) -> Self {
        Self {
            radius, inner_radius, height, mat,
            norm: Vec3(0.0, 1.0, 0.0),
            phi_max: 2.0 * PI,
        }
    }
}

impl Hittable for Disk {
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if r.dir.y() == 0.0 {
            return None;
        }

        // compute disk plane intersection
        let t_hit = (self.height - r.origin.y()) / r.dir.y();

        let p = if t_min <= t_hit && t_hit <= t_max {
            Some(r.at(t_hit))
        } else {
            None
        }?;

        let dist2 = p.x() * p.x() + p.z() * p.z();
        if dist2 > self.radius * self.radius ||
            dist2 < self.inner_radius * self.inner_radius {
                return None;
        }
        let phi = f64::atan2(p.z(), p.x()) + PI;

        if phi > self.phi_max {
            return None;
        }

        let u = phi / self.phi_max;
        let r_hit = dist2.sqrt();
        let one_less_v = (r_hit - self.inner_radius) / (self.radius - self.inner_radius);
        let v = 1.0 - one_less_v;

        let dpdu = Vec3(-self.phi_max * p.z(), 0.0, self.phi_max * p.x());
        let dpdv =
            Vec3(p.z(), 0.0, p.y()) * (self.inner_radius - self.radius) / r_hit;

        Some(HitRecord::with_dps(
            r, p, self.norm, t_hit, u, v, self.mat.clone(), dpdu, dpdv,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: Point3(-self.radius, self.height - 0.000_001, -self.radius),
            max: Point3(self.radius, self.height + 0.000_001, self.radius),
        })
    }
}

impl From<Disk> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Disk) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}
