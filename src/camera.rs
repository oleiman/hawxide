use crate::vec3::{Vec3, Point3};
use crate::vec3;
use crate::ray::Ray;
use crate::util;


pub struct Camera {
    origin : Point3,
    lower_left : Point3,
    horizontal : Vec3,
    vertical : Vec3,
}

impl Camera {
    pub fn new(lookfrom: &Point3,
               lookat: &Point3,
               vup: &Vec3,
               vfov: f64,
               aspect_ratio: f64// ,
               // aperture: f64,
               // focus_dist: f64
    ) -> Camera {
        let theta = util::degrees_to_radians(vfov);
        let h = (theta / 2.).tan();
        let view_height = 2.0 * h;
        let view_width = aspect_ratio * (view_height as f64);

        let w = (lookfrom - lookat).unit_vector();
        let u = (vec3::cross(vup, &w)).unit_vector();
        let v = vec3::cross(&w, &u);

        let origin = *lookfrom;
        let horizontal = view_width * u;
        let vertical = view_height * v;
        Camera {
            origin: origin,
            lower_left: origin - horizontal / 2 - vertical / 2 - w,
            horizontal: horizontal,
            vertical: vertical,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.lower_left + (s * self.horizontal) + (t * self.vertical) - self.origin,
        }
    }
}

