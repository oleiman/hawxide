#![allow(unused)]

use std::io;
use hawxide::*;

use std::io::{Write, BufWriter};
use std::boxed::Box;

fn ray_color<H: Hittable>(r : &Ray, world: &H, depth: i32) -> Color {

    if depth <= 0 {
        return Color(0., 0., 0.);
    }
    match world.hit(r, 0.001, INFINITY) {
        Some(hr) => {
            // let target = hr.p + hr.norm + Vec3::random_unit_vector();
            // // let target = hr.p + Vec3::random_in_hemisphere(&hr.norm);
            // 0.5 * ray_color(&Ray{origin: hr.p, dir: target - hr.p}, world, depth - 1)
            match hr.mat.scatter(r, &hr) {
                Some((attenuation, scattered)) => {
                    attenuation * ray_color(&scattered, world, depth-1)
                }
                _ => Color(0., 0., 0.)
            }

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
    const SAMPLES_PER_PIX : i32 = 100;
    const MAX_DEPTH : i32 = 50;

    // Camera

    const VIEWPORT_HEIGHT : f64 = 2.0;
    const FOCAL_LENGTH : f64 = 1.0;

    let lookfrom = Point3(3., 3., 2.);
    let lookat = Point3(0., 0., -1.);
    let vup = Vec3(0., 1., 0.);
    let fov = 20.0 as f64;
    let dist_to_focus = (lookfrom - lookat).len();
    let aperture = 2.0 as f64;

    let cam =
        Camera::new(&lookfrom, &lookat, &vup, fov, ASPECT_RATIO, aperture, dist_to_focus);

    // Render

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    let mut stderr = BufWriter::new(std::io::stderr().lock());

    static MATERIAL_GROUND : Lambertian = Lambertian{albedo: Color(0.8, 0.8, 0.0)};
    static MATERIAL_CENTER : Lambertian = Lambertian{albedo: Color(0.1, 0.2, 0.5)};
    static MATERIAL_LEFT : Dielectric = Dielectric{ir: 1.5};
    static MATERIAL_RIGHT : Metal = Metal{
        albedo: Color(0.8, 0.6, 0.2),
        fuzz: 0.0,
    };

    static MATERIAL_BLUE : Lambertian = Lambertian{albedo: Color(0., 0., 1.)};
    static MATERIAL_RED : Lambertian = Lambertian{albedo: Color(1., 0., 0.)};

    let mut world = HittableList::new();

    world.add(Box::new(Sphere{
        center: Point3(0., -100.5, -1.),
        radius: 100.,
        mat: &MATERIAL_GROUND,
    }));

    world.add(Box::new(Sphere{
        center: Point3(0., 0., -1.),
        radius: 0.5,
        mat: &MATERIAL_CENTER,
    }));

    world.add(Box::new(Sphere{
        center: Point3(-1.0, 0.0, -1.0),
        radius: 0.5,
        mat: &MATERIAL_LEFT,
    }));

    // Negative radius keeps same geometry but the surface normal is flipped.
    // this results in a sort of glass bubble thing
    world.add(Box::new(Sphere{
        center: Point3(-1.0, 0.0, -1.0),
        radius: -0.45,
        mat: &MATERIAL_LEFT,
    }));

    world.add(Box::new(Sphere{
        center: Point3(1.0, 0.0, -1.0),
        radius: 0.5,
        mat: &MATERIAL_RIGHT,
    }));


    writeln!(stdout, "P3");
    writeln!(stdout, "{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    writeln!(stdout, "255");

    for j in (0..IMAGE_HEIGHT).rev() {
        write!(stderr, "\rScanlines remaining: {} ", j);
        stderr.flush();
        for i in 0..IMAGE_WIDTH {
            let mut pixel_color = Color(0., 0., 0.);
            for s in 0..SAMPLES_PER_PIX {
                let u : f64 =
                    (i as f64 + random::random_double()) / ((IMAGE_WIDTH - 1) as f64);
                let v : f64 =
                    (j as f64 + random::random_double()) / ((IMAGE_HEIGHT - 1) as f64);
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }
            write_color(&mut stdout, &pixel_color, SAMPLES_PER_PIX);
        }
    }

    write!(stderr, "\nDone\n");

}
