use crate::vec3::{Vec3,Point3,dot};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};


pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // vector in the direction from sphere center to the ray origin
        let oc : Vec3 = r.origin - self.center;

        // use the quadratic formula to determine whether the Ray intersects
        // this Sphere for some value of `t`. i.e. r.at(t) lies on the sphere
        // can check back in the book for the algebra that yields the following
        // parameters.

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

        // if t_range.contains(&r1) {
        if t_min <= r1 && r1 <= t_max {
            let p : Point3 = r.at(r1);
            let outward_norm : Vec3 = (p - self.center) / self.radius;
            Some(HitRecord::new(r, &p, &outward_norm, r1))
        } else if t_min <= r2 && r2 <= t_max {
            let p : Point3 = r.at(r2);
            let outward_norm : Vec3 = (p - self.center) / self.radius;
            Some(HitRecord::new(r, &p, &outward_norm, r2))
        } else {
            None
        }
    }
}
