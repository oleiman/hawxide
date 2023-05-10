use crate::vec3::{Point3,Color};
use crate::hit::Hittable;

use std::sync::Arc;

pub struct Scene {
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub background: Color,
    pub vfov: f64,
    pub world: Arc<dyn Hittable + Sync + Send>,
    pub lights: Arc<dyn Hittable + Sync + Send>,
}

pub mod defs {
    use crate::scene::Scene;
    use crate::vec3::{Point3,Color,Vec3};
    use crate::texture::*;
    use crate::material::*;
    use crate::sphere::*;
    use crate::hit::*;
    use crate::aarect::*;
    use crate::hittable_list::*;
    use crate::bvh::*;
    use crate::boxx::*;
    use crate::moving_sphere::*;
    use crate::constant_medium::*;
    use crate::util::random;
    use std::sync::Arc;

    const RED: Color = Color(0.65, 0.05, 0.05);
    const WHITE: Color  = Color(0.73, 0.73, 0.73);
    const GREEN: Color  = Color(0.12, 0.45, 0.15);

    pub fn random_scene() -> Scene {
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;
        let background = Color(0.7, 0.8, 1.0);
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

                if (center - Point3(4., 0.2, 0.)).len() > 0.9 {
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

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList::new()),
        }
    }

    pub fn two_spheres() -> Scene {
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let background = Color(0.7, 0.8, 1.0);
        let vfov = 20.0;

        let checker = Arc::new(CheckerTexture::new(
            &Color(0.2, 0.3, 0.1),
            &Color(0.9, 0.9, 0.9),
        ));

        let world = HittableList {
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
        };
        Scene {
            lookfrom, lookat, background, vfov, 
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList::new()),
        }
    }

    pub fn empty_cornell_box() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(278.0, 278.0, -800.0);
        let lookat = Point3(278.0, 278.0, 0.0);
        let vfov = 40.0;

        let red = Arc::new(Lambertian::new(&RED));
        let white = Arc::new(Lambertian::new(&WHITE));
        let green = Arc::new(Lambertian::new(&GREEN));
        let light = Arc::new(DiffuseLight::new(&Color(15.0, 15.0, 15.0)));

        let light_panel = Arc::new(AARect::xz_rect(
            213.0, 343.0, 227.0, 332.0, 554.0, light.clone()
        ));

        let world = Arc::new(HittableList {
            objects: vec![
                Arc::new(AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
                )),
                Arc::new(AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
                )),
                Arc::new(AARect::xz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, white.clone()
                )),
                Arc::new(AARect::xz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
                )),
                Arc::new(AARect::xy_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
                )),
                Arc::new(FlipFace::new(light_panel.clone())),
            ]
        });

        Scene {
            lookfrom, lookat, background, vfov, world,
            lights: Arc::new(HittableList {
                objects: vec![
                    light_panel.clone(),
                ],
            }),
        }
    }

    #[allow(unused)]
    pub fn cornell_box() -> Scene {

        let aluminum = Arc::new(Metal::new(&Color(0.8, 0.85, 0.88), 0.0));
        let white = Arc::new(Lambertian::new(&WHITE));

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

        let cbox = empty_cornell_box();

        // TODO(oren): do as_any
        // cbox.world.as_any().downcast_ref::<HittableList>.add(box1);
        // cbox.world.add(box2);

        let world = HittableList {
            objects: vec![
                box1, box2,
                cbox.world,
            ],
        };

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: cbox.lights,
        }
    }

    pub fn cornell_sphere() -> Scene {
        let cbox = empty_cornell_box();

        let white = Arc::new(Lambertian::new(&WHITE));

        let mut box1 : Arc<dyn Hittable + Sync + Send> = Arc::new(Boxx::new(
            &Point3(0.0, 0.0, 0.0),
            &Point3(165.0, 330.0, 165.0),
            white.clone(),
        ));
        box1 = Arc::new(Rotate::rotate_y(box1, 15.0));
        box1 = Arc::new(Translate::new(box1, &Vec3(265.0, 0.0, 295.0)));

        let sphere = Arc::new(Sphere::new(
            &Point3(190.0, 90.0, 190.0), 90.0,
            Arc::new(Dielectric{ir: 1.5}),
        ));

        let world = HittableList {
            objects: vec![
                box1, sphere.clone(),
                cbox.world,
            ],
        };

        let lights = Arc::new(HittableList {
            objects: vec![
                cbox.lights.clone(), sphere.clone()
            ]
        });

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: lights,
        }
    }

    pub fn two_perlin_spheres() -> Scene {
        let background = Color(0.7, 0.8, 1.0);
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;

        let pertext = Arc::new(MarbleTexture::new(4.));

        let world = Arc::new(HittableList {
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
        });

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList::new()),
        }
    }

    pub fn earth() -> Scene {
        let background = Color(0.7, 0.8, 1.0);
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;

        let earth_texture = Arc::new(ImageTexture::new("earthmap.jpg"));

        let world = Arc::new(HittableList {
            objects: vec![
                Arc::new(Sphere::new(
                    &Point3(0.0, 0.0, 0.0),
                    2.0,
                    Arc::new(Lambertian {
                        albedo: earth_texture.clone(),
                    })
                )),
            ],
        });

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList::new()),
        }
    }

    pub fn simple_light() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(26.0, 3.0, 6.0);
        let lookat = Point3(0.0, 2.0, 0.0);
        let vfov = 20.0;

        let pertext = Arc::new(MarbleTexture::new(4.));
        // let difflight = Arc::new(DiffuseLight::new(&Color(4., 4., 4.)));
        let difflight = Arc::new(DiffuseLight::new(&Color(7., 7., 7.)));

        let light_panel = Arc::new(AARect::xy_rect(
            3.0, 5.0, 1.0, 3.0, -2.0,
            difflight.clone(),
        ));

        let world = Arc::new(HittableList {
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
                light_panel.clone(),
            ]
        });

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList {
                objects: vec![light_panel],
            }),
        }
    }

    pub fn cornell_smoke() -> Scene {
        let white = Arc::new(Lambertian::new(&WHITE));

        let cbox = empty_cornell_box();

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

        box1 = Arc::new(ConstantMedium::new(box1, 0.01, &Color::new()));
        box2 = Arc::new(ConstantMedium::new(box2, 0.01, &Color(1.0, 1.0, 1.0)));

        let world = HittableList {
            objects: vec![
                box1, box2,
                cbox.world,
            ],
        };

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: cbox.lights,
        }
    }

    pub fn fancy_random_scene() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;

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

                if (center - Point3(4., 0.2, 0.)).len() > 0.9 {
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

        let light_sphere = Arc::new(Sphere::new(
            &Point3(3.0, 2.0, 2.0), 0.6,
            Arc::new(
                DiffuseLight::new(&Color(30.0, 30.0, 30.0))
            )
        ));

        world.add(light_sphere.clone());

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList {
                objects: vec![light_sphere],
            })
        }
    }

    pub fn final_scene() -> Scene {
        let background = Color::new();
        let lookfrom = Point3(478.0, 278.0, -600.0);
        let lookat = Point3(278.0, 278.0, 0.0);
        let vfov = 40.0;

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
        let light_panel = Arc::new(
            AARect::xz_rect(
                123.0, 423.0, 147.0, 412.0, 554.0, light.clone()
            )
        );
        objects.add(Arc::new(FlipFace::new(light_panel.clone())));

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
        for _j in 0..ns {
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

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&objects, 0.0, 1.0)),
            lights: Arc::new(HittableList {
                objects: vec![light_panel]
            }),
        }
    }

    #[allow(unused)]
    pub fn wacky_cornell_box() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(278.0, 278.0, -800.0);
        let lookat = Point3(278.0, 278.0, 0.0);
        let vfov = 40.0;

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
        let light_panel = Arc::new(AARect::xz_rect(
            113.0, 443.0, 127.0, 432.0, 554.0, light.clone()
        ));
        // Arc::new(AARect::xz_rect(
        //     013.0, 543.0, 027.0, 532.0, 554.0, light.clone()
        // )),

        let world = HittableList {
            objects: vec![
                Arc::new(AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
                )),
                Arc::new(AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
                )),
                mirror,
                Arc::new(FlipFace::new(light_panel.clone())),
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
        };

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&world, 0.0, 1.0)),
            lights: Arc::new(HittableList {
                objects: vec![
                    light_panel
                ]
            }),
        }
    }

    pub fn subsurface_perlin_spheres() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(13.0, 5.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 40.0;

        let pertext = Arc::new(MarbleTexture::new(4.0));
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

        let turq_sphere = Arc::new(Sphere::new(
            &Point3(6.0, 4.0, -4.0),
            2.0,
            turq_light.clone(),
        ));

        let red_sphere = Arc::new(Sphere::new(
            &Point3(-3.0, 3.0, 4.0),
            1.0,
            red_light.clone(),
        ));

        objects.add(turq_sphere.clone());
        objects.add(red_sphere.clone());

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&objects, 0.0, 1.0)),
            lights: Arc::new(HittableList{
               objects: vec![
                   turq_sphere, red_sphere,
               ],
            }),

        }
    }

    pub fn solids() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(-4.0, 4.0, 15.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 40.0;

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

        let panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
            AARect::xz_rect(-3.0, 3.0, -2.0, 2.0, -3.0, light.clone())
        );
        let mut right_panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
            Rotate::rotate_z(panel.clone(), 45.0)
        );
        right_panel = Arc::new(Translate::new(right_panel.clone(), &Point3(2.0, 0.0, 0.0)));
        objects.add(right_panel.clone());

        // TODO(oren): this panel has the weird black section issue that we sometimes
        // see on rotated objects. Don't really know what the deal is there, but this
        // one is an excellent example. I think it has something to do with the sign
        // of the surface normal
        let mut left_panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
            Rotate::rotate_z(panel.clone(), 315.0)
        );
        left_panel = Arc::new(Translate::new(left_panel.clone(), &Point3(-2.0, 0.0, 0.0)));
        objects.add(left_panel.clone());

        let mut front_panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
            Rotate::rotate_x(panel.clone(), -45.0)
        );
        front_panel = Arc::new(Translate::new(front_panel.clone(), &Point3(1.0, 1.0, 4.0)));
        objects.add(front_panel.clone());

        let sun = Arc::new(Sphere::new(
            &Point3(0.0, 3.5, 0.0), 1.0,
            sunlight.clone()
        ));

        objects.add(sun.clone());

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

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&objects, 0.0, 1.0)),
            lights: Arc::new(HittableList {
               objects: vec![sun, right_panel, left_panel, front_panel],
            }),
        }
    }

    #[allow(unused)]
    pub fn noise_experiments() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(0.0, 0.0, 18.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 60.0;

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
        let aluminum = Arc::new(Metal::new(&Color(0.8, 0.85, 0.88), 0.0));
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

        let panel: Arc<dyn Hittable + Sync + Send> = Arc::new(
            AARect::xz_rect(-5.0, 5.0, -5.0, 5.0, 9.99, light.clone())
        );
        objects.add(Arc::new(FlipFace::new(panel.clone())));

        // panel = Arc::new(
        //     AARect::xz_rect(-5.0, 5.0, -5.0, 5.0, -9.99, light.clone())
        // );
        let mut panel2: Arc<dyn Hittable + Sync + Send> = Arc::new(
            AARect::yz_rect(-2.0, 2.0, -2.0, 2.0, 2.1, light.clone())
        );
        panel2 = Arc::new(Rotate::rotate_y(panel2, -45.0));
        panel2 = Arc::new(Translate::new(panel2, &Point3(2.0, 0.0, 2.0)));
        // objects.add(Arc::new(FlipFace::new(panel2.clone())));

        let mut block: Arc<dyn Hittable + Sync + Send> = Arc::new(
            Boxx::new(
                &Point3(-2.0, -2.0, -2.0),
                &Point3(2.0, 2.0, 2.0),
                aluminum.clone(),
                // marble.clone(),
            )
        );

        // block = Arc::new(Rotate::rotate_z(block, -45.0));
        // block = Arc::new(Rotate::rotate_x(block, -90.0));
        block = Arc::new(Rotate::rotate_y(block, 45.0));

        // block = Arc::new(Translate::new(block, &Point3(4.0, -2.0, -8.0)));
        // block = Arc::new(Translate::new(block, &Point3(-4.0, 0.0, 0.0)));

        objects.add(block);

        Scene {
            lookfrom, lookat, background, vfov,
            world: Arc::new(BVHNode::new(&objects, 0.0, 1.0)),
            lights: Arc::new(HittableList {
                objects: vec![
                    panel, //panel2,
                ]
            })
        }
    }
}
