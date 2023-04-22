#![allow(unused)]

use std::io;
use hawxide::*;

use std::io::{Write, BufWriter};
use std::boxed::Box;
use std::rc::Rc;

fn ray_color<H: Hittable>(r : &Ray, world: &H, depth: i32) -> Color {

    if depth <= 0 {
        return Color(0., 0., 0.);
    }
    if let Some(hr) = world.hit(r, 0.001, INFINITY) {
        // let target = hr.p + hr.norm + Vec3::random_unit_vector();
        // // let target = hr.p + Vec3::random_in_hemisphere(&hr.norm);
        // 0.5 * ray_color(&Ray{origin: hr.p, dir: target - hr.p}, world, depth - 1)
        if let Some((attenuation, scattered)) =  hr.mat.scatter(r, &hr) {
            attenuation * ray_color(&scattered, world, depth-1)
        } else {
            Color(0., 0., 0.)
        }
    } else {
        let unit_dir : Vec3 = r.dir.unit_vector();
        let t : f64 = 0.5 * (unit_dir.y() + 1.0);
        (1.0 - t) * Color(1., 1., 1.) + t * Color(0.5, 0.7, 1.0)
    }

    // match world.hit(r, 0.001, INFINITY) {
    //     Some(hr) => {
    //         // let target = hr.p + hr.norm + Vec3::random_unit_vector();
    //         // // let target = hr.p + Vec3::random_in_hemisphere(&hr.norm);
    //         // 0.5 * ray_color(&Ray{origin: hr.p, dir: target - hr.p}, world, depth - 1)
    //         match hr.mat.scatter(r, &hr) {
    //             Some((attenuation, scattered)) => {
    //                 attenuation * ray_color(&scattered, world, depth-1)
    //             }
    //             _ => Color(0., 0., 0.)
    //         }

    //     },
    //     _ => {
    //         let unit_dir : Vec3 = r.dir.unit_vector();
    //         let t : f64 = 0.5 * (unit_dir.y() + 1.0);
    //         (1.0 - t) * Color(1., 1., 1.) + t * Color(0.5, 0.7, 1.0)
    //     },
    // }
}

fn random_scene() -> HittableList {
    let mut world = HittableList::new();

    let ground_material: Rc<dyn Material> =
        Rc::new(Lambertian{albedo: Color(0.5, 0.5, 0.5)});
    world.add(Box::new(Sphere::new(
        &Point3(0., -1000., 0.),
        1000.,
        &ground_material
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random::double();
            let center = Point3(
                f64::from(a) + 0.9 * random::double(),
                0.2,
                f64::from(b) + 0.9 * random::double(),
            );

            if ((center - Point3(4., 0.2, 0.)).len() > 0.9) {
                world.add(match choose_mat {
                    i if i < 0.8 => {
                        Box::new(MovingSphere::new(
                            &center,
                            &(center + Vec3(0.0, random::double_in_range(0.0, 0.5), 0.0)),
                            0.0, 1.0, // times
                            0.2,      // radius
                            &(Rc::new(Lambertian{
                                albedo: Color::random() * Color::random(),
                            }) as Rc<dyn Material>)
                        ))
                    },
                    j if j < 0.95 => {
                        Box::new(Sphere::new(
                            &center, 0.2,
                            &(Rc::new(Metal{
                                albedo: Color::random_in_range(0.5, 1.),
                                fuzz: random::double_in_range(0., 0.5),
                            }) as Rc<dyn Material>)
                        ))
                    },
                    _ => {
                        Box::new(Sphere::new(
                            &center, 0.2,
                            &(Rc::new(Dielectric{
                                ir: 1.5,
                            }) as Rc<dyn Material>)
                        ))
                    }
                }
                );
            }
        }
    }

    world.add(Box::new(Sphere::new(
        &Point3(0., 1., 0.), 1.0,
        &(Rc::new(Dielectric{
            ir: 1.5,
        }) as Rc<dyn Material>)
    )));

    world.add(Box::new(Sphere::new(
        &Point3(-4., 1., 0.), 1.0,
        &(Rc::new(Lambertian{
            albedo: Color(0.4, 0.2, 0.1),
        }) as Rc<dyn Material>)
    )));

    world.add(Box::new(Sphere::new(
        &Point3(4., 1., 0.), 1.0,
        &(Rc::new(Metal{
            albedo: Color(0.7, 0.6, 0.5),
            fuzz: 0.0,
        }) as Rc<dyn Material>)
    )));

    world
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

    let lookfrom = Point3(13., 2., 3.);
    let lookat = Point3(0., 0., -0.);
    let vup = Vec3(0., 1., 0.);
    let fov = 20.0_f64;
    let dist_to_focus = 10.0;
    let aperture = 0.1_f64;

    let cam =
        Camera::new(&lookfrom, &lookat, &vup, fov,
                    ASPECT_RATIO, aperture, dist_to_focus, 0.0, 1.0);

    // Render

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    let mut stderr = BufWriter::new(std::io::stderr().lock());

    let material_ground: Rc<dyn Material> =
        Rc::new(Lambertian{
            albedo: Color(0.8, 0.8, 0.0),
        });
    let material_center: Rc<dyn Material> =
        Rc::new(Lambertian{
            albedo: Color(0.1, 0.2, 0.5),
        });
    let material_left: Rc<dyn Material> =
        Rc::new(Dielectric{
            ir: 1.5,
        });
    let material_right: Rc<dyn Material> =
        Rc::new(Metal{
            albedo: Color(0.8, 0.6, 0.2),
            fuzz: 0.0,
        });

    let mut world = HittableList::new();

    world.add(Box::new(Sphere::new(
        &Point3(0., -100.5, -1.),
        100.,
        &material_ground,
    )));

    world.add(Box::new(Sphere::new(
        &Point3(0., 0., -1.),
        0.5,
        &material_center,
    )));

    world.add(Box::new(Sphere::new(
        &Point3(-1., 0., -1.),
        0.5,
        &material_left,
    )));

    // Negative radius keeps same geometry but the surface normal is flipped.
    // this results in a sort of glass bubble thing

    world.add(Box::new(Sphere::new(
        &Point3(-1., 0., -1.),
        0.5,
        &material_left,
    )));

    world.add(Box::new(Sphere::new(
        &Point3(1., 0., -1.),
        0.5,
        &material_right,
    )));

    world = random_scene();

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
                    (f64::from(i) + random::double()) / f64::from(IMAGE_WIDTH - 1);
                let v : f64 =
                    (f64::from(j) + random::double()) / f64::from(IMAGE_HEIGHT - 1);
                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }
            write_color(&mut stdout, &pixel_color, SAMPLES_PER_PIX);
        }
    }

    write!(stderr, "\nDone\n");

}
