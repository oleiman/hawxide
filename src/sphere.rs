use crate::vec3::{Vec3,Point3,dot};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::util::PI;

use std::rc::Rc;

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(center: &Point3, radius: f64, mat: &Rc<dyn Material>) -> Sphere {
        Sphere {
            center: *center,
            radius,
            mat: mat.clone(),
        }
    }

    fn get_sphere_uv(&self, p: &Point3) -> (f64, f64) {
        // p: given a point on the unit sphere centered at the origin
        // return (u,v) s.t.
        //   u in [0,1]: angle around the Y axis from X=-1 (as a fraction of 2*pi)
        //   v in [0,1]: angle from Y=-1 to Y=+1 (as fraction of pi)

        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + PI;

        (
            phi / (2.0 * PI),
            theta / PI,
        )
    }
}

impl Hittable for Sphere {
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // vector in the direction from sphere center to the ray origin
        let oc : Vec3 = r.origin - self.center;

        // use the quadratic formula to determine whether the Ray intersects
        // this Sphere surface for some value of `t`. i.e. r.at(t) lies on the 
        // sphere can check back in the book for the algebra that yields the
        // following parameters. See Weekend book for the algebra.

        let a : f64 = r.dir.len_squared();
        let half_b : f64 = dot(&oc, &r.dir);
        let c : f64 = oc.len_squared() - self.radius * self.radius;

        let discriminant : f64 = half_b * half_b - a * c;

        let sqrtd = discriminant.sqrt();

        if sqrtd.is_nan() {
            return None;
        }

        let r1 : f64 = (-half_b - sqrtd) / a;
        let r2 : f64 = (-half_b + sqrtd) / a;

        if t_min <= r1 && r1 <= t_max {
            let p : Point3 = r.at(r1);
            let outward_norm : Vec3 = (p - self.center) / self.radius;
            let (u,v) = self.get_sphere_uv(&p);
            Some(HitRecord::new(r, &p, &outward_norm, r1, u, v, &self.mat))
        } else if t_min <= r2 && r2 <= t_max {
            let p : Point3 = r.at(r2);
            let outward_norm : Vec3 = (p - self.center) / self.radius;
            let (u,v) = self.get_sphere_uv(&p);
            Some(HitRecord::new(r, &p, &outward_norm, r2, u, v, &self.mat))
        } else {
            None
        }
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: self.center - Vec3(self.radius, self.radius, self.radius),
            max: self.center + Vec3(self.radius, self.radius, self.radius),
        })
    }
}
