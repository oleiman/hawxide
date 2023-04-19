use crate::vec3::{Vec3, Point3};
use crate::ray::Ray;


pub struct Camera {
    origin : Point3,
    lower_left : Point3,
    horizontal : Vec3,
    vertical : Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64, view_height: f64, focal_length: f64) -> Camera {
        let view_width = aspect_ratio * (view_height as f64);
        let origin = Point3(0., 0., 0.);
        let horizontal = Vec3(view_width, 0., 0.);
        let vertical = Vec3(0., view_height, 0.);
        Camera {
            origin: Point3(0.,0.,0.),
            lower_left: origin - horizontal / 2 - vertical / 2 - Vec3(0.,0.,focal_length),
            horizontal: horizontal,
            vertical: vertical,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        Ray {
            origin: self.origin,
            dir: self.lower_left + u * self.horizontal + v * self.vertical - self.origin,
        }
    }
}

