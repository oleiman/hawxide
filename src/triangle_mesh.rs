use crate::vec3::{Vec3,Point3};
use crate::ray::Ray;
use crate::hit::{HitRecord,Hittable};
use crate::material::Material;
use crate::aabb::AABB;
use crate::util::PI;

use std::sync::Arc;
use std::vec::Vec;

pub struct TriangleMesh {
    pub n_faces: usize,
    pub n_vertices: usize,
    pub vertex_indices: Vec<usize>,
    pub p: Vec<Point3>,
    pub n: Option<Vec<Vec3>>,
    pub s: Option<Vec<Vec3>>,
    pub uv: Option<Vec<(f64, f64)>>,
    pub mat: Arc<dyn Material + Sync + Send>,
    // TODO(oren): may want to add alpha mask here, but I think maybe
    // that can be part of some custom material?
}

impl TriangleMesh {
    pub fn new(n_faces: usize, vertex_indices: &[usize],
               n_vertices: usize, p: &[Point3],
               n: Option<&[Vec3]>, s: Option<&[Vec3]>, uv: Option<&[(f64,f64)]>,
               mat: Arc<dyn Material + Sync + Send>
    ) -> Self {
        assert!(vertex_indices.len() >= n_faces * 3,
                "vertex_indices.len(): {}", vertex_indices.len());
        assert!(p.len() == n_vertices, "p.len(): {}", p.len());
        let n = if let Some(n) = n {
            assert!(n.len() == n_vertices, "n.len(): {}", n.len());
            Some(n.to_vec())
        } else {
            None
        };
        let s = if let Some(s) = s {
            assert!(s.len() == n_vertices, "s.len(): {}", s.len());
            Some(s.to_vec())
        } else {
            None
        };
        let uv = if let Some(uv) = uv {
            assert!(uv.len() == n_vertices, "uv.len(): {}", uv.len());
            Some(uv.to_vec())
        } else {
            None
        };
        Self {
            n_faces,
            vertex_indices: vertex_indices.to_vec(),
            n_vertices, p: p.to_vec() , n, s, uv,
            mat,
        }
    }
}
