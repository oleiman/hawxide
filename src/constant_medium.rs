use crate::vec3::{Vec3,Color};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::texture::Texture;
use crate::material::{Material,Isotropic};
use crate::aabb::AABB;
use crate::util::{random, INFINITY, NEG_INFINITY};

use std::sync::Arc;

pub struct ConstantMedium {
    boundary: Arc<dyn Hittable + Sync + Send>,
    neg_inv_density: f64,
    phase_fn: Arc<dyn Material + Sync + Send>,
}

impl ConstantMedium {
    pub fn new(boundary: Arc<dyn Hittable + Sync + Send>, d: f64, c: &Color) -> Self {
        Self {
            boundary: boundary,
            neg_inv_density: -1.0 / d,
            phase_fn: Arc::new(Isotropic::new(c)),

        }
    }

    pub fn from_texture(boundary: Arc<dyn Hittable + Sync + Send>, d: f64, a: Arc<dyn Texture + Sync + Send>) -> Self {
        Self {
            boundary: boundary,
            neg_inv_density: -1.0 / d,
            phase_fn: Arc::new(Isotropic { albedo: a }),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        const DEBUG : bool = false;
        let debugging : bool = DEBUG && random::double() < 0.00001;

        let mut hr1 = if let Some(hr) = self.boundary.hit(r, NEG_INFINITY, INFINITY) {
            hr
        } else {
            return None;
        };

        let mut hr2 = if let Some(hr) = self.boundary.hit(r, hr1.t + 0.0001, INFINITY) {
            hr
        } else {
            return None;
        };

        if debugging {
            eprintln!("t_min={}, t_max={}", hr1.t, hr2.t);
        }

        hr1.t = f64::max(hr1.t, t_min);
        hr2.t = f64::min(hr2.t, t_max);

        if hr1.t >= hr2.t {
            return None;
        }

        hr1.t = f64::max(hr1.t, 0.0);

        let ray_length = r.dir.len();
        let distance_inside_boundary = (hr2.t - hr1.t) * ray_length;
        let hit_distance = self.neg_inv_density * f64::ln(random::double());

        if hit_distance > distance_inside_boundary {
            return None;
        }

        let t = hr1.t + hit_distance / ray_length;
        let p = r.at(t);

        if debugging {
            eprintln!("hit_distance = {}\nrec.t = {}\nrec.p = {}", hit_distance, t, p);
        }

        Some(HitRecord {
            p,
            norm: Vec3(1.0, 0.0, 0.0), // arbitrary
            mat: self.phase_fn.clone(),
            t,
            u: 0.0, v: 0.0, // arbitrary?
            front_face: true // arbitrary
        })



    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.boundary.bounding_box(time0, time1)
    }
}
