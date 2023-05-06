
pub const INFINITY : f64 = f64::INFINITY;
pub const NEG_INFINITY : f64 = f64::NEG_INFINITY;
pub static PI : f64 = std::f64::consts::PI;

pub fn degrees_to_radians(deg : f64) -> f64 {
    let d = if deg < 0.0 {
        deg + 360.0
    } else {
        deg
    };

    d * std::f64::consts::PI / 180.0
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

    static mut RNG : Option<ThreadRng> = None;

    unsafe fn rng() ->  &'static mut ThreadRng {
        if RNG.is_none() {
            RNG = Some(thread_rng());
        }

        RNG.as_mut().unwrap()
    }

    pub fn double() -> f64 {
        return unsafe {rng().gen_range(0.0..1.0)};
    }

    // TODO(oren): could generate better randoms
    pub fn double_range(min: f64, max: f64) -> f64 {
        return unsafe {rng().gen_range(min..max)}
        // min + (max - min) * double()
    }

    pub fn int(min: i32, max: i32) -> i32 {
        return unsafe {rng().gen_range(min..=max)};
        // (double_range(f64::from(min), f64::from(max + 1))) as i32
    }

    pub fn uint(min: usize, max: usize) -> usize {
        return unsafe {rng().gen_range(min..=max)};
        // (double_range(min as f64, (max + 1) as f64)) as usize
    }
}
