#![allow(unused)]

use std::io;
use hawxide::*;

use std::io::{Write, BufWriter};
use std::sync::Arc;
use std::sync::Mutex;
use rayon::prelude::*;

fn ray_color<H: Hittable>(r : &Ray,
                          background: &Color,
                          world: &H,
                          lights: Arc<dyn Hittable + Sync + Send>,
                          depth: i32) -> Color {
    if depth <= 0 {
        return Color(0., 0., 0.);
    }
    if let Some(hr) = world.hit(r, 0.001, INFINITY) {
        let emitted = hr.mat.emitted(r, &hr, hr.u, hr.v, &hr.p);
        if let Some(sr) =  hr.mat.scatter(r, &hr) {
            if let Some(spec_r) = sr.specular_ray {
                return sr.attenuation
                    * ray_color(&spec_r, background, world, lights.clone(), depth - 1);
            }
            let light_pdf = Arc::new(HittablePDF::new(lights.clone(), &hr.p));
            let mix_pdf = MixturePDF::new(light_pdf.clone(), sr.pdf.clone());
            let scattered = Ray::new(&hr.p, &mix_pdf.generate(), r.time);
            let pdf_val = mix_pdf.value(&scattered.dir);

            assert!(pdf_val > 0.0, "PDF val {:12} < 0", pdf_val);

            emitted +
                sr.attenuation *
                  hr.mat.scattering_pdf(r, &hr, &scattered) *
                  ray_color(&scattered, background, world, lights.clone(), depth-1) /
                pdf_val
        } else {
            emitted
        }
    } else {
        *background
    }
}

fn random_scene() -> HittableList {
    let mut world = HittableList::new();

    let checker  = Arc::new(
        CheckerTexture::new(&Color(0.2, 0.3, 0.1), &Color(0.9, 0.9, 0.9)));

    let ground_material =
        Arc::new(Lambertian{albedo: checker.clone()});
    world.add(Arc::new(Sphere::new(
        &Point3(0., -1000., 0.),
        1000.,
        ground_material.clone()
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
                        Arc::new(MovingSphere::new(
                            &center,
                            &(center + Vec3(0.0, random::double_range(0.0, 0.5), 0.0)),
                            0.0, 1.0, // times
                            0.2,      // radius
                            Arc::new(
                                Lambertian::new(&(Color::random() * Color::random())),
                            ),
                        ))
                    },
                    j if j < 0.95 => {
                        Arc::new(Sphere::new(
                            &center, 0.2,
                            Arc::new(Metal::new(
                                &Color::random_range(0.5, 1.),
                                random::double_range(0., 0.5),
                            ))
                        ))
                    },
                    _ => {
                        Arc::new(Sphere::new(
                            &center, 0.2,
                            Arc::new(Dielectric{
                                ir: 1.5,
                            })
                        ))
                    }
                }
                );
            }
        }
    }

    world.add(Arc::new(Sphere::new(
        &Point3(0., 1., 0.), 1.0,
        Arc::new(Dielectric{
            ir: 1.5,
        })
    )));

    world.add(Arc::new(Sphere::new(
        &Point3(-4., 1., 0.), 1.0,
        Arc::new(
            Lambertian::new(&Color(0.4, 0.2, 0.1))
        )
    )));

    world.add(Arc::new(Sphere::new(
        &Point3(4., 1., 0.), 1.0,
        Arc::new(Metal::new(&Color(0.7, 0.6, 0.5), 0.0))
    )));

    world
}

