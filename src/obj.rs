use crate::vec3::{Point3, Color, Vec3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::triangle::Triangle;
use crate::hittable_list::HittableList;
use crate::bvh::BVHNode;
use crate::material::{Metal, Lambertian, WfMtl};
use crate::triangle_mesh::TriangleMesh;
use crate::texture::Texture;
use crate::texture;

use tobj;

use std::sync::Arc;
use std::vec::Vec;
use std::path::Path;

// TODO(oren): may want to pre-compute the bounding box

pub struct WfObject {
    pub triangles: HittableList,
    pub meshes: Vec<Arc<TriangleMesh>>,
    pub mat: Arc<dyn Material + Sync + Send>,
}

impl WfObject {
    #[must_use]
    pub fn new<P: AsRef<Path>>(fname: P, scale: f64,
                               default_mat: Arc<dyn Material + Sync + Send>) -> Self {
        let mut load_opts = tobj::OFFLINE_RENDERING_LOAD_OPTIONS;
        load_opts.triangulate = true;
        load_opts.single_index = true;
        let obj = tobj::load_obj(fname.as_ref(), &load_opts);
        assert!(obj.is_ok());
        let (models, mats_r) = obj.unwrap();

        let mats = match mats_r {
            Ok(mats) => mats,
            Err(_) => vec![],
        };

        let materials: Vec<Arc<dyn Material + Sync + Send>> =
            mats.iter().map(|m| {
                Self::get_material(m, fname.as_ref().parent())
            }).collect();

        let mut result = Self {
            meshes: vec![],
            triangles: HittableList::default(),
            mat: default_mat.clone(),
        };

        let mut n_total: usize = 0;
        let mut v_total: usize = 0;

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

            n_total += n_faces;
            v_total += n_vertices;

            result.meshes.push(Arc::new(TriangleMesh::new(
                n_faces, &indices,
                n_vertices, &positions,
                if !normals.is_empty() { Some(&normals) } else { None },
                None, // tangents
                if !uvs.is_empty() { Some(&uvs) } else { None },
                mat,
            )));

            result.triangles.add(BVHNode::new(
                &HittableList::new(
                    (0..n_faces).map(|i| {
                        Triangle::new(result.meshes.last().unwrap().clone(), i).into()
                    }).collect()
                ), 0.0, 1.0
            ).into());
        }
        eprintln!("{}: N: {}, V: {}, models: {}",
                  fname.as_ref().display(), n_total, v_total, models.len());
        result
    }

    fn get_texture(fname: &Option<String>,
                   dir: &Option<&Path>,
                   k: Color) -> Arc<dyn Texture + Sync + Send> {
        if let Some(tx) = &fname {
            let name = dir.unwrap_or(Path::new(".")).join(tx);
            texture::Image::with_k(name.as_path(), k).into()
        } else {
            texture::SolidColor::new(k).into()
        }
    }

    fn get_material (mm: &tobj::Material, dir: Option<&Path>) -> Arc<dyn Material + Sync + Send> {
        let k_d = mm.diffuse.unwrap_or([0.8, 0.8, 0.8]).into();
        let k_s = mm.specular.unwrap_or([1.0, 1.0, 1.0]).into();
        let k_a = mm.ambient.unwrap_or([0.2, 0.2, 0.2]).into();
        let ns = mm.shininess.unwrap_or(0.0);

        // TODO(oren): should multiply by the Kd value
        let diffuse: Arc<dyn Texture + Sync + Send> =
            Self::get_texture(&mm.diffuse_texture, &dir, k_d);

        let specular: Arc<dyn Texture + Sync + Send> =
            Self::get_texture(&mm.specular_texture, &dir, k_s);

        let ambient: Arc<dyn Texture + Sync + Send> =
            Self::get_texture(&mm.ambient_texture, &dir, k_a);

        let model = if let Some(m) = mm.illumination_model { m } else { 1 };

        WfMtl::new(
            model, ns, diffuse, specular, ambient,
        ).into()
    }
}

impl From<WfObject> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: WfObject) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
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
