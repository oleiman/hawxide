use crate::vec3::{Vec3, Point3, dot};
use crate::ray::Ray;
use crate::hit::{HitRecord, Hittable};
use crate::material::Material;
use crate::aabb::AABB;

use std::rc::Rc;

pub struct MovingSphere {
    pub center0: Point3,
    pub center1: Point3,
    pub time0: f64,
    pub time1: f64,
    pub radius: f64,
    pub mat: Rc<dyn Material>,
}

impl MovingSphere {
    pub fn new(center0: &Point3, center1: &Point3,
               time0: f64, time1: f64,
               radius: f64, mat: &Rc<dyn Material>) -> MovingSphere {
        // Moving sphere has two centers. One where it starts (at time0) and another
        // where it ends up (at time1).
        MovingSphere {
            center0: *center0,
            center1: *center1,
            time0,
            time1,
            radius,
            mat: mat.clone(),
        }
    }

    pub fn center(&self, time: f64) -> Point3 {
        // Translate center0 along the vector from to center1,
        // scaled by current time as proportion of time1 - time0.
        self.center0 +
            ((time - self.time0) / (self.time1 - self.time0)) *
            (self.center1 - self.center0)
    }
}

impl Hittable for MovingSphere {
    // Basically the same as a regular Sphere, but we _calculate_ the location
    // of the MovingSphere's center based on the time associated with the ray.
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc : Vec3 = r.origin - self.center(r.time);

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
            let outward_norm : Vec3 = (p - self.center(r.time)) / self.radius;
            Some(HitRecord::new(r, &p, &outward_norm, r1, 0.0, 0.0, &self.mat))
        } else if t_min <= r2 && r2 <= t_max {
            let p : Point3 = r.at(r2);
            let outward_norm : Vec3 = (p - self.center(r.time)) / self.radius;
            Some(HitRecord::new(r, &p, &outward_norm, r2, 0., 0., &self.mat))
        } else {
            None
        }
    }

    // Take the box for the sphere's initial position and the box for the sphere's
    // final position, and compute a bounding box around those. The MovingSphere is
    // then guaranteed to reside in that BB at any given time.
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        let box0 = AABB {
            min: self.center(time0) - Vec3(self.radius, self.radius, self.radius),
            max: self.center(time0) + Vec3(self.radius, self.radius, self.radius),
        };
        let box1 = AABB {
            min: self.center(time1) - Vec3(self.radius, self.radius, self.radius),
            max: self.center(time1) + Vec3(self.radius, self.radius, self.radius),
        };
        Some(AABB::surrounding_box(box0, box1))
    }
}
