use crate::vec3::{Vec3,Point3,dot};
use crate::util::{random,PI};
use crate::onb::OrthoNormalBasis;
use crate::hit::Hittable;

use std::sync::Arc;

pub trait PDensityFn {
    fn value(&self, dir: Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct NullPDF;

impl NullPDF {
    pub fn new() -> Self {
        Self{}
    }
}

impl PDensityFn for NullPDF {
    fn value(&self, _dir: Vec3) -> f64 { 0.0 }
    fn generate(&self) -> Vec3 { Vec3::new() }
}

pub struct CosPDF {
    uvw: OrthoNormalBasis,
}

impl CosPDF {
    pub fn new(w: Vec3) -> Self {
        let mut uvw = OrthoNormalBasis::new();
        uvw.build_from_w(w);
        Self {uvw}
    }
}

impl PDensityFn for CosPDF {
    fn value(&self, dir: Vec3) -> f64 {
        let cosine = dot(dir.unit_vector(), self.uvw.w());
        if cosine <= 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }
    fn generate(&self) -> Vec3 {
        self.uvw.local_v(random::cosine_direction())
    }
}

pub struct HittablePDF {
    origin: Point3,
    obj: Arc<dyn Hittable + Sync + Send>,
}

impl HittablePDF {
    pub fn new(obj: Arc<dyn Hittable + Sync + Send>, origin: Point3) -> Self {
        Self {
            origin: origin,
            obj: obj.clone(),
        }
    }
}

impl PDensityFn for HittablePDF {
    fn value(&self, dir: Vec3) -> f64 {
        self.obj.pdf_value(self.origin, dir)
    }
    fn generate(&self) -> Vec3 {
        self.obj.random(self.origin)
    }
}

pub struct MixturePDF {
    p: [Arc<dyn PDensityFn + Sync + Send>; 2],
}

impl MixturePDF {
    pub fn new(p0: Arc<dyn PDensityFn + Sync + Send>,
               p1: Arc<dyn PDensityFn + Sync + Send>) -> Self {
        Self {p: [p0, p1]}
    }
}

impl PDensityFn for MixturePDF {
    fn value(&self, dir: Vec3) -> f64 {
        0.5 * self.p[0].value(dir) + 0.5 * self.p[1].value(dir)
    }

    fn generate(&self) -> Vec3 {
        if random::double() < 0.5 {
            self.p[0].generate()
        } else {
            self.p[1].generate()
        }
    }
}


