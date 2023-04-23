use crate::vec3::{Vec3, Point3, dot};
use crate::ray::Ray;
use crate::material::Material;
use crate::aabb::AABB;

use std::rc::Rc;

pub struct HitRecord {
    pub p: Point3,
    pub norm: Vec3,
    pub mat: Rc<dyn Material>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(r: &Ray, p: &Point3, out_norm: &Vec3,
               t: f64, u: f64, v: f64, mat: &Rc<dyn Material>) -> HitRecord {
        // out_norm always points outward from the hittable object
        // instead, we want our hit record norm to point against the
        // ray, thereby telling us whether the ray is inside or outside
        // the object. So if the dot product is < 0, then the angle
        // between ray and surface normal is > 90deg, so we note that
        // the ray intersects the outer face of the surface and leave
        // the normal alone. Otherwise the ray is inside the surface,
        // so we note that and reverse the direction of the normal
        let front_face : bool  = dot(&r.dir, out_norm) < 0.;
        HitRecord {
            p: *p,
            norm: if front_face {
                *out_norm
            } else {
                -out_norm
            },
            mat: Rc::clone(mat),
            t, u, v,
            front_face,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, out_norm: &Vec3) {

        self.front_face = dot(&r.dir, out_norm) < 0.;
        self.norm = if self.front_face {
            *out_norm
        } else {
            -out_norm
        };
    }
}

pub trait Hittable {
    // Does the ray hit the Hittable? Return a record of the hit.
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;

    // Give the smallest reasonable AABB for the Hittable
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB>;
}
