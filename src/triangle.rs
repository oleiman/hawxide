use crate::vec3::{Vec3,Point3,dot,cross};
use crate::ray::Ray;
use crate::hit::{Hittable,HitRecord};
// use crate::material::Material;
use crate::aabb::AABB;
use crate::triangle_mesh::TriangleMesh;
use crate::onb::OrthoNormalBasis;

use std::sync::Arc;

pub struct Triangle {
    // pub norm: Vec3,
    pub mesh: Arc<TriangleMesh>,
    pub i_min: usize,
    pub vs: [usize; 3],
    // pub mat: Arc<dyn Material + Sync + Send>
}

impl Triangle {
    #[must_use]
    pub fn new(mesh: Arc<TriangleMesh>, t_idx: usize) -> Self {
        // TODO(oren): it would be nice to not store the normal, but I need it for other
        // stuff, right?
        let i_min = t_idx * 3;
        Self {
            vs: [
                mesh.vertex_indices[i_min],
                mesh.vertex_indices[i_min + 1],
                mesh.vertex_indices[i_min + 2],
            ],
            mesh, i_min,
        }
    }

    #[must_use]
    fn vertex(&self, i: usize) -> Point3 {
        self.mesh.p[self.vs[i]]
    }

    #[must_use]
    fn bary_to_cart(&self, u: f64, v: f64) -> Point3 {
        (1.0 - u - v)*self.vertex(0) + u * self.vertex(1) + v * self.vertex(2)
    }

    fn get_uvs(&self) -> [(f64,f64); 3] {
        if let Some(uv) = &self.mesh.uv {
            [
                uv[self.vs[0]],
                uv[self.vs[1]],
                uv[self.vs[2]],
            ]
        } else {
            [
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 1.1),
            ]
        }
    }

    fn pair_sub(a: (f64,f64), b: (f64,f64)) -> (f64,f64) {
        (a.0 - b.0, a.1 - b.1)
    }

    fn get_partial_derivatives(&self) -> (Vec3, Vec3) {
        let [uv0, uv1, uv2] = self.get_uvs();
        let duv02 = Self::pair_sub(uv0, uv2);
        let duv12 = Self::pair_sub(uv1, uv2);
        let dp02 = self.vertex(0) - self.vertex(2);
        let dp12 = self.vertex(1) - self.vertex(2);

        let determinant = duv02.0 * duv12.1 - duv02.1 * duv12.0;
        if determinant.abs() < 0.000_001 {
            let mut onb = OrthoNormalBasis::new();
            onb.build_from_w(cross(
                self.vertex(2) - self.vertex(0),
                self.vertex(1) - self.vertex(0)));
            (onb.u(), onb.v())
        } else {
            let invdet = 1.0 / determinant;
            (
                ( duv12.1 * dp02 - duv02.1 * dp12) * invdet,
                (-duv12.0 * dp02 + duv02.0 * dp12) * invdet,
            )
        }
    }
}

impl From<Triangle> for Arc<dyn Hittable + Sync + Send> {
    fn from(hh: Triangle) -> Arc<dyn Hittable + Sync + Send> {
        Arc::new(hh)
    }
}

impl Hittable for Triangle {
    #[allow(clippy::many_single_char_names)]
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let e1 = self.vertex(1) - self.vertex(0);
        let e2 = self.vertex(2) - self.vertex(0);
        let t_vec = r.origin - self.vertex(0);
        // let d_norm = r.dir.unit_vector();
        let d_norm = r.dir;

        let p_vec = cross(d_norm, e2);
        let q_vec = cross(t_vec, e1);

        let det = dot(p_vec, e1);

        // if we want to do culling, we would discard intersections on one side
        // i.e. discard if determinant is l.t. epsilon
        // if det.abs() < 0.000_001 {
        if det < 0.000_001 {
            return None;
        }

        let Vec3(t_hit, u, v) =
            (1.0 / det) *
            Vec3(dot(q_vec, e2), dot(p_vec, t_vec), dot(q_vec, r.dir));

        if u < 0.0 || v < 0.0 || u > 1.0 || u + v > 1.0 {
            return None;
        } else if t_hit < t_min || t_hit > t_max {
            return None;
        }

        let p_hit = self.bary_to_cart(u, v);

        let (dpdu, dpdv) = self.get_partial_derivatives();

        // TODO(oren): make sure normal is outward facing I guess? Also
        // I guess it should come from the normals array if possible?
        Some(HitRecord::with_dps(
            r, p_hit, cross(dpdu, dpdv).unit_vector(),
            t_hit, u, v, self.mesh.mat.clone(), dpdu, dpdv,
        ))

    }

    // Give the smallest reasonable AABB for the Hittable
    fn bounding_box(&self, _time0: f64, _time1: f64) -> Option<AABB> {
        let (a, b, c) = (self.vertex(0), self.vertex(1), self.vertex(2));

        let min_x = f64::min(
            f64::min(a.x(), b.x()),
            c.x()
        );
        let min_y = f64::min(
            f64::min(a.y(), b.y()),
            c.y()
        );
        let min_z = f64::min(
            f64::min(a.z(), b.z()),
            c.z()
        );

        let max_x = f64::max(
            f64::max(a.x(), b.x()),
            c.x());
        let max_y = f64::max(
            f64::max(a.y(), b.y()),
            c.y()
        );
        let max_z = f64::max(
            f64::max(a.z(), b.z()),
            c.z()
        );

        Some(AABB {
            min: Point3(min_x, min_y, min_z),
            max: Point3(max_x, max_y, max_z)
        })
    }

    // TODO(oren): pdf stuff...not needed unless we want to sample toward
    // a triangle, which I don't particularly care about right now
}
