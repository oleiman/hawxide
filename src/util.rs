
pub const INFINITY : f64 = f64::INFINITY;
pub const NEG_INFINITY : f64 = f64::NEG_INFINITY;
pub static PI : f64 = std::f64::consts::PI;

#[must_use]
pub fn degrees_to_radians(deg : f64) -> f64 {
    let d = if deg < 0.0 {
        deg + 360.0
    } else {
        deg
    };

    d * std::f64::consts::PI / 180.0
}

#[must_use]
pub fn clamp(x : f64, min: f64, max: f64) -> f64 {
    match x {
        x if x < min => min,
        x if x > max => max,
        _ => x
    }
}

pub mod random {
    use crate::vec3::Vec3;
    use crate::util::PI;

    use rand::prelude::*;

    static mut RNG : Option<ThreadRng> = None;

    unsafe fn rng() ->  &'static mut ThreadRng {
        if RNG.is_none() {
            RNG = Some(thread_rng());
        }

        RNG.as_mut().unwrap()
    }

    #[must_use]
    pub fn double() -> f64 {
        return unsafe {rng().gen_range(0.0..1.0)};
    }

    #[must_use]
    // TODO(oren): could generate better randoms
    pub fn double_range(min: f64, max: f64) -> f64 {
        return unsafe {rng().gen_range(min..max)}
    }

    #[must_use]
    pub fn int(min: i32, max: i32) -> i32 {
        return unsafe {rng().gen_range(min..=max)};
    }

    #[must_use]
    pub fn uint(min: usize, max: usize) -> usize {
        return unsafe {rng().gen_range(min..=max)};
    }

    #[must_use]
    pub fn cosine_direction() -> Vec3 {
        let r1 = double();
        let r2 = double();
        let z = f64::sqrt(1.0 - r2);

        let phi = 2.0 * PI * r1;
        let x = f64::cos(phi) * f64::sqrt(r2);
        let y = f64::sin(phi) * f64::sqrt(r2);

        Vec3(x, y, z)
    }
}
