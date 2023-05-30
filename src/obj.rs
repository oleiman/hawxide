use crate::vec3::{Point3, Color, Vec3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::triangle::Triangle;
use crate::hittable_list::HittableList;
use crate::bvh::BVHNode;
use crate::material::{Metal, Lambertian, DiffuseLight, WfMtl};
use crate::triangle_mesh::TriangleMesh;
use crate::texture;

use tobj;

use std::sync::Arc;
use std::vec::Vec;

// TODO(oren): may want to pre-compute the bounding box

pub struct WfObject {
    pub triangles: HittableList,
    pub meshes: Vec<Arc<TriangleMesh>>,
    pub mat: Arc<dyn Material + Sync + Send>,
}

impl WfObject {
    #[must_use]
    pub fn new(fname: &str, scale: f64, default_mat: Arc<dyn Material + Sync + Send>) -> Self {
        let mut load_opts = tobj::OFFLINE_RENDERING_LOAD_OPTIONS;
        load_opts.triangulate = true;
        load_opts.single_index = true;
        let obj = tobj::load_obj(fname, &load_opts);
        assert!(obj.is_ok());
        let (models, mats_r) = obj.unwrap();
        assert!(mats_r.is_ok());
        // let mats = mats_r.unwrap();

        let mats = match mats_r {
            Ok(mats) => mats,
            Err(_) => vec![],
        };

        let materials: Vec<Arc<dyn Material + Sync + Send>> =
            mats.iter().map(|m| {
                Self::get_material(m)
            }).collect();

        let mut result = Self {
            meshes: vec![],
            triangles: HittableList::default(),
            mat: default_mat.clone(),
        };

        for m in &models {
            let mesh = &m.mesh;
            let mat = if let Some(m_id) = mesh.material_id {
                assert!(m_id <= materials.len());
                if m_id <= materials.len() {
                    materials[m_id].clone()
                } else {
                    default_mat.clone()
                }
            } else {
                default_mat.clone()
            };

            let n_faces = mesh.indices.len() / 3;
            let indices: Vec<usize> = mesh.indices.iter().map(|i| {
                *i as usize
            }).collect();

            let normals: Vec<Vec3> = mesh.normals.chunks(3).map(|n| {
                Vec3(n[0], n[1], n[2])
            }).collect();
            let uvs: Vec<(f64,f64)> = mesh.texcoords.chunks(2).map(|uv| {
                (uv[0], uv[1])
            }).collect();

            let positions: Vec<Point3> = mesh.positions.chunks(3).map(|p| {
                Point3(p[0] * scale, p[1] * scale, p[2] * scale)
            }).collect();

            let n_vertices = positions.len();

            result.meshes.push(Arc::new(TriangleMesh::new(
                n_faces, &indices,
                n_vertices, &positions,
                if !normals.is_empty() { Some(&normals) } else { None },
                None, // tangents
                if !uvs.is_empty() { Some(&uvs) } else { None },
                mat,
            )));

            result.triangles.add(HittableList::new(
                (0..n_faces).map(|i| {
                    Triangle::new(result.meshes.last().unwrap().clone(), i).into()
                }).collect()
            ).into());
        }
        result
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

        if let Some(tx) = &mm.diffuse_texture {
            let name = format!("data/purple_flower/{}", tx);
            return Lambertian::from_texture(
                texture::Image::new(&name).into()
            ).into();
        }

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
