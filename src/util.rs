
pub const INFINITY : f64 = f64::INFINITY;
pub const PI : f64 = 3.1415926535897932385;

pub fn degrees_to_radians(deg : f64) -> f64 {
    return deg * PI / 180.0;
}

pub fn clamp(x : f64, min: f64, max: f64) -> f64 {
    if x < min {
        return min;
    } else if x > max {
        return max;
    } else {
        return x;
    }
}

pub mod random {
    use rand::prelude::*;
    // use rand::{Rng, thread_rng, rngs::ThreadRng};

    static mut RNG : Option<ThreadRng> = None;

    unsafe fn rng() ->  &'static mut ThreadRng {
        if RNG.is_none() {
            RNG = Some(thread_rng());
        }

        RNG.as_mut().unwrap()
    }

    pub fn random_double() -> f64 {
        return unsafe {rng().gen::<f64>()};
    }

    // TODO(oren): could generate better randoms
    pub fn random_double_in_range(min: f64, max: f64) -> f64 {
        return min + (max - min) * random_double();
    }
}
