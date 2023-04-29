#![allow(unused)]

use std::io;
use hawxide::*;

use std::io::{Write, BufWriter};
use std::rc::Rc;

fn ray_color<H: Hittable>(r : &Ray, background: &Color, world: &H, depth: i32) -> Color {
    if depth <= 0 {
        return Color(0., 0., 0.);
    }
    if let Some(hr) = world.hit(r, 0.001, INFINITY) {
        let emitted = hr.mat.emitted(hr.u, hr.v, &hr.p);
        if let Some((attenuation, scattered)) =  hr.mat.scatter(r, &hr) {
            emitted + attenuation * ray_color(&scattered, background, world, depth-1)
        } else {
            emitted
        }
    } else {
        *background
    }
}

fn random_scene() -> HittableList {
    let mut world = HittableList::new();

    let checker  = Rc::new(
        CheckerTexture::new(&Color(0.2, 0.3, 0.1), &Color(0.9, 0.9, 0.9)));

    let ground_material: Rc<dyn Material> =
        Rc::new(Lambertian{albedo: checker.clone()});
    world.add(Rc::new(Sphere::new(
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
                        Rc::new(MovingSphere::new(
                            &center,
                            &(center + Vec3(0.0, random::double_range(0.0, 0.5), 0.0)),
                            0.0, 1.0, // times
                            0.2,      // radius
                            &(Rc::new(
                                Lambertian::new(&(Color::random() * Color::random())),
                            ) as Rc<dyn Material>)
                        ))
                    },
                    j if j < 0.95 => {
                        Rc::new(Sphere::new(
                            &center, 0.2,
                            &(Rc::new(Metal{
                                albedo: Color::random_range(0.5, 1.),
                                fuzz: random::double_range(0., 0.5),
                            }) as Rc<dyn Material>)
                        ))
                    },
                    _ => {
                        Rc::new(Sphere::new(
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

    world.add(Rc::new(Sphere::new(
        &Point3(0., 1., 0.), 1.0,
        &(Rc::new(Dielectric{
            ir: 1.5,
        }) as Rc<dyn Material>)
    )));

    world.add(Rc::new(Sphere::new(
        &Point3(-4., 1., 0.), 1.0,
        &(Rc::new(
            Lambertian::new(&Color(0.4, 0.2, 0.1))
        ) as Rc<dyn Material>)
    )));

    world.add(Rc::new(Sphere::new(
        &Point3(4., 1., 0.), 1.0,
        &(Rc::new(Metal{
            albedo: Color(0.7, 0.6, 0.5),
            fuzz: 0.0,
        }) as Rc<dyn Material>)
    )));

    world
}

fn two_spheres() -> HittableList {
    let checker = Rc::new(CheckerTexture::new(
        &Color(0.2, 0.3, 0.1),
        &Color(0.9, 0.9, 0.9),
    ));

    HittableList {
        objects: vec![
            Rc::new(Sphere::new(
                &Point3(0.0, -10.0, 0.0),
                10.0,
                &(Rc::new(Lambertian {
                    albedo: checker.clone()
                }) as Rc<dyn Material>),
            )),
            Rc::new(Sphere::new(
                &Point3(0.0, 10.0, 0.0),
                10.0,
                &(Rc::new(Lambertian {
                    albedo: checker.clone()
                }) as Rc<dyn Material>),
            )),
        ],
    }
}

fn two_perlin_spheres() -> HittableList {
    let pertext = Rc::new(NoiseTexture::new(4.));

    HittableList {
        objects: vec![
            Rc::new(Sphere::new(
                &Point3(0.0, -1000.0, 0.0),
                1000.0,
                &(Rc::new(Lambertian {
                    albedo: pertext.clone(),
                }) as Rc<dyn Material>),
            )),
            Rc::new(Sphere::new(
                &Point3(0.0, 2.0, 0.0),
                2.0,
                &(Rc::new(Lambertian {
                    albedo: pertext.clone(),
                }) as Rc<dyn Material>),
            )),
        ]
    }
}

fn earth() -> HittableList {

    let earth_texture = Rc::new(ImageTexture::new("earthmap.jpg"));

    HittableList {
        objects: vec![
            Rc::new(Sphere::new(
                &Point3(0.0, 0.0, 0.0),
                2.0,
                &(Rc::new(Lambertian {
                    albedo: earth_texture.clone(),
                }) as Rc<dyn Material>)
            )),
        ],
    }
}

fn simple_light() -> HittableList {
    let pertext = Rc::new(NoiseTexture::new(4.));
    // let difflight = Rc::new(DiffuseLight::new(&Color(4., 4., 4.)));
    let difflight = Rc::new(DiffuseLight::new(&Color(7., 7., 7.)));

    HittableList {
        objects: vec![
            Rc::new(Sphere::new(
                &Point3(0.0, -1000.0, 0.0),
                1000.0,
                &(Rc::new(Lambertian {
                    albedo: pertext.clone(),
                }) as Rc<dyn Material>),
            )),
            Rc::new(Sphere::new(
                &Point3(0.0, 2.0, 0.0),
                2.0,
                &(Rc::new(Lambertian {
                    albedo: pertext.clone(),
                }) as Rc<dyn Material>),
            )),
            Rc::new(AARect::xy_rect(
                3.0, 5.0, 1.0, 3.0, -2.0,
                &(difflight.clone() as Rc<dyn Material>),
            )),
        ]
    }
}

fn cornell_box() -> HittableList {
    let red = Rc::new(Lambertian::new(&Color(0.65, 0.05, 0.05)));
    let white = Rc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let green = Rc::new(Lambertian::new(&Color(0.12, 0.45, 0.15)));
    let light = Rc::new(DiffuseLight::new(&Color(15.0, 15.0, 15.0)));
    // let light = Rc::new(DiffuseLight::new(&Color(7.0, 7.0, 7.0)));

    let mut box1 : Rc<dyn Hittable> = Rc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 330.0, 165.0),
        &(white.clone() as Rc<dyn Material>),
    ));

    box1 = Rc::new(Rotate::rotate_y(&box1, 15.0));
    box1 = Rc::new(Translate::new(&box1, &Vec3(265.0, 0.0, 295.0)));

    let mut box2 : Rc<dyn Hittable> = Rc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 165.0, 165.0),
        &(white.clone() as Rc<dyn Material>),
    ));

    box2 = Rc::new(Rotate::rotate_y(&box2, -18.0));
    // box2 = Rc::new(Rotate::rotate_x(&box2, 15.0));
    // box2 = Rc::new(Rotate::rotate_z(&box2, 15.0));

    box2 = Rc::new(Translate::new(&box2, &Vec3(130.0, 0.0, 65.0)));

    HittableList {
        objects: vec![
            Rc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, &(green.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, &(red.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xz_rect(
                213.0, 343.0, 227.0, 332.0, 554.0, &(light.clone() as Rc<dyn Material>)
            )),
            // Rc::new(AARect::xz_rect(
            //     113.0, 443.0, 127.0, 432.0, 554.0, &(light.clone() as Rc<dyn Material>)
            // )),
            Rc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, &(white.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, &(white.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xy_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, &(white.clone() as Rc<dyn Material>)
            )),
            box1,
            box2,
        ]
    }
}

