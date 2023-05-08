use crate::vec3::{Vec3,cross};

use std::ops;

pub struct OrthoNormalBasis {
    axis: [Vec3; 3],
}

impl OrthoNormalBasis {
    pub fn new() -> OrthoNormalBasis {
        OrthoNormalBasis {
            axis: [Vec3::new(), Vec3::new(), Vec3::new()],
        }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    pub fn local(&self, a: f64, b: f64, c: f64) -> Vec3 {
        a * self.u() + b * self.v() + c * self.w()
    }

    pub fn local_v(&self, a: &Vec3) -> Vec3 {
        self.local(a.x(), a.y(), a.z())
    }

    pub fn build_from_w(&mut self, n: &Vec3) {
        self.axis[2] = n.unit_vector();
        let a = if f64::abs(self.w().x()) > 0.9 {
            Vec3(0.0, 1.0, 0.0)
        } else {
            Vec3(1.0, 0.0, 0.0)
        };
        self.axis[1] = cross(&self.w(), &a);
        self.axis[0] = cross(&self.w(), &self.v());
    }
}

impl ops::Index<usize> for OrthoNormalBasis {
    type Output = Vec3;

    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < 3, "Index {} out of range for Vec3", idx);
        &self.axis[idx]
    }
}

impl ops::IndexMut<usize> for OrthoNormalBasis {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        assert!(idx < 3, "Index {} out of range for Vec3", idx);
        &mut self.axis[idx]
    }
}
