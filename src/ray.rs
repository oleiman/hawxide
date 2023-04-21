use crate::vec3::{Vec3,Point3};

pub struct Ray {
    pub origin : Point3,
    pub dir : Vec3,
}

impl Ray {
    pub fn at(&self, t : f64) -> Point3 {
        self.origin + t * self.dir
    }
}


