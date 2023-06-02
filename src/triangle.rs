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
        let vs = [
            mesh.vertex_indices[i_min],
            mesh.vertex_indices[i_min + 1],
            mesh.vertex_indices[i_min + 2],
        ];
        Self {
            vs,
            mesh, i_min,
        }
    }

    #[must_use]
    fn vertex(&self, i: usize) -> Point3 {
        self.mesh.p[self.vs[i]]
    }

    #[must_use]
    fn bary_to_cart(&self, u: f64, v: f64) -> Point3 {
        (1.0 - u - v) * self.vertex(0) + u * self.vertex(1) + v * self.vertex(2)
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
                (1.0, 1.0),
            ]
        }
    }

    fn pair_sub(a: (f64,f64), b: (f64,f64)) -> (f64,f64) {
        (a.0 - b.0, a.1 - b.1)
    }

    fn get_partial_derivatives(&self) -> (Vec3, Vec3, Vec3) {
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
            (onb.u(), onb.v(), cross(onb.u(), onb.v()).unit_vector())
        } else {
            let invdet = 1.0 / determinant;
            (
                ( duv12.1 * dp02 - duv02.1 * dp12) * invdet,
                (-duv12.0 * dp02 + duv02.0 * dp12) * invdet,
                cross(dp02, dp12).unit_vector(),
            )
        }
    }

    fn compute_shading_normals(&self, b: (f64, f64, f64)) -> Option<Vec3> {
        if let Some(norms) = &self.mesh.n {
            Some(
                (b.0 * norms[self.vs[0]] +
                 b.1 * norms[self.vs[1]] +
                 b.2 * norms[self.vs[2]]).unit_vector()
            )
        } else {
            None
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
        // NOTE(oren): doesn't seem to be much of a performance hit either way, so
        // may as well keep everything I guess? 
        if det.abs() < 0.000_001 {
        // if det < 0.000_001 {
            return None;
        }

        let inv_det = 1.0 / det;

        let Vec3(t_hit, b1, b2) =
            inv_det *
            Vec3(dot(q_vec, e2), dot(p_vec, t_vec), dot(q_vec, r.dir));

        let b0 = 1.0 - b1 - b2;

        if b1 < 0.0 || b2 < 0.0 || b1 > 1.0 || b1 + b2 > 1.0 {
            return None;
        } else if t_hit < t_min || t_hit > t_max {
            return None;
        }

        let p_hit = self.bary_to_cart(b1, b2);

        // duplicate, waste
        let [uv0, uv1, uv2] = self.get_uvs();

        let (dpdu, dpdv, norm) = self.get_partial_derivatives();

        // cross(dpdu, dpdv).unit_vector()

        let (uhit, vhit) = (
            (b0 * uv0.0 + b1 * uv1.0 + b2 * uv2.0),
            (b0 * uv0.1 + b1 * uv1.1 + b2 * uv2.1),
        );

        let mut hr = HitRecord::with_dps(
            r, p_hit, norm,
            t_hit, uhit, vhit, self.mesh.mat.clone(), dpdu, dpdv,
        );

        // assert!(self.mesh.n.is_none());

        if let Some(sn) = self.compute_shading_normals((b0,b1,b2)) {
            hr.shading_geo.n = sn;
        };

        Some(hr)
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

        // Adjust the bounding box to account for situations where the triangle is
        // alligned to some axis. More efficient approach would be to subtract off a
        // fraction of the geometric normal, but we're not storing that currently
        // and the calculation has some cost; might be a wash.
        Some(AABB {
            min: Point3(min_x, min_y, min_z) - Point3(0.000_001, 0.000_001, 0.000_001),
            max: Point3(max_x, max_y, max_z) + Point3(0.000_001, 0.000_001, 0.000_001),
        })
    }

    // TODO(oren): pdf stuff...not needed unless we want to sample toward
    // a triangle, which I don't particularly care about right now
}
