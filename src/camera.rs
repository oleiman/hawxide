#![allow(dead_code)]

use crate::vec3::{Vec3, Point3};
use crate::vec3;
use crate::ray::Ray;
use crate::util;
use crate::util::random;


pub struct Camera {
    origin : Point3,
    lower_left : Point3,
    horizontal : Vec3,
    vertical : Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    lens_radius: f64,
    time0 : f64,
    time1 : f64,
}

impl Camera {
    pub fn new(lookfrom: &Point3,
               lookat: &Point3,
               vup: &Vec3,
               vfov: f64,
               aspect_ratio: f64,
               aperture: f64,
               focus_dist: f64,
               time0: f64,
               time1: f64,
    ) -> Camera {
        let theta = util::degrees_to_radians(vfov);
        let h = (theta / 2.).tan();

        // we still have a sort of view port sitting between the lens and
        // the focus plane, which is sort of a projection of the viewport,
        // but now we calculate our rays incrementally across the focus plane
        // instead of the viewport
        let view_height = 2.0 * h;
        let view_width = aspect_ratio * (view_height as f64);

        // (u,v,w) forms an orthonormal basis for the lens, viewport,
        // and focus plane, which are parallel
        let w = (lookfrom - lookat).unit_vector();
        let u = (vec3::cross(vup, &w)).unit_vector();
        let v = vec3::cross(&w, &u);

        eprintln!("Camera: from: {}, at: {}", lookfrom, lookat);
        eprintln!("Camera: u: {}, v: {}, w: {}", u, v, w);

        // Origin is now the center of the lens
        let origin = *lookfrom;

        // Actual dimensions of the focus plane
        let horizontal = focus_dist * view_width * u;
        let vertical = focus_dist * view_height * v;
        let lower_left = origin - horizontal / 2 - vertical / 2 - focus_dist * w;

        let lens_radius = aperture / 2.;
        Camera {
            origin,
            lower_left,
            horizontal, vertical,
            u, v, w,
            lens_radius,
            time0, time1,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.u * rd.x() + self.v * rd.y();
        // generate randomly timed rays out into the scene
        Ray {
            origin: self.origin + offset,
            dir: self.lower_left +
                (s * self.horizontal) +
                (t * self.vertical) -
                self.origin - offset,
            time: random::double_in_range(self.time0, self.time1),
        }
    }
}

