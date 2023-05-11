
use crate::vec3::{Point3, Vec3, dot};
use crate::util::random;

const  POINT_COUNT : usize = 256;

pub struct Perlin {
    ranvec: [Vec3; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    pub fn new() -> Perlin {
        let ranvec =
            std::array::from_fn(
                |_| Vec3::random_range(-1., 1.).unit_vector()
            );
        let perm_x = Self::perlin_generate_perm();
        let perm_y = Self::perlin_generate_perm();
        let perm_z = Self::perlin_generate_perm();
        Perlin{
            ranvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    pub fn turb(&self, p: Point3, depth: Option<i32>) -> f64 {
        static DEFAULT_DEPTH : i32 = 7;
        let mut accum = 0.0f64;
        let mut tmp_p = p;
        let mut weight = 1.0f64;

        for _ in 0..depth.unwrap_or(DEFAULT_DEPTH) {
            accum += weight * self.smooth_noise(tmp_p);
            weight *= 0.5;
            tmp_p *= 2.;
        }

        accum.abs()
    }

    pub fn smooth_noise(&self, p: Point3) -> f64 {
        let u = p.x() - f64::floor(p.x());
        let v = p.y() - f64::floor(p.y());
        let w = p.z() - f64::floor(p.z());

        // u = u*u * (3.0 - 2.0*u);
        // v = v*v * (3.0 - 2.0*v);
        // w = w*w * (3.0 - 2.0*w);

        let i = p.x().floor() as i32;
        let j = p.y().floor() as i32;
        let k = p.z().floor() as i32;

        let mut c: [[[Vec3; 2]; 2]; 2] = [[[Vec3(0.,0.,0.); 2]; 2]; 2];
        for di in 0..2i32 {
            for dj in 0..2i32 {
                for dk in 0..2i32 {
                    c[di as usize][dj as usize][dk as usize] = self.ranvec[
                            self.perm_x[(i+di) as usize & 0xFF] ^
                            self.perm_y[(j+dj) as usize & 0xFF] ^
                            self.perm_z[(k+dk) as usize & 0xFF]
                    ]
                }
            }
        }

        Self::perlin_interp(&c, u, v, w)
    }

    fn perlin_interp(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u*u * (3.0 - 2.0*u);
        let vv = v*v * (3.0 - 2.0*v);
        let ww = w*w * (3.0 - 2.0*w);
        let mut accum = 0.0f64;
        for i in 0..2u8 {
            for j in 0..2u8 {
                for k in 0..2u8 {
                    let fv = Vec3(i as f64, j as f64, k as f64);
                    let weight_v = Vec3(u - fv.0, v - fv.1, w - fv.2);
                    accum +=
                        (fv.0 * uu + f64::from(1-i)*(1.-uu)) *
                        (fv.1 * vv + f64::from(1-j)*(1.-vv)) *
                        (fv.2 * ww + f64::from(1-k)*(1.-ww)) *
                        dot(c[i as usize][j as usize][k as usize], weight_v);
                }
            }
        }
        accum
    }

    #[allow(dead_code)]
    fn trilinear_interp(c: &[[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0f64;
        for i in 0..2u8 {
            for j in 0..2u8 {
                for k in 0..2u8 {
                    let fv = Vec3(i as f64, j as f64, k as f64);
                    accum +=
                        (fv.0 * u + f64::from(1-i)*(1.-u)) *
                        (fv.1 * v + f64::from(1-j)*(1.-v)) *
                        (fv.2 * w + f64::from(1-k)*(1.-w)) *
                        c[i as usize][j as usize][k as usize];
                }
            }
        }
        accum
    }

    fn perlin_generate_perm() -> [usize; POINT_COUNT] {
        let mut iter = 0..POINT_COUNT;
        let mut result: [usize; POINT_COUNT] =
            std::array::from_fn(|_| iter.next().expect("too short"));
        Self::permute(&mut result);
        result
    }

    fn permute(arr: &mut [usize; POINT_COUNT]) {
        for i in (0..arr.len()).rev() {
            let target = random::uint(0, i);
            arr.swap(i, target);
        }
    }
}