fn cornell_smoke() -> HittableList {
    let red = Rc::new(Lambertian::new(&Color(0.65, 0.05, 0.05)));
    let white = Rc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let green = Rc::new(Lambertian::new(&Color(0.12, 0.45, 0.15)));
    let light = Rc::new(DiffuseLight::new(&Color(7.0, 7.0, 7.0)));

    let mut box1 : Rc<dyn Hittable> = Rc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 330.0, 165.0),
        &(white.clone() as Rc<dyn Material>),
    ));

    box1 = Rc::new(Rotate::rotate_y(&box1, 15.0));
    box1 = Rc::new(Translate::new(&box1, &Vec3(265.0, 0.0, 295.0)));

    let mut box2 : Rc<dyn Hittable> = Rc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 165.0, 165.0),
        &(white.clone() as Rc<dyn Material>),
    ));

    box2 = Rc::new(Rotate::rotate_y(&box2, -18.0));
    // box2 = Rc::new(Rotate::rotate_x(&box2, 18.0));
    // box2 = Rc::new(Rotate::rotate_z(&box2, 18.0));

    box2 = Rc::new(Translate::new(&box2, &Vec3(130.0, 0.0, 65.0)));

    HittableList {
        objects: vec![
            Rc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, &(green.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, &(red.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xz_rect(
                113.0, 443.0, 127.0, 432.0, 554.0, &(light.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, &(white.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, &(white.clone() as Rc<dyn Material>)
            )),
            Rc::new(AARect::xy_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, &(white.clone() as Rc<dyn Material>)
            )),
            Rc::new(ConstantMedium::new(&box1, 0.01, &Color::new())),
            Rc::new(ConstantMedium::new(&box2, 0.01, &Color(1.0, 1.0, 1.0))),
        ]
    }
}


fn main() {

    // Image

    const MAX_DEPTH : i32 = 50;

    // Camera

    const VIEWPORT_HEIGHT : f64 = 2.0;
    const FOCAL_LENGTH : f64 = 1.0;

    let mut aspect_ratio : f64 = 16.0 / 9.0;
    let mut image_width : i32 = 400;
    let mut samples_per_pixel : i32 = 400;
    let mut lookfrom = Point3(13., 2., 3.);
    let mut lookat = Point3(0., 0., -0.);
    let mut vfov = 20.0_f64;
    let mut aperture = 0.0;
    let mut background = Color(0.0, 0.0, 0.0);

    let scene_select : usize = 0;

    let world = BVHNode::new( &match scene_select {
        1 => {
            background = Color(0.7, 0.8, 1.0);
            lookfrom = Point3(13.0, 2.0, 3.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 20.0;
            aperture = 0.1;
            random_scene()
        },
        2 => {
            background = Color(0.7, 0.8, 1.0);
            lookfrom = Point3(13.0, 2.0, 3.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 20.0;
            two_spheres()
        },
        3 => {
            background = Color(0.7, 0.8, 1.0);
            lookfrom = Point3(13.0, 2.0, 3.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 20.0;
            two_perlin_spheres()
        },
        4 => {
            background = Color(0.7, 0.8, 1.0);
            lookfrom = Point3(13.0, 2.0, 3.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 20.0;
            earth()
        },
        5 => {
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(26.0, 3.0, 6.0);
            lookat = Point3(0.0, 2.0, 0.0);
            vfov = 20.0;
            simple_light()
        },
        6 => {
            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 200;
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(278.0, 278.0, -800.0);
            lookat = Point3(278.0, 278.0, 0.0);
            vfov = 40.0;
            cornell_box()
        },
        _ => {
            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 200;
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(278.0, 278.0, -800.0);
            lookat = Point3(278.0, 278.0, 0.0);
            vfov = 40.0;
            cornell_smoke()
        }
    }, 0.0, 1.0);

    let image_height : i32 = ((image_width as f64) / aspect_ratio) as i32;
    let vup = Vec3(0., 1., 0.);
    let dist_to_focus = 10.0;
    let cam =
        Camera::new(&lookfrom, &lookat, &vup, vfov,
                    aspect_ratio, aperture, dist_to_focus, 0.0, 1.0);

    // Render

    let mut stdout = BufWriter::new(std::io::stdout().lock());
    let mut stderr = BufWriter::new(std::io::stderr().lock());

    writeln!(stdout, "P3");
    writeln!(stdout, "{} {}", image_width, image_height);
    writeln!(stdout, "255");

    for j in (0..image_height).rev() {
        write!(stderr, "\rScanlines remaining: {} ", j);
        stderr.flush();
        for i in 0..image_width {
            let mut pixel_color = Color(0., 0., 0.);
            for s in 0..samples_per_pixel {
                let u : f64 =
                    (f64::from(i) + random::double()) / f64::from(image_width - 1);
                let v : f64 =
                    (f64::from(j) + random::double()) / f64::from(image_height - 1);

                let r = cam.get_ray(u, v);
                pixel_color += ray_color(&r, &background, &world, MAX_DEPTH);
            }
            write_color(&mut stdout, &pixel_color, samples_per_pixel);
        }
    }
    write!(stderr, "\nDone\n");
}
