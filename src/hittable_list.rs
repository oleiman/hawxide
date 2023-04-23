use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::aabb::AABB;

use std::vec::Vec;
use std::rc::Rc;

pub struct HittableList {
    pub objects : Vec<Rc<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, obj: Rc<dyn Hittable>) {
        self.objects.push(obj);
    }

    pub fn len(&self) -> usize {
        return self.objects.len();
    }
}

impl Default for HittableList {
    fn default() -> Self {
        Self::new()
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut opt_rec : Option<HitRecord> = None;
        let mut closest : f64 = t_max;

        for obj in &self.objects {
            if let Some(hr) = obj.hit(r, t_min, closest) {
                closest = hr.t;
                opt_rec = Some(hr);
            }
        }

        opt_rec
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        // if self.objects.is_empty() {
        //     return None
        // }
        let mut output_box : Option<AABB> = None;

        for obj in &self.objects {
            if let Some(tmp_box) = obj.bounding_box(time0, time1) {
                output_box = Some(
                    if let Some(ob) = output_box {
                        AABB::surrounding_box(ob, tmp_box)
                    } else {
                        tmp_box
                    }
                );
            } else {
                return None;
            }
        }
        return output_box;
    }
}

