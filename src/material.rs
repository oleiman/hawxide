use crate::ray::Ray;
use crate::hit::HitRecord;
use crate::vec3::{Color, Vec3, Point3};
use crate::vec3;
use crate::util::random;
use crate::texture::{Texture,FloatTexture,SolidColor,RandomBump,CheckerBump};
use crate::util::PI;
use crate::pdf::{PDensityFn,CosPDF,NullPDF};

use std::sync::Arc;

pub struct ScatterRecord {
    pub specular_ray: Option<Ray>,
    pub attenuation: Color,
    pub pdf: Arc<dyn PDensityFn + Sync + Send>,
}

pub trait Material {
    fn scatter(&self, _ray_in: &Ray, _rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }
    fn scattering_pdf(&self, _ray_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }
    fn emitted(&self, _ray_in: &Ray, _rec: &HitRecord,
               _u: f64, _v: f64, _p: Point3) -> Color {
        Color(0.0, 0.0, 0.0)
    }

    fn bump(&self, d: &Arc<dyn FloatTexture + Sync + Send>, rec: &HitRecord) -> HitRecord {
        let mut rec = (*rec).clone();
        let mut rec_eval = rec.clone();
        let du = 0.01;
        let dv = 0.01;

        // eprintln!("dpdu: {}, dpdv: {}", rec.shading_geo.dpdu, rec.shading_geo.dpdv);

        rec_eval.p = rec.p + du * rec.shading_geo.dpdu;
        rec_eval.u = rec.u + du;
        let u_disp = d.value(rec_eval.u, rec_eval.v, rec_eval.p);

        rec_eval.p = rec.p + dv * rec.shading_geo.dpdv;
        rec_eval.u = rec.u;
        rec_eval.v = rec.v + dv;
        let v_disp = d.value(rec_eval.u, rec_eval.v, rec_eval.p);

        let disp = d.value(rec.u, rec.v, rec.p);

        let dpdu = rec.shading_geo.dpdu +
            (u_disp - disp) / du * rec.shading_geo.n;

        let dpdv = rec.shading_geo.dpdv +
            (v_disp - disp) / dv * rec.shading_geo.n;

        rec.set_shading_geometry(dpdu, dpdv);

        rec
    }
}

pub struct Lambertian {
    pub albedo: Arc<dyn Texture + Sync + Send>,
}

impl Lambertian {
    #[must_use]
    pub fn new(c: Color) -> Self {
        Self::from_texture(SolidColor::new(c).into())
    }

    #[must_use]
    pub fn from_texture(albedo: Arc<dyn Texture + Sync + Send>)
                        -> Self {
        Self { albedo }
    }
}

impl From<Lambertian> for Arc<dyn Material + Sync + Send> {
    fn from(mm: Lambertian) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for Lambertian {
    #[allow(unused_variables)]
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<ScatterRecord> {
        Some(ScatterRecord{
            specular_ray: None,
            attenuation: self.albedo.value(rec.u, rec.v, rec.p),
            pdf: CosPDF::new(rec.norm).into(),
        })
    }

    fn scattering_pdf(&self, _ray_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = vec3::dot(rec.norm, scattered.dir.unit_vector());
        if cosine < 0.0 {
            0.0
        } else {
            cosine / PI
        }
    }
}

pub struct Metal {
    pub albedo: Arc<dyn Texture + Sync + Send>,
    pub fuzz: f64,
}

impl Metal {
    #[must_use]
    pub fn new(c: Color, fuzz: f64) -> Self {
        Self {
            albedo: SolidColor::new(c).into(),
            fuzz,
        }
    }
}

impl From<Metal> for Arc<dyn Material + Sync + Send> {
    fn from(mm: Metal) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<ScatterRecord> {
        let reflected = vec3::reflect(ray_in.dir.unit_vector(), rec.norm);
        let f = match self.fuzz {
            f if f <= 1.0 => f,
            _ => 1.0,
        };

        let dir = reflected + f * Vec3::random_in_unit_sphere();
        if vec3::dot(dir, rec.norm) <= 0.0 {
            return None;
        }

        Some(ScatterRecord{
            specular_ray: Some(Ray::new(
                rec.p, dir, ray_in.time)
            ),
            attenuation: self.albedo.value(rec.u, rec.v, rec.p),
            pdf: NullPDF::new().into(),
        })
    }
}

pub struct Dielectric {
    pub ir : f64,
    pub density: f64,
    pub vol_color: Color,
    pub albedo: Arc<dyn Texture + Sync + Send>,
}

impl Dielectric {
    #[must_use]
    pub fn new(ir: f64, density: f64, vol_color: Color) -> Self {
        Self {
            ir, density,vol_color,
            albedo: SolidColor::new(Color(1.0, 1.0, 1.0)).into(),

        }
    }

    fn reflectance(cos: f64, ref_idx: f64) -> f64 {
        // Use Schlick's approximation for reflectance
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 *= r0;
        r0 + (1. - r0) * (1. - cos).powf(5.)
    }

