use crate::vec3::{Point3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::hittable_list::HittableList;
use crate::material::Material;
use crate::aabb::AABB;
use crate::aarect::AARect;

use std::rc::Rc;

pub struct Boxx {
    box_min: Point3,
    box_max: Point3,
    sides: HittableList,
}

impl Boxx {
    pub fn new(p0: &Point3, p1: &Point3, mat: &Rc<dyn Material>) -> Self {
        let mut sides = HittableList::new();
        sides.add(Rc::new(
            AARect::xy_rect(p0.x(), p1.x(), p0.y(), p1.y(), p1.z(), mat)
        ));
        sides.add(Rc::new(
            AARect::xy_rect(p0.x(), p1.x(), p0.y(), p1.y(), p0.z(), mat)
        ));

        sides.add(Rc::new(
            AARect::xz_rect(p0.x(), p1.x(), p0.z(), p1.z(), p1.y(), mat)
        ));
        sides.add(Rc::new(
            AARect::xz_rect(p0.x(), p1.x(), p0.z(), p1.z(), p0.y(), mat)
        ));

        sides.add(Rc::new(
            AARect::yz_rect(p0.y(), p1.y(), p0.z(), p1.z(), p1.x(), mat)
        ));
        sides.add(Rc::new(
            AARect::yz_rect(p0.y(), p1.y(), p0.z(), p1.z(), p0.x(), mat)
        ));

        Boxx {
            box_min: *p0,
            box_max: *p1,
            sides
        }
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
