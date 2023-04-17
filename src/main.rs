#![allow(unused)]

use hawxide::{vec3, Vec3, vec3::dot};
use hawxide::util::*;

fn main() {
    println!("Hello, world!");

    let v = Vec3(1., 2., 3.);

    println!("{}", v);

    let mut u = Vec3::random_unit_vector();

    println!("random {} len {}", u, u.len());

    u = -v;
    println!("{}", u);

    u += v;

    println!("{}", u);

    u = Vec3(2., 3., 4.);

    u *= 2.;

    println!("{}", u);

    u /= 2.;

    println!("{}", u);
    println!("{}.len(): {:.4}, nz: {}", v, v.len(), v.near_zero());

    let w = v - (3 * v);
    println!("{} -> {}", w, w.unit_vector());
    println!("{}", w.unit_vector().len());
    println!("w.w: {}", dot(&w, &w));

}
