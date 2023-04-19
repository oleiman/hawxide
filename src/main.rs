#![allow(unused)]

use std::io;
use hawxide::*;

use std::io::{Write, BufWriter};
use std::boxed::Box;

fn ray_color<H: Hittable>(r : &Ray, world: &H) -> Color {
    match world.hit(r, 0., INFINITY) {
        Some(hr) => {
            0.5 * (hr.norm + Color(1.,1.,1.))
        },
        _ => {
            let unit_dir : Vec3 = r.dir.unit_vector();
            let t : f64 = 0.5 * (unit_dir.y() + 1.0);
            (1.0 - t) * Color(1., 1., 1.) + t * Color(0.5, 0.7, 1.0)
        },
    }
}

fn main() {

    // Image

    const ASPECT_RATIO : f64 = 16.0 / 9.0;
    const IMAGE_WIDTH : i32 = 400;
    const IMAGE_HEIGHT : i32 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as i32;

    // Camera

    const VIEWPORT_HEIGHT : f64 = 2.0;
    const VIEWPORT_WIDTH : f64 = ASPECT_RATIO * VIEWPORT_HEIGHT as f64;
    const FOCAL_LENGTH : f64 = 1.0;

    const ORIGIN : Point3 = Point3(0., 0., 0.);
    const HORIZONTAL : Vec3 = Vec3(VIEWPORT_WIDTH, 0., 0.);
    const VERTICAL : Vec3 = Vec3(0., VIEWPORT_HEIGHT, 0.);
    let lower_left : Point3 =
        ORIGIN - (HORIZONTAL / 2) - (VERTICAL / 2) - Vec3(0., 0., FOCAL_LENGTH);

    // Render

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    let mut stderr = BufWriter::new(std::io::stderr().lock());

    let mut world = HittableList::new();
    world.add(Box::new(Sphere{
        center: Point3(0., 0., -1.),
        radius: 0.5,
    }));
    world.add(Box::new(Sphere{
        center: Point3(0., -100.5, -1.),
        radius: 100.,
    }));

    writeln!(stdout, "P3");
    writeln!(stdout, "{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    writeln!(stdout, "255");

    for j in (0..IMAGE_HEIGHT).rev() {
        // TODO(oren): write with flush?
        write!(stderr, "\rScanlines remaining: {} ", j);
        for i in 0..IMAGE_WIDTH {
            let u : f64 = (i as f64) / ((IMAGE_WIDTH - 1) as f64);
            let v : f64 = (j as f64) / ((IMAGE_HEIGHT - 1) as f64);
            let r : Ray = Ray {
                origin: ORIGIN,
                dir: lower_left + (u * HORIZONTAL) + (v * VERTICAL) - ORIGIN,
            };
            let pixel_color : Color = ray_color(&r, &world);
            write_color(&pixel_color, &mut stdout);
        }
    }

    write!(stderr, "\nDone\n");

}
