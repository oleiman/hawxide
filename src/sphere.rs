use crate::vec3::{Vec3,Point3,dot};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::util::{PI,INFINITY};
use crate::onb::OrthoNormalBasis;

use std::sync::Arc;

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat: Arc<dyn Material + Sync + Send>,
    phi_max: f64,
    theta_min: f64,
    theta_max: f64,
}

impl Sphere {
    #[must_use]
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Material + Sync + Send>)
               -> Sphere {
        Self {
            center,
            radius,
            mat,
            phi_max: 2.0 * PI,
            theta_min: 0.0,
            theta_max: PI,

        }
    }

    // TODO(oren): these are calculated on the normal vector, need to go back to
    // the book and recall why that is
    fn get_sphere_uv(&self, p: Point3) -> (f64, f64, Vec3, Vec3) {
        // p: given a point on the unit sphere centered at the origin
        // return (u,v) s.t.
        //   u in [0,1]: angle around the Y axis from X=-1 (as a fraction of 2*pi)
        //   v in [0,1]: angle from Y=-1 to Y=+1 (as fraction of pi)

        let theta = f64::acos(-p.y());
        let phi = f64::atan2(-p.z(), p.x()) + PI;
        let u = phi / self.phi_max; //(2.0 * PI);
        let v = theta / (self.theta_max - self.theta_min); //PI;

        // Source: https://www.pbr-book.org/3ed-2018/Shapes/Spheres
        // let z_radius = f64::sqrt(p.x() * p.x() + p.y() * p.y());
        // assert!(z_radius == 1.0, "z radius: {:3}", z_radius);
        // let inv_z_radius = 1.0 / z_radius;
        // NOTE(oren): already ostensibly on the unit sphere, so no need to
        // normalize for sin/cos
        let cos_phi = p.x();
        let sin_phi = -p.z();

        let dpdu = Vec3(self.phi_max * p.z(), self.phi_max * p.x(), 0.0);
        let dpdv = (self.theta_max - self.theta_min) * Vec3(
            -p.y() * cos_phi,
            -p.y() * sin_phi,
            -self.radius * f64::sin(theta)
        );

        (
            u, v,
            dpdu, dpdv,
        )
    }

}

impl From<Sphere> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Sphere) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

impl Hittable for Sphere {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        // vector in the direction from sphere center to the ray origin
        let oc : Vec3 = r.origin - self.center;

        // use the quadratic formula to determine whether the Ray intersects
        // this Sphere surface for some value of `t`. i.e. r.at(t) lies on the 
        // sphere can check back in the book for the algebra that yields the
        // following parameters. See Weekend book for the algebra.

        let a : f64 = r.dir.len_squared();
        let half_b : f64 = dot(oc, r.dir);
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
            let (u,v,dpdu,dpdv) = self.get_sphere_uv(outward_norm);
            Some(HitRecord::with_dps(
                r, p, outward_norm, r1, u, v, self.mat.clone(), dpdu, dpdv)
            )
        } else if t_min <= r2 && r2 <= t_max {
            let p : Point3 = r.at(r2);
            let outward_norm : Vec3 = (p - self.center) / self.radius;
            let (u,v,dpdu,dpdv) = self.get_sphere_uv(outward_norm);
            Some(HitRecord::with_dps(
                r, p, outward_norm, r2, u, v, self.mat.clone(), dpdu, dpdv)
            )
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

    fn pdf_value(&self, origin: Point3, v: Vec3) -> f64 {
        if self.hit(&Ray::new(origin, v, 0.0), 0.001, INFINITY).is_none() {
            return 0.0;
        }
        let cos_theta_max = f64::sqrt(
            1.0 - self.radius * self.radius / (self.center - origin).len_squared()
        );
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);
        1.0 / solid_angle
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        let direction = self.center - origin;
        let dist_squared = direction.len_squared();
        let mut uvw = OrthoNormalBasis::new();
        uvw.build_from_w(direction);
        uvw.local_v(Vec3::random_to_sphere(self.radius, dist_squared))
    }
}
