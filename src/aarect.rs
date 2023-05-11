use crate::vec3::{Point3, Axis, Vec3, dot};
use crate::hit::{Hittable, HitRecord};
use crate::material::Material;
use crate::aabb::AABB;
use crate::ray::Ray;
use crate::util::{random, INFINITY};

use std::sync::Arc;

pub struct AARect {
    p0: Point3, p1: Point3,
    norm: Vec3,
    k_axis: Axis,
    mat: Arc<dyn Material + Sync + Send>,
}

impl AARect {
    fn new(p0: Point3, p1: Point3, k_axis: Axis, mat: Arc<dyn Material + Sync + Send>) -> Self {
        assert!(p0.axis(k_axis) == p1.axis(k_axis));
        AARect {
            p0: p0, p1: p1,
            norm: match k_axis {
                Axis::X => Vec3(1.0, 0.0, 0.0),
                Axis::Y => Vec3(0.0, 1.0, 0.0),
                Axis::Z => Vec3(0.0, 0.0, 1.0),
            },
            k_axis,
            mat: mat,
        }
    }

    pub fn xy_rect(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, mat: Arc<dyn Material + Sync + Send>) -> Self {
        Self::new(
            Point3(x0, y0, k),
            Point3(x1, y1, k),
            Axis::Z,
            mat,
        )
    }
    pub fn xz_rect(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, mat: Arc<dyn Material + Sync + Send>) -> Self {
        Self::new(
            Point3(x0, k, z0),
            Point3(x1, k, z1),
            Axis::Y,
            mat,
        )
    }
    pub fn yz_rect(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, mat: Arc<dyn Material + Sync + Send>) -> Self {
        Self::new(
            Point3(k, y0, z0),
            Point3(k, y1, z1),
            Axis::X,
            mat,
        )
    }

    fn area(&self) -> f64 {
        match self.k_axis {
            Axis::X => (self.p1.y() - self.p0.y()) * (self.p1.z() - self.p0.z()),
            Axis::Y => (self.p1.x() - self.p0.x()) * (self.p1.z() - self.p0.z()),
            Axis::Z => (self.p1.x() - self.p0.x()) * (self.p1.y() - self.p0.y()),
        }
    }

}

impl Hittable for AARect {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let k_axis = self.k_axis;
        let a_axis = match k_axis {
            Axis::X => Axis::Y,
            _ => Axis::X,
        };
        let b_axis = match k_axis {
            Axis::Z => Axis::Y,
            _ => Axis::Z,
        };

        let k  = self.p1.axis(k_axis);
        let a0 = self.p0.axis(a_axis);
        let a1 = self.p1.axis(a_axis);
        let b0 = self.p0.axis(b_axis);
        let b1 = self.p1.axis(b_axis);

        let t = (k - r.origin.axis(k_axis)) / r.dir.axis(k_axis);
        if (t < t_min) || (t > t_max) {
            return None;
        }

        let a = r.origin.axis(a_axis) + t * r.dir.axis(a_axis);
        let b = r.origin.axis(b_axis) + t * r.dir.axis(b_axis);

        if (a < a0)  || (a > a1) || (b < b0) || (b > b1)  {
            return None;
        }

        let u = (a - a0) / (a1 - a0);
        let v = (b - b0) / (b1 - b0);

        let outward_normal = self.norm;

        let p = r.at(t);
        Some(HitRecord::new(
            r, p, outward_normal, t, u, v, self.mat.clone(),
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: self.p0 - self.norm / 10000.,
            max: self.p1 + self.norm / 10000.,
        })
    }

    fn pdf_value(&self, origin: Point3, v: Vec3) -> f64 {
        if let Some(hr) = self.hit(&Ray::new(origin, v, 0.0), 0.001, INFINITY) {
            let area = self.area();
            let distance_squared = hr.t * hr.t * v.len_squared();
            let cosine = f64::abs(dot(v, hr.norm) / v.len());
            let v = distance_squared / (cosine * area);
            v
        } else {
            0.0
        }
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        let random_point = match self.k_axis {
            Axis::X => Point3(
                self.p0.x(),
                random::double_range(self.p0.y(), self.p1.y()),
                random::double_range(self.p0.z(), self.p1.z()),
            ),
            Axis::Y => Point3(
                random::double_range(self.p0.x(), self.p1.x()),
                self.p0.y(),
                random::double_range(self.p0.z(), self.p1.z()),
            ),
            Axis::Z => Point3(
                random::double_range(self.p0.x(), self.p1.x()),
                random::double_range(self.p0.y(), self.p1.y()),
                self.p1.z(),
            ),
        };
        random_point - origin
    }
}
