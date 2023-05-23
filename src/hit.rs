use crate::vec3::{Vec3, Point3, dot, Axis, cross};
use crate::ray::Ray;
use crate::material::Material;
use crate::aabb::AABB;
use crate::util;

use std::sync::Arc;

#[derive(Clone)]
pub struct ShadingGeometry {
    pub n: Vec3,
    // for recomputing the normal if we perturb the hit point later
    pub dpdu: Vec3,
    pub dpdv: Vec3,
    // don't need these just yet
    // pub dndu: Vec3,
    // pub dndv: Vec3,
}

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub norm: Vec3,
    pub mat: Arc<dyn Material + Sync + Send>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
    pub shading_geo: ShadingGeometry,
}

impl HitRecord {
    #[must_use]
    pub fn new(ray: &Ray, p: Point3, out_norm: Vec3,
               t: f64, u: f64, v: f64, mat: Arc<dyn Material + Sync + Send>) -> HitRecord {
        // out_norm always points outward from the hittable object
        // instead, we want our hit record norm to point against the
        // ray, thereby telling us whether the ray is inside or outside
        // the object. So if the dot product is < 0, then the angle
        // between ray and surface normal is > 90deg, so we note that
        // the ray intersects the outer face of the surface and leave
        // the normal alone. Otherwise the ray is inside the surface,
        // so we note that and reverse the direction of the normal
        let front_face : bool  = dot(ray.dir, out_norm) < 0.;
        let norm = if front_face {
            out_norm
        } else {
            -out_norm
        };
        HitRecord {
            p,
            norm,
            mat,
            t, u, v,
            front_face,
            shading_geo: ShadingGeometry {
                n: norm,
                dpdu: Vec3::new(),
                dpdv: Vec3::new(),
            }
        }
    }

    pub fn with_dps(ray: &Ray, p: Point3, out_norm: Vec3,
                    t: f64, u: f64, v: f64, mat: Arc<dyn Material + Sync + Send>,
                    dpdu: Vec3, dpdv: Vec3) -> HitRecord {
        let mut hr = Self::new(ray, p, out_norm, t, u, v, mat);
        hr.shading_geo.dpdu = dpdu;
        hr.shading_geo.dpdv = dpdv;
        hr
    }

    pub fn set_shading_geometry(&mut self, dpdu: Vec3, dpdv: Vec3) {
        let mut n = cross(dpdu, dpdv).unit_vector();

        if n.is_nan() {
            return;
        }

        if dot(self.norm, n) < 0.0 {
            n = -n;
        }

        // eprintln!("{}", f64::acos(dot(self.norm, n) / (self.norm.len() * n.len())));

        // TODO(oren): danger zone
        // self.norm = n;
        self.shading_geo = ShadingGeometry{
            n, dpdu, dpdv,
        }
    }


    pub fn set_face_normal(&mut self, r: &Ray, out_norm: Vec3) {

        self.front_face = dot(r.dir, out_norm) < 0.;
        self.norm = if self.front_face {
            out_norm
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

    fn pdf_value(&self, _origin: Point3, _v: Vec3) -> f64 {
        0.0
    }

    fn random(&self, _origin: Vec3) -> Vec3 {
        Vec3(1.0, 0.0, 0.0)
    }

    fn empty(&self) -> bool {
        false
    }
}

pub struct Translate {
    obj: Arc<dyn Hittable + Sync + Send>,
    offset: Vec3,
}

impl Translate {
    #[must_use]
    pub fn new(obj: Arc<dyn Hittable + Sync + Send>, offset: Vec3)
               -> Self {
        Self{ obj, offset }
    }
}

impl From<Translate> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Translate) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
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
            Some(HitRecord::with_dps(
                &moved_r, hr.p + self.offset, hr.norm,
                hr.t, hr.u, hr.v, hr.mat.clone(),
                hr.shading_geo.dpdu, hr.shading_geo.dpdv,
            ))
        } else {
            None
        }
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.obj.bounding_box(time0, time1).map(|bb| AABB{
            min: bb.min + self.offset,
            max: bb.max + self.offset,
        })
    }

    fn pdf_value(&self, origin: Point3, v: Vec3) -> f64 {
        self.obj.pdf_value(origin, v)
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        self.obj.random(origin)
    }
}

