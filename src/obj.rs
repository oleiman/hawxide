use crate::vec3::{Point3, Color};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::triangle::Triangle;
use crate::hittable_list::HittableList;
use crate::bvh::BVHNode;
use crate::material::{Metal, Lambertian, DiffuseLight, WfMtl};

use tobj;

use std::sync::Arc;
use std::vec::Vec;

// TODO(oren): may want to pre-compute the bounding box

pub struct WfObject {
    pub triangles: HittableList,
    pub mat: Arc<dyn Material + Sync + Send>,
}

impl WfObject {
    #[must_use]
    pub fn new(fname: &str, scale: f64, mat: Arc<dyn Material + Sync + Send>) -> Self{
        let mut load_opts = tobj::OFFLINE_RENDERING_LOAD_OPTIONS;
        load_opts.triangulate = true;
        let obj = tobj::load_obj(fname, &load_opts);
        assert!(obj.is_ok());
        let (models, mats_r) = obj.unwrap();
        assert!(mats_r.is_ok());
        let mats = mats_r.unwrap();

        let materials: Vec<Arc<dyn Material + Sync + Send>> =
            mats.iter().map(|m| {
                Self::get_material(m)
            }).collect();

        let mut triangles = HittableList::default();
        let mut n_points = 0usize;
        let mut n_faces = 0usize;

        for m in &models {
            let mesh = &m.mesh;

            let mat: Arc<dyn Material + Sync + Send> =
                if let Some(m_id) = mesh.material_id {
                    assert!(m_id <= materials.len());
                    if m_id <= materials.len() {
                        materials[m_id].clone()
                    } else {
                        mat.clone()
                    }
                } else {
                    mat.clone()
                };

            let mut points: Vec<Point3> = vec![];
            for p in mesh.positions.chunks(3) {
                points.push(
                    Point3(p[0], p[1], p[2]) * scale
                );
            }

            let mut m_tri = HittableList::default();

            for face in mesh.indices.chunks(3) {
                if face.len() != 3 {
                    continue;
                }
                let p0_i = face[0] as usize;
                let p1_i = face[1] as usize;
                let p2_i = face[2] as usize;


                m_tri.add(Triangle::new(
                    points[p0_i], points[p1_i], points[p2_i], mat.clone()
                ).into());
            }

            n_faces += m_tri.len();
            n_points += points.len();
            triangles.add(m_tri.into());
        }

        eprintln!("{}: models: {}, positions: {}, faces: {}, materials: {}",
                  fname, models.len(), n_points, n_faces, mats.len(),
        );

        Self {
            triangles, mat
        }
    }

    fn get_material (mm: &tobj::Material) -> Arc<dyn Material + Sync + Send> {

        let dc = if let Some([r, g, b]) = mm.diffuse {
            Color(r, g, b)
        } else {
            Color(0.8, 0.8, 0.8)
        };
        let sc = if let Some([r, g, b]) = mm.specular {
            Color(r, g, b)
        } else {
            Color(1.0, 1.0, 1.0)
        };
        let ac = if let Some([r, g, b]) = mm.ambient {
            Color(r, g, b)
        } else {
            Color(0.2, 0.2, 0.2)
        };
        let ns = if let Some(ns) = mm.shininess { ns } else { 0.0 };

        match mm.illumination_model {
            Some(x) => {
                WfMtl::new(
                    x,
                    Lambertian::new(dc),
                    Metal::new(sc, 1.0 - ns / 1000.0),
                    DiffuseLight::new(ac),
                ).into()
            },
            _ => {
                Lambertian::new(dc).into()
            },
        }
    }
}

impl From<WfObject> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: WfObject) -> Arc<dyn Hittable + Sync + Send> {
        BVHNode::new(&hh.triangles, 0.0, 1.0).into()
    }
}

impl Hittable for WfObject {
    #[must_use]
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        self.triangles.hit(r, t_min, t_max)
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.triangles.bounding_box(time0, time1)
    }
}