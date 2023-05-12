use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::aabb::AABB;
use crate::vec3::{Vec3, Point3};
use crate::util::random;

use std::vec::Vec;
use std::sync::Arc;

#[derive(Default)]
pub struct HittableList {
    pub objects : Vec<Arc<dyn Hittable + Sync + Send>>,
}

impl HittableList {
    #[must_use]
    pub fn new(objects: Vec<Arc<dyn Hittable + Sync + Send>>)
               -> Self {
        Self {objects}
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, obj: Arc<dyn Hittable + Sync + Send>) {
        self.objects.push(obj);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl From<HittableList> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: HittableList) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
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
        output_box
    }

    fn pdf_value(&self, origin: Point3, v: Vec3) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;

        self.objects.iter().fold(
            0.0,
            |acc, obj| acc + weight * obj.pdf_value(origin, v)
        )
    }

    /// # Panics
    /// 
    /// Panics if self.objects is empty
    fn random(&self, origin: Vec3) -> Vec3 {
        assert!(!self.objects.is_empty());
        self.objects[random::uint(0, self.objects.len() - 1)].random(origin)
    }

    fn empty(&self) -> bool {
        self.objects.is_empty()
    }
}