fn fancy_random_scene() -> HittableList {
    let mut world = HittableList::new();

    let checker  = Arc::new(
        CheckerTexture::new(&Color(0.2, 0.3, 0.1), &Color(0.9, 0.9, 0.9)));

    let ground_material =
        Arc::new(Lambertian{albedo: checker.clone()});
    world.add(Arc::new(Sphere::new(
        &Point3(0., -1000., 0.),
        1000.,
        ground_material.clone()
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
                    i if i < 0.75 => {
                        let rand_off = random::double_range(-0.05, 0.05);
                        Arc::new(Sphere::new(
                            &(center + Vec3(0.0, rand_off, 0.0)),
                            0.2 + rand_off,      // radius
                            Arc::new(
                                Lambertian::new(&(Color::random() * Color::random())),
                            ),
                        ))
                    },
                    j if j < 0.95 => {
                        Arc::new(Sphere::new(
                            &center, 0.2,
                            Arc::new(Metal::new(
                                &Color::random_range(0.5, 1.),
                                random::double_range(0., 0.5),
                            ))
                        ))
                    },
                    _ => {
                        Arc::new(Sphere::new(
                            &center, 0.2,
                            Arc::new(Dielectric{
                                ir: 1.5,
                            })
                        ))
                    }
                }
                );
            }
        }
    }

    world.add(Arc::new(Sphere::new(
        &Point3(0., 1., 0.), 1.0,
        Arc::new(Dielectric{
            ir: 1.5,
        })
    )));

    world.add(Arc::new(Sphere::new(
        &Point3(-4., 1., 0.), 1.0,
        Arc::new(
            Lambertian::new(&Color(0.4, 0.2, 0.1))
        )
    )));

    world.add(Arc::new(Sphere::new(
        &Point3(4., 1., 0.), 1.0,
        Arc::new(Metal::new(&Color(0.7, 0.6, 0.5), 0.0))
    )));

    world.add(Arc::new(Sphere::new(
        &Point3(3.0, 2.0, 2.0), 0.6,
        Arc::new(
            DiffuseLight::new(&Color(30.0, 30.0, 30.0))
        )
    )));

    world
}

fn two_spheres() -> HittableList {
    let checker = Arc::new(CheckerTexture::new(
        &Color(0.2, 0.3, 0.1),
        &Color(0.9, 0.9, 0.9),
    ));

    HittableList {
        objects: vec![
            Arc::new(Sphere::new(
                &Point3(0.0, -10.0, 0.0),
                10.0,
                Arc::new(Lambertian {
                    albedo: checker.clone()
                }),
            )),
            Arc::new(Sphere::new(
                &Point3(0.0, 10.0, 0.0),
                10.0,
                Arc::new(Lambertian {
                    albedo: checker.clone()
                }),
            )),
        ],
    }
}

fn two_perlin_spheres() -> HittableList {
    let pertext = Arc::new(MarbleTexture::new(4.));

    HittableList {
        objects: vec![
            Arc::new(Sphere::new(
                &Point3(0.0, -1000.0, 0.0),
                1000.0,
                Arc::new(Lambertian {
                    albedo: pertext.clone(),
                }),
            )),
            Arc::new(Sphere::new(
                &Point3(0.0, 2.0, 0.0),
                2.0,
                Arc::new(Lambertian {
                    albedo: pertext.clone(),
                }),
            )),
        ]
    }
}

fn subsurface_perlin_spheres() -> HittableList {
    let pertext = Arc::new(MarbleTexture::new(4.));
    let turq_light = Arc::new(DiffuseLight::new(&Color(0.0, 12., 10.)));
    let red_light = Arc::new(DiffuseLight::new(&Color(12.0, 0.0, 5.0)));

    let mut objects = HittableList::new();

    objects.add(Arc::new(Sphere::new(
        &Point3(0.0, -1000.0, 0.0),
        999.5,
        Arc::new(Lambertian {
            albedo: pertext.clone(),
        }),
    )));

    objects.add(Arc::new(Sphere::new(
        &Point3(0.0, -1000.0, 0.0),
        1000.0,
        Arc::new(Dielectric {ir: 1.5}),
    )));

    objects.add(Arc::new(Sphere::new(
        &Point3(0.0, 2.0, 0.0),
        2.0,
        Arc::new(Dielectric { ir: 1.5}),
    )));
    objects.add(Arc::new(Sphere::new(
        &Point3(0.0, 2.0, 0.0),
        1.5,
        Arc::new(Lambertian {
            albedo: pertext.clone(),
        }),
    )));

    objects.add(Arc::new(Sphere::new(
        &Point3(6.0, 4.0, -4.0),
        2.0,
        turq_light.clone(),
    )));

    objects.add(Arc::new(Sphere::new(
        &Point3(-3.0, 3.0, 4.0),
        1.0,
        red_light.clone(),
    )));

    objects
}

fn earth() -> HittableList {

    let earth_texture = Arc::new(ImageTexture::new("earthmap.jpg"));

    HittableList {
        objects: vec![
            Arc::new(Sphere::new(
                &Point3(0.0, 0.0, 0.0),
                2.0,
                Arc::new(Lambertian {
                    albedo: earth_texture.clone(),
                })
            )),
        ],
    }
}

