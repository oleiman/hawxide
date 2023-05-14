use crate::vec3::{Vec3,Point3,dot,cross};
use crate::ray::Ray;
use crate::hit::{Hittable,HitRecord};
use crate::material::Material;
use crate::aabb::AABB;

use std::sync::Arc;

pub struct Triangle {
    pub v0: Point3,
    pub v1: Point3,
    pub v2: Point3,
    pub norm: Vec3,
    pub mat: Arc<dyn Material + Sync + Send>
}

impl Triangle {
    #[must_use]
    pub fn new(v0: Point3, v1: Point3, v2: Point3, mat: Arc<dyn Material + Sync + Send>)
        -> Self {
        // TODO(oren): it would be nice to not store the normal, but I need it for other
        // stuff, right?
        Self {
            v0, v1, v2, norm: cross(v1 - v0, v2 - v0).unit_vector(), mat,
        }
    }

    #[must_use]
    pub fn bary_to_cart(&self, u: f64, v: f64) -> Point3 {
        (1.0 - u - v)*self.v0 + u * self.v1 + v * self.v2
    }
}

impl From<Triangle> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Triangle) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

impl Hittable for Triangle {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let e1 = self.v1 - self.v0;
        let e2 = self.v2 - self.v0;
        let t_vec = r.origin - self.v0;
        // let d_norm = r.dir.unit_vector();
        let d_norm = r.dir;

        let p_vec = cross(d_norm, e2);
        let q_vec = cross(t_vec, e1);

        let det = dot(p_vec, e1);

        // if we want to do culling, we would discard intersections on one side
        // i.e. discard if determinant is l.t. epsilon
        if det.abs() < 0.000_001 {
            return None;
        }

        let Vec3(t_hit, u, v) =
            (1.0 / det) *
            Vec3(dot(q_vec, e2), dot(p_vec, t_vec), dot(q_vec, r.dir));

        if u < 0.0 || v < 0.0 || u > 1.0 || u + v > 1.0 {
            return None;
        } else if t_hit < t_min || t_hit > t_max {
            return None;
        }

        let p_hit = self.bary_to_cart(u, v);

        Some(HitRecord::new(
            r, p_hit, self.norm, t_hit, u, v, self.mat.clone()
        ))

    }

    // Give the smallest reasonable AABB for the Hittable
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        let min_x = f64::min(f64::min(self.v0.x(), self.v1.x()), self.v2.x());
        let min_y = f64::min(f64::min(self.v0.y(), self.v1.y()), self.v2.y());
        let min_z = f64::min(f64::min(self.v0.z(), self.v1.z()), self.v2.z());

        let max_x = f64::max(f64::max(self.v0.x(), self.v1.x()), self.v2.x());
        let max_y = f64::max(f64::max(self.v0.y(), self.v1.y()), self.v2.y());
        let max_z = f64::max(f64::max(self.v0.z(), self.v1.z()), self.v2.z());

        Some(AABB {
            min: Point3(min_x, min_y, min_z),
            max: Point3(max_x, max_y, max_z)
        })
    }

    // TODO(oren): pdf stuff...not needed unless we want to sample toward
    // a triangle, which I don't particularly care about right now
}
