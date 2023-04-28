use crate::vec3::{Point3, Dimension, Vec3};
use crate::hit::{Hittable, HitRecord};
use crate::material::Material;
use crate::aabb::AABB;
use crate::ray::Ray;

use std::rc::Rc;

pub struct AARect {
    p0: Point3, p1: Point3,
    norm: Vec3,
    k_dim: Dimension,
    mat: Rc<dyn Material>,
}

impl AARect {
    pub fn new(p0: &Point3, p1: &Point3, k_dim: Dimension, mat: &Rc<dyn Material>) -> Self {
        assert!(p0.dim(&k_dim) == p1.dim(&k_dim));
        AARect {
            p0: *p0, p1: *p1,
            norm: match k_dim {
                Dimension::X => Vec3(1.0, 0.0, 0.0),
                Dimension::Y => Vec3(0.0, 1.0, 0.0),
                Dimension::Z => Vec3(0.0, 0.0, 1.0),
            },
            k_dim,
            mat: mat.clone(),
        }
    }

    pub fn xy_rect(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, mat: &Rc<dyn Material>) -> Self {
        Self::new(
            &Point3(x0, y0, k),
            &Point3(x1, y1, k),
            Dimension::Z,
            mat,
        )
    }
    pub fn xz_rect(x0: f64, x1: f64, z0: f64, z1: f64, k: f64, mat: &Rc<dyn Material>) -> Self {
        Self::new(
            &Point3(x0, k, z0),
            &Point3(x1, k, z1),
            Dimension::Y,
            mat,
        )
    }
    pub fn yz_rect(y0: f64, y1: f64, z0: f64, z1: f64, k: f64, mat: &Rc<dyn Material>) -> Self {
        Self::new(
            &Point3(k, y0, z0),
            &Point3(k, y1, z1),
            Dimension::X,
            mat,
        )
    }

}

impl Hittable for AARect {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let k_dim = &self.k_dim;
        let a_dim = &match k_dim {
            Dimension::X => Dimension::Y,
            _ => Dimension::X,
        };
        let b_dim = &match k_dim {
            Dimension::Z => Dimension::Y,
            _ => Dimension::Z,
        };

        let k  = self.p1.dim(k_dim);
        let a0 = self.p0.dim(a_dim);
        let a1 = self.p1.dim(a_dim);
        let b0 = self.p0.dim(b_dim);
        let b1 = self.p1.dim(b_dim);

        let t = (k - r.origin.dim(k_dim)) / r.dir.dim(k_dim);
        if t < t_min || t > t_max {
            return None;
        }

        let a = r.origin.dim(a_dim) + t * r.dir.dim(a_dim);
        let b = r.origin.dim(b_dim) + t * r.dir.dim(b_dim);
        if a < a0 || a > a1 || b < b0 || b > b1 {
            return None;
        }

        let u = (a - a0) / (a1 - a0);
        let v = (b - b0) / (b1 - b0);

        let outward_normal = self.norm;

        let p = r.at(t);
        Some(HitRecord::new(
            r, &p, &outward_normal, t, u, v, &self.mat,
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: self.p0 - self.norm / 10000.,
            max: self.p1 + self.norm / 10000.,
        })
    }
}

pub struct XYRect {
    x0: f64, x1: f64, y0: f64, y1: f64, k: f64,
    mat: Rc <dyn Material>,
}

impl XYRect {
    pub fn new(x0: f64, x1: f64, y0: f64, y1: f64, k: f64, mat: &Rc<dyn Material>) -> XYRect {
        XYRect {
            x0, x1, y0, y1, k,
            mat: mat.clone(),
        }
    }
}

impl Hittable for XYRect {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let t = (self.k - r.origin.z()) / r.dir.z();
        if t < t_min || t > t_max {
            return None;
        }

        let x = r.origin.x() + t * r.dir.x();
        let y = r.origin.y() + t * r.dir.y();

        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        let u = (x - self.x0) / (self.x1 - self.x0);
        let v = (y - self.y0) / (self.y1 - self.y0);
        let outward_normal = Vec3(0.0, 0.0, 1.0);
        Some(HitRecord::new(
            r, &r.at(t), &outward_normal, t, u, v, &(self.mat.clone()),
        ))
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: Point3(self.x0, self.y0, self.k - 0.0001),
            max: Point3(self.x1, self.y1, self.k + 0.0001),
        })
    }
}