fn simple_light() -> HittableList {
    let pertext = Arc::new(MarbleTexture::new(4.));
    // let difflight = Arc::new(DiffuseLight::new(&Color(4., 4., 4.)));
    let difflight = Arc::new(DiffuseLight::new(&Color(7., 7., 7.)));

    HittableList {
        objects: vec![
            Arc::new(Sphere::new(
                &Point3(0.0, -1000.0, 0.0),
                1000.0,
                Arc::new(Lambertian {
                    albedo: pertext.clone(),
                }),
            )),
            Arc::new(Sphere::new(
                &Point3(0.0, 2.0, 0.0),
                2.0,
                Arc::new(Lambertian {
                    albedo: pertext.clone(),
                }),
            )),
            Arc::new(AARect::xy_rect(
                3.0, 5.0, 1.0, 3.0, -2.0,
                difflight.clone(),
            )),
        ]
    }
}

fn wacky_cornell_box() -> HittableList {
    let red = Arc::new(Lambertian::new(&Color(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(&Color(0.12, 0.45, 0.15)));
    // let light = Arc::new(DiffuseLight::new(&Color(15.0, 15.0, 15.0)));
    let light = Arc::new(DiffuseLight::new(&Color(5.0, 5.0, 5.0)));
    let mirror_back = Arc::new(Metal::new(&Color(0.73, 0.73, 0.73), 0.0));
    let mirror_front = Arc::new(Dielectric{ir: 1.5});
    let earth = Arc::new(
        Lambertian{albedo: Arc::new(ImageTexture::new("earthmap.jpg"))}
    );
    let wood = Arc::new(
        Lambertian{
            albedo: Arc::new(
                WoodTexture::new(Vec3(4.0, 0.1, 1.0), Color(0.7, 0.3, 0.1))
            )
        });

    let voronoi = Arc::new(
        Lambertian{
            albedo: Arc::new(VoronoiTexture::new(&Color(1.0, 1.0, 1.0), 200))
        });
    let fun_noise = Arc::new(
        Lambertian{
            albedo: Arc::new(NoiseTexture::from_texture(voronoi.albedo.clone()))
        });
    // let mirror = Arc::new(Metal::new(&Color(1.0, 1.0, 1.0), 0.0));

    let mut box1 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 330.0, 165.0),
        white.clone(),
    ));
    box1 = Arc::new(Rotate::rotate_y(box1, 15.0));
    box1 = Arc::new(Translate::new(box1, &Vec3(265.0, 0.0, 295.0)));

    let mut box2 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 165.0, 165.0),
        fun_noise.clone(),
    ));
    box2 = Arc::new(Rotate::rotate_y(box2, -18.0));
    box2 = Arc::new(Translate::new(box2, &Vec3(130.0, 0.0, 65.0)));

    let mut mirror: Arc<dyn Hittable + Sync + Send> = Arc::new(HittableList{
        objects: vec![
            Arc::new(AARect::xy_rect(
                113.0, 443.0, 127.0, 432.0, 554.0, mirror_back.clone()
            )),
            // Arc::new(AARect::xy_rect(
            //     113.0, 443.0, 127.0, 432.0, 553.99, mirror_front.clone()
            // )),
        ]
    });

    mirror = Arc::new(Translate::new(mirror, &Vec3(-100.0, 0.0, 0.0)));

    HittableList {
        objects: vec![
            Arc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
            )),
            Arc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
            )),
            mirror,
            // Arc::new(AARect::xz_rect(
            //     213.0, 343.0, 227.0, 332.0, 554.0, light.clone()
            // )),
            Arc::new(AARect::xz_rect(
                013.0, 543.0, 027.0, 532.0, 554.0, light.clone()
            )),
            Arc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, wood.clone()
            )),
            Arc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
            )),
            Arc::new(AARect::xy_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
            )),
            box1,
            box2,
        ]
    }
}

