use crate::vec3::{Point3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::hittable_list::HittableList;
use crate::material::Material;
use crate::aabb::AABB;
use crate::aarect::AARect;

use std::sync::Arc;

pub struct Boxx {
    box_min: Point3,
    box_max: Point3,
    sides: HittableList,
}

impl Boxx {
    pub fn new(p0: Point3, p1: Point3, mat: Arc<dyn Material + Sync + Send>)
               -> Self {
        let mut sides = HittableList::default();
        sides.add(
            AARect::xy_rect(p0.x(), p1.x(), p0.y(), p1.y(), p1.z(), mat.clone()).into()
        );
        sides.add(
            AARect::xy_rect(p0.x(), p1.x(), p0.y(), p1.y(), p0.z(), mat.clone()).into()
        );

        sides.add(
            AARect::xz_rect(p0.x(), p1.x(), p0.z(), p1.z(), p1.y(), mat.clone()).into()
        );
        sides.add(
            AARect::xz_rect(p0.x(), p1.x(), p0.z(), p1.z(), p0.y(), mat.clone()).into()
        );

        sides.add(
            AARect::yz_rect(p0.y(), p1.y(), p0.z(), p1.z(), p1.x(), mat.clone()).into()
        );
        sides.add(
            AARect::yz_rect(p0.y(), p1.y(), p0.z(), p1.z(), p0.x(), mat.clone()).into()
        );

        Boxx {
            box_min: p0,
            box_max: p1,
            sides
        }
    }
}

impl From<Boxx> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Boxx) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

impl Hittable for Boxx {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.sides.hit(r, t_min, t_max)
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB {
            min: self.box_min,
            max: self.box_max,
        })
    }
}
