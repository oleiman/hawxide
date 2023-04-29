use crate::vec3::{Vec3, Point3, dot, Axis};
use crate::ray::Ray;
use crate::material::Material;
use crate::aabb::AABB;
use crate::util;

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
               t: f64, u: f64, v: f64, mat: Rc<dyn Material>) -> HitRecord {
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
            mat: mat,
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

pub struct Translate {
    obj: Rc<dyn Hittable>,
    offset: Vec3,
}

impl Translate {
    pub fn new(obj: Rc<dyn Hittable>, offset: &Vec3) -> Translate {
        Translate {
            obj: obj,
            offset: *offset,
        }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let moved_r = Ray {
            origin: r.origin - self.offset,
            dir: r.dir,
            time: r.time,
        };
        if let Some(hr) = self.obj.hit(&moved_r, t_min, t_max) {
            Some(HitRecord::new(
                &moved_r, &(hr.p + self.offset), &hr.norm,
                hr.t, hr.u, hr.v, hr.mat.clone()
            ))
        } else {
            None
        }
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        if let Some(bb) = self.obj.bounding_box(time0, time1) {
            Some(AABB{
                min: bb.min + self.offset,
                max: bb.max + self.offset,
            })
        } else {
            None
        }
    }
}

pub struct Rotate {
    obj: Rc<dyn Hittable>,
    axis: Axis,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<AABB>,
}

impl Rotate {
    pub fn rotate_x(obj: Rc<dyn Hittable>, angle: f64) -> Self {
        Self::new(obj, angle, Axis::X)
    }

    pub fn rotate_y(obj: Rc<dyn Hittable>, angle: f64) -> Self {
        Self::new(obj, angle, Axis::Y)
    }

    pub fn rotate_z(obj: Rc<dyn Hittable>, angle: f64) -> Self {
        Self::new(obj, angle, Axis::Z)
    }

    fn new(obj: Rc<dyn Hittable>, angle: f64, axis: Axis) -> Self {
        let radians = util::degrees_to_radians(angle);
        // eprintln!("degrees: {}, radians: {}", angle, radians);
        let sin_theta = f64::sin(radians);
        // eprintln!("sin: {}", sin_theta);
        let cos_theta = f64::cos(radians);
        let (hasbox, bbox) = if let Some(bb) = obj.bounding_box(0.0, 1.0) {
            (true, bb)
        } else {
            (false, AABB::new())
        };

        let mut min = Point3(util::INFINITY, util::INFINITY, util::INFINITY);
        let mut max = Point3(util::NEG_INFINITY, util::NEG_INFINITY, util::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = f64::from(i) * bbox.max.x() + f64::from(1 - i) * bbox.min.x();
                    let y = f64::from(j) * bbox.max.y() + f64::from(1 - j) * bbox.min.y();
                    let z = f64::from(k) * bbox.max.z() + f64::from(1 - k) * bbox.min.z();

                    let (a_coeff, b_coeff) = Self::rot_coeffs(x, y, z, axis);

                    let new_a = cos_theta * a_coeff.0 + sin_theta * a_coeff.1;
                    let new_b = sin_theta * b_coeff.0 + cos_theta * b_coeff.1;

                    let tester = match axis {
                        Axis::X => Vec3(x, new_a, new_b),
                        Axis::Y => Vec3(new_a, y, new_b),
                        Axis::Z => Vec3(new_a, new_b, z),
                    };

                    for c in 0..3 {
                        min[c] = f64::min(min[c], tester[c]);
                        max[c] = f64::max(max[c], tester[c]);
                    }
                }
            }
        }

        Rotate {
            obj: obj,
            axis, sin_theta, cos_theta,
            bbox: if hasbox { Some(AABB {min, max}) } else { None },
        }
    }

    fn rot_coeffs(x: f64, y: f64, z: f64, axis: Axis) -> ((f64,f64), (f64, f64)) {
        let a_coeff = match axis {
            Axis::X => (y, -z),
            Axis::Y => (x, z),
            Axis::Z => (x, -y),
        };
        let b_coeff = match axis {
            Axis::X => (y, z),
            Axis::Y => (-x, z),
            Axis::Z => (x, y),
        };
        (a_coeff, b_coeff)
    }

    fn rot_coeffs_vec(v: &Vec3, axis: Axis) -> ((f64,f64), (f64, f64)) {
        Self::rot_coeffs(v.0, v.1, v.2, axis)
    }

    fn off_axes(rot_axis: Axis) -> (usize, usize) {
        match rot_axis {
            Axis::X => (1, 2),
            Axis::Y => (0, 2),
            Axis::Z => (0, 1),
        }
    }
}

impl Hittable for Rotate {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut origin = r.origin;
        let mut dir = r.dir;

        // Basically rotate the input Ray opposite the specified rotation
        let (a_axis, b_axis) = Self::off_axes(self.axis);
        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(&r.origin, self.axis);
        origin[a_axis] =
            self.cos_theta * a_coeff.0 - self.sin_theta * a_coeff.1;
        origin[b_axis] =
            -self.sin_theta * b_coeff.0 + self.cos_theta * b_coeff.1;

        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(&r.dir, self.axis);
        dir[a_axis] =
            self.cos_theta * a_coeff.0 - self.sin_theta * a_coeff.1;
        dir[b_axis] =
            -self.sin_theta * b_coeff.0 + self.cos_theta * b_coeff.1;

        let rotated_r = Ray { origin, dir, time: r.time };

        let hr = if let Some(hr) = self.obj.hit(&rotated_r, t_min, t_max) {
            hr
        } else {
            return None;
        };

        let mut p = hr.p;
        let mut normal = hr.norm;

        // Then rotate the hit point and the normal vector by theta
        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(&hr.p, self.axis);
        p[a_axis] =
            self.cos_theta * a_coeff.0 + self.sin_theta * a_coeff.1;
        p[b_axis] =
            self.sin_theta * b_coeff.0 + self.cos_theta * b_coeff.1;

        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(&hr.norm, self.axis);
        normal[a_axis] =
            self.cos_theta * a_coeff.0 + self.sin_theta * a_coeff.1;
        normal[b_axis] =
            self.sin_theta * b_coeff.0 + self.cos_theta * b_coeff.1;

        Some(HitRecord::new(
            &rotated_r, &p, &normal, hr.t, hr.u, hr.v, hr.mat.clone()
        ))

    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        self.bbox
        // if self.bbox.is_none() {
        //     None
        // } else {
        //     self.bbox
        //     // Some(AABB{
        //     //     min: self.bbox.as_ref().unwrap().min,
        //     //     max: self.bbox.as_ref().unwrap().max,
        //     // })
        // }
    }
}
