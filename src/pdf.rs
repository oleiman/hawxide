use crate::vec3::{Vec3,Point3,Color,dot};
use crate::vec3;
use crate::util::{random,PI};
use crate::onb::OrthoNormalBasis;
use crate::hit::Hittable;
use crate::material::ScatterRecord;

use std::sync::Arc;

pub trait PDensityFn {
    fn value(&self, dir: Vec3) -> f64;
    fn generate(&self, sr: &mut ScatterRecord) -> Vec3;
}

#[derive(Default)]
pub struct NullPDF;

impl NullPDF {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl From<NullPDF> for Arc<dyn PDensityFn + Sync + Send> {
    fn from(pdf: NullPDF) -> Arc<dyn PDensityFn + Sync + Send> {
        Arc::new(pdf)
    }
}

impl PDensityFn for NullPDF {
    fn value(&self, _dir: Vec3) -> f64 { 0.0 }
    fn generate(&self, sr: &mut ScatterRecord) -> Vec3 { Vec3::new() }
}

pub struct CosPDF {
    uvw: OrthoNormalBasis,
}

impl CosPDF {
    #[must_use]
    pub fn new(w: Vec3) -> Self {
        let mut uvw = OrthoNormalBasis::new();
        uvw.build_from_w(w);
        Self {uvw}
    }
}

impl From<CosPDF> for Arc<dyn PDensityFn + Sync + Send> {
    fn from(pdf: CosPDF) -> Arc<dyn PDensityFn + Sync + Send> {
        Arc::new(pdf)
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
    fn generate(&self, sr: &mut ScatterRecord) -> Vec3 {
        self.uvw.local_v(random::cosine_direction())
    }
}

pub struct HittablePDF {
    origin: Point3,
    obj: Arc<dyn Hittable + Sync + Send>,
}

impl HittablePDF {
    #[must_use]
    pub fn new(obj: Arc<dyn Hittable + Sync + Send>, origin: Point3)
               -> Self {
        Self {
            origin,
            obj,
        }
    }
}

impl From<HittablePDF> for Arc<dyn PDensityFn + Sync + Send> {
    fn from(pdf: HittablePDF) -> Arc<dyn PDensityFn + Sync + Send> {
        Arc::new(pdf)
    }
}

impl PDensityFn for HittablePDF {
    fn value(&self, dir: Vec3) -> f64 {
        self.obj.pdf_value(self.origin, dir)
    }
    fn generate(&self, sr: &mut ScatterRecord) -> Vec3 {
        self.obj.random(self.origin)
    }
}

pub struct MixturePDF {
    pub p: [Arc<dyn PDensityFn + Sync + Send>; 2],
}

impl MixturePDF {
    #[must_use]
    pub fn new(p0: Arc<dyn PDensityFn + Sync + Send>,
               p1: Arc<dyn PDensityFn + Sync + Send>)
               -> Self {
        Self {p: [p0, p1]}
    }
}

impl From<MixturePDF> for Arc<dyn PDensityFn + Sync + Send> {
    fn from(pdf: MixturePDF) -> Arc<dyn PDensityFn + Sync + Send> {
        Arc::new(pdf)
    }
}

impl PDensityFn for MixturePDF {
    fn value(&self, dir: Vec3) -> f64 {
        0.5 * self.p[0].value(dir) + 0.5 * self.p[1].value(dir)
    }