fn cornell_box() -> HittableList {
    let red = Arc::new(Lambertian::new(&Color(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(&Color(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(&Color(15.0, 15.0, 15.0)));
    let aluminum = Arc::new(Metal::new(&Color(0.8, 0.85, 0.88), 0.0));

    let mut box1 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 330.0, 165.0),
        // aluminum.clone(),
        white.clone(),
    ));
    box1 = Arc::new(Rotate::rotate_y(box1, 15.0));
    box1 = Arc::new(Translate::new(box1, &Vec3(265.0, 0.0, 295.0)));

    let mut box2 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 165.0, 165.0),
        white.clone(),
    ));
    box2 = Arc::new(Rotate::rotate_y(box2, -18.0));
    box2 = Arc::new(Translate::new(box2, &Vec3(130.0, 0.0, 65.0)));

    let sphere = Arc::new(Sphere::new(
        &Point3(190.0, 90.0, 190.0), 90.0,
        Arc::new(Dielectric{ir: 1.5}),
    ));

    HittableList {
        objects: vec![
            Arc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
            )),
            Arc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
            )),
            Arc::new(FlipFace::new(Arc::new(AARect::xz_rect(
                213.0, 343.0, 227.0, 332.0, 554.0, light.clone()
            )))),
            Arc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, white.clone()
            )),
            Arc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
            )),
            Arc::new(AARect::xy_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
            )),
            box1,
            sphere,
            // box2,
        ]
    }
}

fn cornell_smoke() -> HittableList {
    let red = Arc::new(Lambertian::new(&Color(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(&Color(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(&Color(7.0, 7.0, 7.0)));

    let mut box1 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 330.0, 165.0),
        white.clone(),
    ));
    box1 = Arc::new(Rotate::rotate_y(box1, 15.0));
    box1 = Arc::new(Translate::new(box1, &Vec3(265.0, 0.0, 295.0)));

    let mut box2 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
        &Point3(0.0, 0.0, 0.0),
        &Point3(165.0, 165.0, 165.0),
        white.clone(),
    ));
    box2 = Arc::new(Rotate::rotate_y(box2, -18.0));
    box2 = Arc::new(Translate::new(box2, &Vec3(130.0, 0.0, 65.0)));

    HittableList {
        objects: vec![
            Arc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
            )),
            Arc::new(AARect::yz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
            )),
            Arc::new(FlipFace::new(Arc::new(AARect::xz_rect(
                113.0, 443.0, 127.0, 432.0, 554.0, light.clone()
            )))),
            // Arc::new(AARect::xz_rect(
            //     213.0, 343.0, 227.0, 332.0, 554.0, light.clone()
            // )),
            Arc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 0.0, white.clone()
            )),
            Arc::new(AARect::xz_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
            )),
            Arc::new(AARect::xy_rect(
                0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
            )),
            Arc::new(ConstantMedium::new(box1, 0.01, &Color::new())),
            Arc::new(ConstantMedium::new(box2, 0.01, &Color(1.0, 1.0, 1.0))),
        ]
    }
}

fn solids() -> HittableList {

    let wood = Arc::new(WoodTexture::new(Vec3(4.0, 0.1, 1.0), Color(0.7, 0.3, 0.1)));
    let light = Arc::new(DiffuseLight::new(&Color(7.0, 7.0, 7.0)));
    let sunlight = Arc::new(DiffuseLight::new(&Color(20.0, 15.5, 11.0)));
    let white = Arc::new(Metal::new(&Color(0.73, 0.73, 0.73), 0.95));

    let mut objects = HittableList{
        objects: vec![
            Arc::new(AARect::yz_rect(
                -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
            )),
            Arc::new(AARect::yz_rect(
                -10.0, 10.0, -10.0, 10.0, 10.0, white.clone(),
            )),
            Arc::new(AARect::xz_rect(
                -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
            )),
            Arc::new(AARect::xz_rect(
                -10.0, 10.0, -10.0, 10.0, 10.0, white.clone(),
            )),
            Arc::new(AARect::xy_rect(
                -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
            )),
        ]
    };

    let mut panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
        AARect::xz_rect(-3.0, 3.0, -2.0, 2.0, -3.0, light.clone())
    );
    let mut right_panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
        Rotate::rotate_z(panel.clone(), 45.0)
    );
    right_panel = Arc::new(Translate::new(right_panel.clone(), &Point3(2.0, 0.0, 0.0)));
    objects.add(right_panel);

    let mut left_panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
        Rotate::rotate_z(panel.clone(), -45.0)
    );
    left_panel = Arc::new(Translate::new(left_panel.clone(), &Point3(-2.0, 0.0, 0.0)));
    objects.add(left_panel);

    let mut front_panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
        Rotate::rotate_x(panel.clone(), -45.0)
    );
    front_panel = Arc::new(Translate::new(front_panel.clone(), &Point3(1.0, 1.0, 4.0)));
    objects.add(front_panel);

    objects.add(Arc::new(Sphere::new(
        &Point3(0.0, 3.5, 0.0), 1.0,
        sunlight.clone()
    )));

    let sphere: Arc<dyn Hittable + Sync + Send> = Arc::new(Sphere::new(
        &Point3(0.0, 0.0, 0.0), 2.0,
        Arc::new(Lambertian{albedo: wood.clone()}),
    ));

    objects.add(sphere);

    let boundary = Arc::new(Sphere::new(
        &Point3(3.0, 3.0, 0.0), 1.0,
        Arc::new(Dielectric {ir: 1.5})
    ));
    objects.add(boundary.clone());
    objects.add(Arc::new(ConstantMedium::new(
        boundary.clone(), 1.0, &Color(0.2, 0.4, 0.9),
    )));

    // let mut block: Arc<dyn Hittable + Sync + Send>= Arc::new(Boxx::new(
    //     &Point3(-2.0, -2.0, -2.0), &Point3(2.0, 2.0, 2.0),
    //     Arc::new(Lambertian{albedo: wood.clone()}),
    // ));

    // block = Arc::new(Rotate::rotate_y(block, 18.0));
    // block = Arc::new(Rotate::rotate_z(block, 10.0));



    objects
}

