use crate::vec3::{Vec3,Point3};

pub struct Ray {
    pub origin : Point3,
    pub dir : Vec3,
    pub time : f64,
}

impl Ray {
    #[must_use]
    pub fn new(origin: Point3, dir: Vec3, time: f64) -> Self {
        Self {
            origin, dir, time,
        }
    }

    #[must_use]
    pub fn at(&self, t : f64) -> Point3 {
        self.origin + t * self.dir
    }
}


