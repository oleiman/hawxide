use crate::vec3::{Vec3, Point3, dot};
use crate::ray::Ray;
use crate::material::Material;

pub struct HitRecord<'a> {
    pub p: Point3,
    pub norm: Vec3,
    pub mat: &'a (dyn Material + 'static),
    pub t: f64,
    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new(r: &Ray, p: &Point3, out_norm: &Vec3, t: f64, mat: &'a (dyn Material + 'static))
               -> HitRecord<'a> {
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
            norm: match front_face {
                true => *out_norm,
                false => -out_norm,
            },
            mat: mat,
            t: t,
            front_face: front_face,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, out_norm: &Vec3) {

        self.front_face = dot(&r.dir, out_norm) < 0.;
        self.norm = match self.front_face {
            true => *out_norm,
            false => -out_norm,
        };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord>;
}