    fn absorbance(dist: f64, c: Color, alpha: f64) -> Color {
        (Color(1.0, 1.0, 1.0) - c) * alpha * -dist
    }
}

impl From<Dielectric> for Arc<dyn Material + Sync + Send> {
    fn from(mm: Dielectric) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
               -> Option<ScatterRecord> {
        let refraction_ratio : f64 = if rec.front_face {
            1.0 / self.ir
        } else {
            self.ir
        };

        let attenuation = if rec.front_face {
            self.albedo.value(rec.u, rec.v, rec.p)
        } else {
            let absorb = Self::absorbance(rec.t, self.vol_color, self.density);
            let atten = absorb.exp();
            self.albedo.value(rec.u, rec.v, rec.p) * atten
        };


        let unit_direction = ray_in.dir.unit_vector();
        let cos_theta = vec3::dot(-unit_direction, rec.norm).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract: bool =
            (refraction_ratio * sin_theta > 1.0) ||
            Dielectric::reflectance(cos_theta, refraction_ratio) > random::double();

        let direction = if cannot_refract {
            vec3::reflect(unit_direction, rec.norm)
        } else {
            vec3::refract(unit_direction, rec.norm, refraction_ratio)
        };

        Some(ScatterRecord {
            specular_ray: Some(Ray::new(
                rec.p,
                direction,
                ray_in.time
            )),
            attenuation,
            pdf: NullPDF::new().into(),
        })
    }
}

pub struct DiffuseLight {
    pub emit: Arc<dyn Texture + Sync + Send>,
}

impl DiffuseLight {
    #[must_use]
    pub fn new(c: Color) -> Self {
        Self {
            emit: SolidColor::new(c).into(),
        }
    }
}

impl From<DiffuseLight> for Arc<dyn Material + Sync + Send> {
    fn from(mm: DiffuseLight) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, _ray_in: &Ray, rec: &HitRecord,
               u: f64, v: f64, p: Point3) -> Color {
        if rec.front_face {
        self.emit.value(u, v, p)
        } else {
            Color(0.0, 0.0, 0.0)
        }
    }
}

pub struct Isotropic {
    pub albedo: Arc<dyn Texture + Sync + Send>,
}

impl Isotropic {
    #[must_use]
    pub fn new(c: Color) -> Self {
        Self::from_texture(SolidColor::new(c).into())
    }

    #[must_use]
    pub fn from_texture(albedo: Arc<dyn Texture + Sync + Send>)
                        -> Self{
        Self {albedo}
    }
}

impl From<Isotropic> for Arc<dyn Material + Sync + Send> {
    fn from(mm: Isotropic) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        Some(ScatterRecord{
            specular_ray: Some(Ray::new(
                rec.p, Vec3::random_in_unit_sphere(), ray_in.time,
            )),
            attenuation: self.albedo.value(rec.u, rec.v, rec.p),
            pdf: NullPDF::new().into(),
        })
    }
}

pub struct WfMtl {
    pub model: u8,
    pub diffuse: Lambertian,
    pub specular: Metal,
    pub ambient: DiffuseLight,
}

impl WfMtl {
    #[must_use]
    pub fn new(model: u8,
               diffuse: Lambertian,
               specular: Metal,
               ambient: DiffuseLight) -> WfMtl {
        assert!(model <= 2, "Model {} not supported", model);
        WfMtl {
            diffuse,
            specular,
            ambient,
            model,
        }
    }
}

impl From<WfMtl> for Arc<dyn Material + Sync + Send> {
    fn from(mm: WfMtl) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for WfMtl {
    fn emitted(&self, ray_in: &Ray, rec: &HitRecord,
               u: f64, v: f64, p: Point3) -> Color {
        if self.model == 0 {
            self.diffuse.albedo.value(u, v, p)
        } else {
            self.ambient.emitted(ray_in, rec, u, v, p)
        }
    }

    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let sel = random::double();
        match self.model {
            0 => {
                // self.diffuse.scatter(ray_in, rec)
                None
            },
            1 => {
                self.diffuse.scatter(ray_in, rec)
            },
            _ => {
                // TODO(oren): maybe change the ratio?
                if sel < 0.9 {
                    self.diffuse.scatter(ray_in, rec)
                } else {
                    self.specular.scatter(ray_in, rec)
                }
            }
        }

    }

    fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        self.diffuse.scattering_pdf(ray_in, rec, scattered)
    }
}

pub struct Corroded {
    pub bump_t: Arc<dyn FloatTexture + Sync + Send>,
    pub mat: Arc<dyn Material + Sync + Send>,
}

impl Corroded {
    #[must_use]
    pub fn new(mat: Arc<dyn Material + Sync + Send>) -> Self {
        Self {
            bump_t: RandomBump::new().into(),
            // bump_t: CheckerBump::new(-5.0, 0.0).into(),
            mat: mat.clone(),
        }
    }
}

impl From<Corroded> for Arc<dyn Material + Sync + Send> {
    fn from(mm: Corroded) -> Arc<dyn Material + Sync + Send> {
        Arc::new(mm)
    }
}

impl Material for Corroded {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord)
        -> Option<ScatterRecord> {
        let rec = self.bump(&self.bump_t, rec);
        self.mat.scatter(ray_in, &rec)
        // self.mat.scatter(ray_in, rec)
    }

    fn scattering_pdf(&self, ray_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let rec = self.bump(&self.bump_t, rec);
        self.mat.scattering_pdf(ray_in, &rec, scattered)
            // self.mat.scattering_pdf(ray_in, rec, scattered)
    }

}

