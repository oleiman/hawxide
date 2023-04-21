
pub const INFINITY : f64 = f64::INFINITY;

pub fn degrees_to_radians(deg : f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

pub fn clamp(x : f64, min: f64, max: f64) -> f64 {
    match x {
        x if x < min => min,
        x if x > max => max,
        _ => x
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

    pub fn double() -> f64 {
        return unsafe {rng().gen::<f64>()};
    }

    // TODO(oren): could generate better randoms
    pub fn double_in_range(min: f64, max: f64) -> f64 {
        min + (max - min) * double()
    }
}