    fn generate(&self, sr: &mut ScatterRecord) -> Vec3 {
        if random::double() < 0.5 {
            self.p[0].generate(sr)
        } else {
            self.p[1].generate(sr)
        }
    }
}

pub struct PhongSpecularPDF {
    incident: Vec3,
    pub uvw: OrthoNormalBasis,
    nu: f64,
    nv: f64,
    pub h: Vec3,
    cos_2_phi: f64,
    sin_2_phi: f64,
}

impl PhongSpecularPDF {
    #[must_use]
    pub fn new(incident: Vec3, norm: Vec3, nu: f64, nv: f64) -> Self {
        let mut uvw = OrthoNormalBasis::new();
        uvw.build_from_w(norm);

        let tan_phi_coeff = f64::sqrt((nu + 1.0) / (nv + 1.0));

        let xi_1 = random::double();
        let (xi_1, phase, flip) = Self::quadrants(xi_1);
        let phi = f64::atan(tan_phi_coeff * f64::tan(PI * xi_1 * 0.5));
        let phi = if flip {
            phase - phi
        } else {
            phi + phase
        };
        let cos_phi = f64::cos(phi);
        let sin_phi = f64::sin(phi);
        let cos_2_phi = cos_phi * cos_phi;
        let sin_2_phi = 1.0 - cos_2_phi;

        let xi_2 = random::double();
        let (xi_2, phase, flip) = Self::quadrants(xi_2);
        let exp = 1.0 / (nu * cos_2_phi + nv * sin_2_phi + 1.0);
        let theta = f64::acos(f64::powf(1.0 - xi_2, exp));
        let theta = if flip {
            phase - theta
        } else {
            theta + phase
        };
        let cos_theta = f64::cos(theta);
        let sin_theta = f64::sin(theta);

        let h = uvw.local(
            sin_theta * cos_phi,
            sin_theta * sin_phi,
            cos_theta,
        );
        Self {
            incident, uvw, nu, nv, h,
            cos_2_phi, sin_2_phi,
        }
    }

    fn quadrants(xi: f64) -> (f64, f64, bool) {
        if xi < 0.25 {
            (
                4.0 * xi,
                0.0,
                false
            )
        } else if xi < 0.5 {
            (
                1.0 - 4.0 * (0.5 - xi),
                PI,
                true
            )
        } else if xi < 0.75 {
            (
                1.0 - 4.0 * (0.75 - xi),
                PI,
                false,
            )
        } else {
            (
                1.0 - 4.0 * (1.0 - xi),
                2.0 * PI,
                true,
            )
        }
    }
}

impl From<PhongSpecularPDF> for Arc<dyn PDensityFn + Sync + Send> {
    fn from(pdf: PhongSpecularPDF) -> Arc<dyn PDensityFn + Sync + Send> {
        Arc::new(pdf)
    }
}

impl PDensityFn for PhongSpecularPDF {
    fn value(&self, _dir: Vec3) -> f64 {
        let ph =
            f64::sqrt((self.nu + 1.0) * (self.nv + 1.0)) / (2.0 * PI) *
            f64::powf(dot(self.uvw.w(), self.h),
                      self.nu * self.cos_2_phi + self.nv * self.sin_2_phi);

        ph / (4.0 * dot(-self.incident, self.h))
        // ph / (4.0 * dot(_dir, self.h))
    }

    // NOTE(oren): for simplicity, this only samples from a quadrant
    // of the hemisphere.
    // TODO(oren): deal with quadrants (see Ashikhmin & Shirley)
    fn generate(&self, _sr: &mut ScatterRecord) -> Vec3 {
        // vec3::reflect(self.incident, self.uvw.w())
        vec3::reflect(self.incident, self.h)
    }
}

pub struct PhongPDF {
    diffuse: CosPDF,
    specular: PhongSpecularPDF,
}

impl PhongPDF {
    #[must_use]
    pub fn new(incident: Vec3, norm: Vec3) -> Self {
        let diffuse = CosPDF::new(norm);
        let specular = PhongSpecularPDF::new(
            incident.unit_vector(), norm, 1000.0, 1000.0);
        Self {
            diffuse,
            specular,
        }
    }
}

impl From<PhongPDF> for Arc<dyn PDensityFn + Sync + Send> {
    fn from(pdf: PhongPDF) -> Arc<dyn PDensityFn + Sync + Send> {
        Arc::new(pdf)
    }
}

impl PDensityFn for PhongPDF {
    fn value(&self, dir: Vec3) -> f64 {
        self.diffuse.value(dir)
    }

    fn generate(&self, sr: &mut ScatterRecord) -> Vec3 {
        let sel = random::double();
        let spec = self.specular.generate(sr);

        let diffuse_p = if dot(spec, self.specular.uvw.w()) < 0.0 {
            1.0
        } else {
            1.0 / (1.0 + self.specular.value(spec))
        };

        // eprintln!("{}", diffuse_p);

        if sel < diffuse_p {
            self.diffuse.generate(sr)
        } else {
            // eprintln!("specular");
            sr.attenuation =
                0.8 * sr.specular_color.unwrap_or(Color(1.0, 1.0, 1.0)) +
                0.2 * sr.attenuation;
            spec
        }
    }
}


