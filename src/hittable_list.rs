use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};

use std::vec::Vec;
use std::boxed::Box;

pub struct HittableList {
    objects : Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> HittableList {
        HittableList {
            objects: Vec::new(),
        }
    }
    pub fn clear(&mut self) {
        self.objects.clear();
    }
    pub fn add(&mut self, obj: Box<dyn Hittable>) {
        self.objects.push(obj);
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let mut opt_rec : Option<HitRecord> = None;
        let mut closest : f64 = t_max;

        for obj in self.objects.iter() {
            opt_rec = match obj.hit(r, t_min, closest) {
                Some(hr) => {
                    closest = hr.t;
                    Some(hr)
                },
                _ => opt_rec,
            };
        }

        opt_rec
    }
}
