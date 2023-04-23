use crate::vec3::{Point3};
use crate::ray::Ray;

use std::mem::swap;

pub struct AABB {
    pub min: Point3,
    pub max: Point3,
}

impl AABB {

    // NOTE: unoptimized version. kept around for reference.
    // Though the optimized version is not much different, just precomputes
    // the inverse direction and aboids some min/max calls.
    // pub fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
    //     for a in 0..3 {
    //         let t0 = f64::min(
    //             (self.min[a] - ray.origin[a]) / ray.dir[a],
    //             (self.max[a] - ray.origin[a]) / ray.dir[a]
    //         );
    //         let t1 = f64::max(
    //             (self.min[a] - ray.origin[a]) / ray.dir[a],
    //             (self.max[a] - ray.origin[a]) / ray.dir[a]
    //         );
    //         t_min = f64::max(t0, t_min);
    //         t_max = f64::min(t1, t_max);
    //         if t_max < t_min {
    //             return false;
    //         }
    //     }
    //     return true;
    // }

    pub fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for a in 0..3 {
            let inv_dir = 1.0 / ray.dir[a];
            let mut t0 = (self.min[a] - ray.origin[a]) * inv_dir;
            let mut t1 = (self.max[a] - ray.origin[a]) * inv_dir;
            if inv_dir < 0.0 {
                swap(&mut t0, &mut t1);
            }
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max <= t_min {
                return false;
            }
        }
        true
    }

    // calculate a box that includes both box0 and box1
    pub fn surrounding_box(box0: AABB, box1: AABB) -> AABB {
        AABB {
            min: Point3 (
                f64::min(box0.min.x(), box1.min.x()),
                f64::min(box0.min.y(), box1.min.y()),
                f64::min(box0.min.z(), box1.min.z()),
            ),
            max: Point3 (
                f64::max(box0.max.x(), box1.max.x()),
                f64::max(box0.max.y(), box1.max.y()),
                f64::max(box0.max.z(), box1.max.z()),
            ),
        }

    }
}




