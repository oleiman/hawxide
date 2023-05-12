use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::hittable_list::HittableList;
use crate::aabb::AABB;
use crate::util::random;

use std::sync::Arc;
use std::cmp::Ordering;

pub struct BVHNode {
    left: Arc<dyn Hittable + Sync + Send>,
    right: Arc<dyn Hittable + Sync + Send>,
    bbox: AABB,
}

impl BVHNode {
    #[must_use]
    pub fn new(list: &HittableList, time0: f64, time1: f64)
               -> Self {
        Self::new_slice(&list.objects, time0, time1)
    }

    /// # Panics
    ///
    /// Will panic if either the left or right child has no bounding box
    #[must_use]
    pub fn new_slice(src_objects: &[Arc<dyn Hittable + Sync + Send>], time0: f64, time1: f64)
                     -> Self {
        let mut objects = Vec::<Arc<dyn Hittable + Sync + Send>>::new();
        for o in src_objects {
            objects.push(o.clone());
        }

        let comparator = |a: &Arc<dyn Hittable + Sync + Send>, b: &Arc<dyn Hittable + Sync + Send>| -> Ordering {
            box_compare(a, b, random::uint(0, 2))
        };

        let (left, right) : (Arc<dyn Hittable + Sync + Send>, Arc<dyn Hittable + Sync + Send>) = match objects.len() {
            1 => (objects[0].clone(), objects[0].clone()),
            2 =>  {
                objects.sort_by(comparator);
                (objects[0].clone(), objects[1].clone())
            },
            _ => {
                objects.sort_by(comparator);
                let mid = objects.len() / 2;
                (BVHNode::new_slice(&objects[0..mid], time0, time1).into(),
                 BVHNode::new_slice(&objects[mid..], time0, time1).into())
            }
        };

        let box_left = left.bounding_box(time0, time1);
        let box_right = right.bounding_box(time0, time1);

        assert!(box_left.is_some() && box_right.is_some(),
                "No bounding box in BVHNode constructor");

        Self {
            left,
            right,
            bbox: AABB::surrounding_box(box_left.unwrap(), box_right.unwrap()),
        }
    }
}

impl From<BVHNode> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: BVHNode) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

fn box_compare(a: &Arc<dyn Hittable + Sync + Send>, b: &Arc<dyn Hittable + Sync + Send>, axis: usize) -> Ordering {
    let box_a = a.bounding_box(0.0, 0.0);
    let box_b = b.bounding_box(0.0, 0.0);

    assert!(box_a.is_some() && box_b.is_some(),
            "No bounding box in BVHNode constructor");

    box_a.unwrap().min[axis].partial_cmp(&box_b.unwrap().min[axis]).unwrap()
}

impl Hittable for BVHNode {
    fn hit(&self, r : &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        if !self.bbox.hit(r, t_min, t_max) {
            return None;
        }

        if let Some(hr_l) = self.left.hit(r, t_min, t_max) {
            if let Some(hr_r) = self.right.hit(r, t_min, hr_l.t) {
                Some(hr_r)
            } else {
                Some(hr_l)
            }
        } else {
            self.right.hit(r, t_min, t_max)
        }
    }

    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        Some(AABB{
            min: self.bbox.min,
            max: self.bbox.max,
        })
    }
}