fn noise_experiments() -> HittableList {

    let wood = Arc::new(Lambertian{
        albedo: Arc::new(
            WoodTexture::new(Vec3(4.0, 0.1, 1.0), Color(0.7, 0.3, 0.1))
        )
    });
    let marble = Arc::new(Lambertian{
        albedo: Arc::new(
            MarbleTexture::new(4.)
        )
    });
    let voronoi = Arc::new(VoronoiTexture::new(&Color(1.0, 1.0, 1.0), 200));
    let fun_noise = Arc::new(
        NoiseTexture::from_texture(voronoi.clone())
    );
    let noise = Arc::new(Lambertian{albedo: fun_noise.clone()});
    let light = Arc::new(DiffuseLight::new(&Color(10.0, 10.0, 10.0)));
    let sunlight = Arc::new(DiffuseLight::new(&Color(20.0, 15.5, 11.0)));
    let white = Arc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(&Color(0.12, 0.45, 0.15)));

    let fun = Arc::new(Lambertian{
        albedo: voronoi.clone(),
    });

    let mut objects = HittableList{
        objects: vec![
            Arc::new(AARect::yz_rect(
                -10.0, 10.0, -10.0, 10.0, -10.0, green.clone(),
            )),
            Arc::new(AARect::yz_rect(
                -10.0, 10.0, -10.0, 10.0, 10.0, green.clone(),
            )),
            Arc::new(AARect::xz_rect(
                -10.0, 10.0, -10.0, 10.0, -10.0, fun.clone(),
            )),
            Arc::new(AARect::xz_rect(
                -10.0, 10.0, -10.0, 10.0, 10.0, white.clone(),
            )),
            Arc::new(AARect::xy_rect(
                -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
            )),
        ]
    };

    let mut panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
        AARect::xz_rect(-5.0, 5.0, -5.0, 5.0, 9.99, light.clone())
    );
    objects.add(panel);

    // panel = Arc::new(
    //     AARect::xz_rect(-5.0, 5.0, -5.0, 5.0, -9.99, light.clone())
    // );
    panel = Arc::new(
        AARect::yz_rect(-2.0, 2.0, -2.0, 2.0, 2.1, light.clone())
    );
    panel = Arc::new(Rotate::rotate_y(panel, -45.0));
    panel = Arc::new(Translate::new(panel, &Point3(2.0, 0.0, 2.0)));
    objects.add(panel);

    let mut block: Arc<dyn Hittable + Sync + Send> = Arc::new(
        Boxx::new(
            &Point3(-2.0, -2.0, -2.0),
            &Point3(2.0, 2.0, 2.0),
            marble.clone(),
        )
    );

    // block = Arc::new(Rotate::rotate_z(block, -45.0));
    // block = Arc::new(Rotate::rotate_x(block, -90.0));
    block = Arc::new(Rotate::rotate_y(block, 45.0));

    // block = Arc::new(Translate::new(block, &Point3(4.0, -2.0, -8.0)));
    // block = Arc::new(Translate::new(block, &Point3(-4.0, 0.0, 0.0)));

    objects.add(block);

    objects
}
fn final_scene() -> HittableList {
    let mut boxes1 = HittableList::new();
    let ground = Arc::new(Lambertian::new(&Color(0.48, 0.83, 0.53)));

    const BOXES_PER_SIDE : i32 = 20;

    for i in 0..BOXES_PER_SIDE {
        for j in 0..BOXES_PER_SIDE {
            let w = 100.0;
            let p0 = Point3(
                -1000.0 + f64::from(i) * w,
                0.0,
                -1000.0 + f64::from(j) * w,
            );
            let p1 = Point3(
                p0.x() + w,
                random::double_range(1.0, 101.0),
                p0.z() + w,
            );

            boxes1.add(
                Arc::new(Boxx::new(&p0, &p1, ground.clone())),
            );
        }
    };

    let mut objects = HittableList::new();

    objects.add(Arc::new(BVHNode::new(&boxes1, 0.0, 1.0)));

    let light = Arc::new(DiffuseLight::new(&Color(7.0, 7.0, 7.0)));
    objects.add(Arc::new(
        AARect::xz_rect(
            123.0, 423.0, 147.0, 412.0, 554.0, light.clone()
        )
    ));

    let center1 = Point3(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3(30.0, 0.0, 0.0);
    let moving_sphere_mat = Arc::new(Lambertian::new(&Color(0.7, 0.3, 0.1)));
    objects.add(Arc::new(MovingSphere::new(
        &center1, &center2, 0.0, 1.0, 50.0, moving_sphere_mat.clone()
    )));


    objects.add(Arc::new(Sphere::new(
        &Point3(260.0, 150.0, 45.0), 50.0,
        Arc::new(Dielectric {ir: 1.5}),
    )));
    objects.add(Arc::new(Sphere::new(
        &Point3(0.0, 150.0, 145.0), 50.0,
        Arc::new(Metal::new(&Color(0.8, 0.8, 0.9), 1.0))
    )));

    let boundary = Arc::new(Sphere::new(
        &Point3(360.0, 150.0, 145.0), 70.0,
        Arc::new(Dielectric {ir: 1.5}),
    ));
    objects.add(boundary.clone());
    objects.add(Arc::new(ConstantMedium::new(
        boundary.clone(), 0.2, &Color(0.2, 0.4, 0.9),
    )));
    let boundary = Arc::new(Sphere::new(
        &Point3::new(), 5000.0,
        Arc::new(Dielectric {ir: 1.5}),
    ));
    objects.add(Arc::new(ConstantMedium::new(
        boundary.clone(), 0.0001, &Color(1.0, 1.0, 1.0),
    )));

    let emat = Arc::new(Lambertian {
        albedo: Arc::new(ImageTexture::new("earthmap.jpg"))}
    );
    objects.add(Arc::new(Sphere::new(
        &Point3(400.0, 200.0, 400.0), 100.0, emat.clone(),
    )));

    let pertext = Arc::new(MarbleTexture::new(0.1));
    objects.add(Arc::new(Sphere::new(
        &Point3(220.0, 280.0, 300.0), 80.0,
        Arc::new(Lambertian { albedo: pertext.clone() }),
    )));

    let mut boxes2 = HittableList::new();
    let white = Arc::new(Lambertian::new(&Color(0.73, 0.73, 0.73)));
    let ns = 1000;
    for j in 0..ns {
        boxes2.add(Arc::new(Sphere::new(
            &Point3::random_range(0.0, 165.0), 10.0, white.clone(),
        )));
    }

    objects.add(Arc::new(Translate::new(
        Arc::new(Rotate::rotate_y(
            Arc::new(BVHNode::new(&boxes2, 0.0, 1.0)), 15.0
        )),
        &Vec3(-100.0, 270.0, 395.0),
        ))
    );

    objects
}

fn estimate_pi() {
    let mut inside_circle = 0u32;
    let mut inside_circle_stratified = 0u32;
    let mut runs = 0;
    let sqrt_n = 10000;
    for i in 0..sqrt_n {
        for j in 0..sqrt_n {
            runs += 1;
            let mut x = random::double_range(-1.0, 1.0);
            let mut y = random::double_range(-1.0, 1.0);
            if x * x + y * y < 1.0 {
                inside_circle += 1
            }
            x = 2.0 * ((f64::from(i) + random::double()) / f64::from(sqrt_n)) - 1.0;
            y = 2.0 * ((f64::from(j) + random::double()) / f64::from(sqrt_n)) - 1.0;
            if x * x + y * y < 1.0 {
                inside_circle_stratified += 1;
            }
        }
    }
    let n = f64::from(sqrt_n * sqrt_n);
    eprintln!("Regular Estimate of pi = : {:12}",
              4.0 * f64::from(inside_circle) / n);
    eprintln!("Stratified Estimate of pi = : {:12}",
              4.0 * f64::from(inside_circle_stratified) / n);
}

fn pdf(_x: &Vec3) -> f64 {
    return  1.0 / (4. * PI);
}

fn estimate_integral() {
    let n = 1000000;
    let mut sum = 0.0;
    for i in 0..n {
        let v = random::cosine_direction();
        sum += v.z() * v.z() * v.z() / (v.z()/PI);
    }
    eprintln!("PI / 2      = {:12}", PI / 2.0);
    eprintln!("Estimate    = {:12}", sum / f64::from(n));
}

fn main() {

    // estimate_integral();
    // return;
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

    let scene_select: usize = 7;

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
            samples_per_pixel = 400;
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
        7 => {
            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 100;
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(278.0, 278.0, -800.0);
            lookat = Point3(278.0, 278.0, 0.0);
            vfov = 40.0;
            cornell_smoke()
        },
        8 => {
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(13.0, 5.0, 3.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 40.0;
            samples_per_pixel = 400;
            subsurface_perlin_spheres()
        },
        9 => {
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(-4.0, 4.0, 15.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 40.0;
            samples_per_pixel = 500;
            solids()
        },
        10 => {
            background = Color(0.0, 0.0, 0.0);
            // background = Color(1.0, 1.0, 1.0);
            lookfrom = Point3(0.0, 0.0, 18.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 60.0;
            samples_per_pixel = 200;
            noise_experiments()
        },
        11 => {
            aspect_ratio = 1.0;
            // image_width = 600;
            image_width = 300;
            samples_per_pixel = 1000;
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(278.0, 278.0, -800.0);
            lookat = Point3(278.0, 278.0, 0.0);
            vfov = 40.0;
            wacky_cornell_box()
        },
        12 => {
            // background = Color(0.7, 0.8, 1.0);
            background = Color(0.0, 0.0, 0.0);
            lookfrom = Point3(13.0, 2.0, 3.0);
            lookat = Point3(0.0, 0.0, 0.0);
            vfov = 20.0;
            aperture = 0.05;
            samples_per_pixel = 4000;
            fancy_random_scene()
        },
        _ => {
            aspect_ratio = 1.0;
            image_width = 800;
            samples_per_pixel = 4000;
            background = Color::new();
            lookfrom = Point3(478.0, 278.0, -600.0);
            lookat = Point3(278.0, 278.0, 0.0);
            vfov = 40.0;
            final_scene()
        }
    }, 0.0, 1.0);

    let panel = Arc::new(AARect::xz_rect(
        213.0, 343.0, 227.0, 332.0, 554.0,
        Arc::new(DiffuseLight::new(&Color(15.0, 15.0, 15.0)))
    ));

    let panel = Arc::new(AARect::xz_rect(
        113.0, 443.0, 127.0, 432.0, 554.0,
        Arc::new(DiffuseLight::new(&Color(7.0, 7.0, 7.0)))
    ));

    let sphere = Arc::new(Sphere::new(
        &Point3(190.0, 90.0, 190.0), 90.0,
        Arc::new(Dielectric{ir: 1.5}),
    ));

    let lights = Arc::new(HittableList {
        objects: vec![
            panel, sphere,
        ]
    });

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
        let colors : Vec<Color> = (0..image_width).into_par_iter().map(|i| {
            let mut pixel_color = Color(0., 0., 0.);
            for s in (0..samples_per_pixel) {
                let u : f64 =
                    (f64::from(i) + random::double()) / f64::from(image_width - 1);
                let v : f64 =
                    (f64::from(j) + random::double()) / f64::from(image_height - 1);

                let r = cam.get_ray(u, v);
                pixel_color +=
                    ray_color(&r, &background, &world, lights.clone(), MAX_DEPTH)
            };
            pixel_color
        }).collect();
        for pc in &colors {
            write_color(&mut stdout, &pc, samples_per_pixel);
        }
    }
    write!(stderr, "\nDone\n");
}