pub struct Rotate {
    obj: Arc<dyn Hittable + Sync + Send>,
    axis: Axis,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Option<AABB>,
}

impl Rotate {
    #[must_use]
    pub fn rotate_x(obj: Arc<dyn Hittable + Sync + Send>, angle: f64)
                    -> Self {
        Self::new(obj, angle, Axis::X)
    }

    #[must_use]
    pub fn rotate_y(obj: Arc<dyn Hittable + Sync + Send>, angle: f64)
                    -> Self {
        Self::new(obj, angle, Axis::Y)
    }

    #[must_use]
    pub fn rotate_z(obj: Arc<dyn Hittable + Sync + Send>, angle: f64)
                    -> Self {
        Self::new(obj, angle, Axis::Z)
    }

    fn new(obj: Arc<dyn Hittable + Sync + Send>, angle: f64, axis: Axis)
           -> Self {
        let radians = util::degrees_to_radians(angle);
        let sin_theta = f64::sin(radians);
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

        Self {
            obj, axis, sin_theta, cos_theta,
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

    fn rot_coeffs_vec(v: Vec3, axis: Axis) -> ((f64,f64), (f64, f64)) {
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

impl From<Rotate> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Rotate) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

impl Hittable for Rotate {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut origin = r.origin;
        let mut dir = r.dir;

        let sin_neg_theta = -self.sin_theta;
        let cos_neg_theta = self.cos_theta;

        // Basically rotate the input Ray opposite the specified rotation
        let (a_axis, b_axis) = Self::off_axes(self.axis);
        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(r.origin, self.axis);
        origin[a_axis] =
            cos_neg_theta * a_coeff.0 + sin_neg_theta * a_coeff.1;
        origin[b_axis] =
            sin_neg_theta * b_coeff.0 + cos_neg_theta * b_coeff.1;

        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(r.dir, self.axis);
        dir[a_axis] =
            cos_neg_theta * a_coeff.0 + sin_neg_theta * a_coeff.1;
        dir[b_axis] =
            sin_neg_theta * b_coeff.0 + cos_neg_theta * b_coeff.1;

        let rotated_r = Ray { origin, dir, time: r.time };

        let hr = self.obj.hit(&rotated_r, t_min, t_max)?;

        let mut p = hr.p;
        let mut normal = hr.norm;

        // Then rotate the hit point and the normal vector by theta
        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(hr.p, self.axis);
        p[a_axis] =
            self.cos_theta * a_coeff.0 + self.sin_theta * a_coeff.1;
        p[b_axis] =
            self.sin_theta * b_coeff.0 + self.cos_theta * b_coeff.1;

        let (a_coeff, b_coeff) = Self::rot_coeffs_vec(normal, self.axis);
        normal[a_axis] =
            self.cos_theta * a_coeff.0 + self.sin_theta * a_coeff.1;
        normal[b_axis] =
            self.sin_theta * b_coeff.0 + self.cos_theta * b_coeff.1;

        Some(HitRecord::with_dps(
            &rotated_r, p, normal, hr.t, hr.u, hr.v, hr.mat.clone(),
            hr.shading_geo.dpdu, hr.shading_geo.dpdv,
        ))

    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        self.bbox
    }

    fn pdf_value(&self, origin: Point3, v: Vec3) -> f64 {
        self.obj.pdf_value(origin, v)
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        self.obj.random(origin)
    }
}

pub struct FlipFace {
    obj: Arc<dyn Hittable + Sync + Send>,
}

impl FlipFace {
    #[must_use]
    pub fn new(obj: Arc<dyn Hittable + Sync + Send>)
               -> Self {
        Self {obj}
    }
}

impl From<FlipFace> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: FlipFace) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

impl Hittable for FlipFace {

    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if let Some(mut hr) = self.obj.hit(r, t_min, t_max) {
            hr.front_face = !hr.front_face;
            Some(hr)
        } else {
            None
        }
    }

    // Give the smallest reasonable AABB for the Hittable
    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.obj.bounding_box(time0, time1)
    }

    // TODO(oren): Not sure this is quite right
    fn pdf_value(&self, origin: Point3, v: Vec3) -> f64 {
        self.obj.pdf_value(origin, v)
    }

    fn random(&self, origin: Vec3) -> Vec3 {
        self.obj.random(origin)
    }
}
