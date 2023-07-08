use crate::vec3::{Point3,Color};
use crate::hit::{Hittable, FlipFace};
use crate::hittable_list::HittableList;
use crate::texture;
use crate::texture::Texture;
use crate::material;
use crate::material::Material;
use crate::sphere::Sphere;
use crate::aarect::AARect;

use std::sync::Arc;
use std::fs::File;
use std::io::{Read};
use std::error::Error;
use std::collections::hash_map::HashMap;
use serde_json::{Value, from_value};

pub struct Scene {
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub background: Color,
    pub vfov: f64,
    pub world: Arc<dyn Hittable + Sync + Send>,
    pub lights: Arc<dyn Hittable + Sync + Send>,
}

impl Scene {
    pub fn from_reader<R: Read>(reader: R) -> Result<Scene, Box<dyn Error>> {
        let s: Value = serde_json::from_reader(reader)?;

        let lookfrom: Point3 = from_value(s["lookfrom"].clone())?;
        let lookat: Point3 = from_value(s["lookat"].clone())?;
        let background: Color = from_value(s["background"].clone()).unwrap_or_else(
            |_e| Color(0.0, 0.0, 0.0)
        );
        let vfov: f64 = from_value(s["vfov"].clone()).unwrap_or_else(
            |_e| 20.0
        );

        let textures: HashMap<String, Arc<dyn Texture + Sync + Send>> =
            s["textures"].as_object().unwrap().iter().map(
                |(name, value)| {
                    let o = value.as_object().unwrap();
                    let p = o["params"].as_array().unwrap();
                    (
                        name.clone(),
                        match o["type"].as_str() {
                            Some("Checker") => {
                                texture::Checker::new(
                                    from_value(p[0].clone()).unwrap(),
                                    from_value(p[1].clone()).unwrap()
                                ).into()
                            },
                            _ => panic!("remove me")
                        }
                     )
                }
            ).collect();

        let materials: HashMap<String, Arc<dyn Material + Sync + Send>> =
            s["materials"].as_object().unwrap().iter().map(
                |(name, value)| {
                    let o = value.as_object().unwrap();
                    (
                        name.clone(),
                        match o["type"].as_str() {
                            Some("Lambertian") => {
                                let t = if let Some(t) = o.get("texture") {
                                    textures[t.as_str().unwrap()].clone()
                                } else {
                                    texture::SolidColor::new(
                                        from_value(o["color"].clone()).unwrap()
                                    ).into()
                                };
                                material::Lambertian::from_texture(t).into()
                            }
                            Some(x) => panic!("Not implemented: {}", x),
                            None => panic!("Material without type")
                        }
                    )
                }
            ).collect();

        let mut world = HittableList::new(
            s["world"].as_array().unwrap().iter().map(
                |obj| {
                    let obj = obj.as_object().unwrap();
                    let mat = obj["material"].as_str().unwrap();
                    match obj["type"].as_str() {
                        Some("Sphere") => {
                            Sphere::new(
                                from_value(obj["center"].clone()).unwrap(),
                                from_value(obj["radius"].clone()).unwrap(),
                                materials[mat].clone()
                            ).into()
                        },
                        Some("AARect") => {
                            let b = obj["bounds"].as_array().unwrap();
                            let ctor = match obj["orientation"].as_str().unwrap() {
                                "yz" => AARect::yz_rect,
                                "xz" => AARect::xz_rect,
                                "xy" => AARect::xy_rect,
                                e => panic!("Invalid orientation: {}", e)
                            };
                            ctor(
                                from_value(b[0].clone()).unwrap(),
                                from_value(b[1].clone()).unwrap(),
                                from_value(b[2].clone()).unwrap(),
                                from_value(b[3].clone()).unwrap(),
                                from_value(obj["height"].clone()).unwrap(),
                                materials[mat].clone()
                            ).into()
                        },
                        Some(x) => panic!("Not implemented: {}", x),
                        None => panic!("Hittable missing type"),
                    }
                }
            ).collect()
        );

        let lights = HittableList::new(
            s["lights"].as_array().unwrap().iter().map(
                |obj| {
                    let obj = obj.as_object().unwrap();
                    let mat = material::DiffuseLight::new(
                        Color(1.0, 1.0, 1.0) *
                            from_value::<f64>(obj["intensity"].clone()).unwrap());
                     match obj["type"].as_str() {
                        Some("AARect") => {
                            let b = obj["bounds"].as_array().unwrap();
                            let ctor = match obj["orientation"].as_str().unwrap() {
                                "yz" => AARect::yz_rect,
                                "xz" => AARect::xz_rect,
                                "xy" => AARect::xy_rect,
                                e => panic!("Invalid orientation: {}", e)
                            };
                            let h : Arc<dyn Hittable + Sync + Send> = ctor(
                                from_value(b[0].clone()).unwrap(),
                                from_value(b[1].clone()).unwrap(),
                                from_value(b[2].clone()).unwrap(),
                                from_value(b[3].clone()).unwrap(),
                                from_value(obj["height"].clone()).unwrap(),
                                mat.into()
                            ).into();
                            world.add(FlipFace::new(h.clone()).into());
                            h
                        },
                        Some(x) => panic!("Not implemented: {}", x),
                        None => panic!("Light missing type"),
                    }
                }
            ).collect()
        );

        Ok(Scene{
            lookfrom, lookat, background, vfov,
            world: world.into(),
            lights: lights.into()
        })
    }
}

pub mod defs {
    use crate::scene::Scene;
    use crate::vec3::{Point3,Color,Vec3};
    use crate::texture;
    use crate::texture::Texture;
    use crate::material::{
        Dielectric, DiffuseLight, Lambertian,
        Material, Metal, Corroded,
        AnisotropicPhong,
    };
    use crate::sphere::Sphere;
    use crate::cylinder::Cylinder;
    use crate::disk::Disk;
    use crate::hit::{Hittable, FlipFace, Rotate, Translate};
    use crate::aarect::AARect;
    use crate::hittable_list::HittableList;
    use crate::bvh::BVHNode;
    use crate::boxx::Boxx;
    use crate::moving_sphere::MovingSphere;
    use crate::constant_medium::ConstantMedium;
    use crate::util::random;
    use crate::obj::WfObject;
    use std::sync::Arc;

    const RED: Color = Color(0.65, 0.05, 0.05);
    const WHITE: Color  = Color(0.73, 0.73, 0.73);
    const GREEN: Color  = Color(0.12, 0.45, 0.15);
    const COPPER: Color = Color(184.0/256., 115.0/256., 51.0/256.);

    #[must_use]
    pub fn random_scene() -> Scene {
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;
        let background = Color(0.7, 0.8, 1.0);
        let mut world = HittableList::default();

        let checker: Arc<dyn Texture + Sync + Send> = 
            texture::Checker::new(Color(0.2, 0.3, 0.1), Color(0.9, 0.9, 0.9)).into();

        let ground_material: Arc<dyn Material + Sync + Send> =
            Lambertian::from_texture(checker.clone()).into();
        world.add(Sphere::new(
            Point3(0., -1000., 0.),
            1000.,
            ground_material.clone()
        ).into());

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
                            MovingSphere::new(
                                center,
                                center + Vec3(0.0, random::double_range(0.0, 0.5), 0.0),
                                0.0, 1.0, // times
                                0.2,      // radius
                                Lambertian::new(Color::random() * Color::random()).into(),
                            ).into()
                        },
                        j if j < 0.95 => {
                            Sphere::new(
                                center, 0.2,
                                Metal::new(
                                    Color::random_range(0.5, 1.),
                                    random::double_range(0., 0.5),
                                ).into()
                            ).into()
                        },
                        _ => {
                            Sphere::new(
                                center, 0.2,
                                Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
                            ).into()
                        }
                    }
                    );
                }
            }
        }
        world.add(Sphere::new(
            Point3(0., 1., 0.), 1.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
        ).into());

        world.add(Sphere::new(
            Point3(-4., 1., 0.), 1.0,
            Lambertian::new(Color(0.4, 0.2, 0.1)).into()
        ).into());

        world.add(Sphere::new(
            Point3(4., 1., 0.), 1.0,
            Metal::new(Color(0.7, 0.6, 0.5), 0.0).into()
        ).into());

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&world, 0.0, 1.0).into(),
            lights: HittableList::new(vec![]).into(),
        }
    }

    #[must_use]
    pub fn two_spheres() -> Scene {
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let background = Color(0.7, 0.8, 1.0);
        let vfov = 20.0;

        let checker: Arc<dyn Texture + Sync + Send> = texture::Checker::new(
            Color(0.2, 0.3, 0.1),
            Color(0.9, 0.9, 0.9),
        ).into();

        let world = HittableList {
            objects: vec![
                Sphere::new(
                    Point3(0.0, -10.0, 0.0),
                    10.0,
                    Lambertian::from_texture(checker.clone()).into(),
                ).into(),
                Sphere::new(
                    Point3(0.0, 10.0, 0.0),
                    10.0,
                    Lambertian::from_texture(checker.clone()).into(),
                ).into(),
            ],
        };
        Scene {
            lookfrom, lookat, background, vfov, 
            world: BVHNode::new(&world, 0.0, 1.0).into(),
            lights: HittableList::new(vec![]).into(),
        }
    }

    #[must_use]
    pub fn empty_cornell_box() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(278.0, 278.0, -800.0);
        let lookat = Point3(278.0, 278.0, 0.0);
        let vfov = 40.0;

        let red: Arc<dyn Material + Sync + Send> =
            Lambertian::new(RED).into();
        let white: Arc<dyn Material + Sync + Send> =
            Lambertian::new(WHITE).into();
        let green: Arc<dyn Material + Sync + Send> =
            Lambertian::new(GREEN).into();
        let light: Arc<dyn Material + Sync + Send> =
            DiffuseLight::new(Color(15.0, 15.0, 15.0)).into();

        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xz_rect(
            213.0, 343.0, 227.0, 332.0, 554.0, light.clone()
        ).into();

        let world = HittableList::new(
            vec![
                AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
                ).into(),
                AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
                ).into(),
                AARect::xz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, white.clone()
                ).into(),
                AARect::xz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
                ).into(),
                AARect::xy_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
                ).into(),
                FlipFace::new(light_panel.clone()).into(),
            ]
        );

        Scene {
            lookfrom, lookat, background, vfov, world: world.into(),
            lights: HittableList::new(vec![light_panel]).into(),
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn cornell_box() -> Scene {

        let aluminum: Arc<dyn Material + Sync + Send> =
            Metal::new(Color(0.8, 0.85, 0.88), 0.0).into();
        let white: Arc<dyn Material + Sync + Send> =
            Lambertian::new(WHITE).into();
        let green: Arc<dyn Material + Sync + Send> =
            Lambertian::new(GREEN).into();
        let lavender: Arc<dyn Material + Sync + Send> =
            Lambertian::new(Color(191.0 / 256.0, 64.0 / 256.0, 191.0 / 256.0)).into();
        let mir: Arc<dyn Material + Sync + Send> =
            Metal::new(WHITE, 0.0).into();

        let mut box1 : Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 330.0, 165.0),
            white.clone(),
        ).into();
        box1 = Rotate::rotate_y(box1, 15.0).into();
        box1 = Translate::new(box1, Vec3(265.0, 0.0, 295.0)).into();

        let mut box2: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 165.0, 165.0),
            white.clone(),
        ).into();
        box2 = Rotate::rotate_y(box2, -18.0).into();
        box2 = Translate::new(box2, Vec3(130.0, 0.0, 65.0)).into();

        // let mut tri: Arc<dyn Hittable + Sync + Send> = Triangle::new(
        //     Point3(0.0, 0.0, 0.0),
        //     Point3(0.0, 70.0, 0.0),
        //     Point3(70.0, 0.0, 0.0),
        //     lavender.clone(),
        // ).into();

        // tri = Rotate::rotate_x(tri, 60.0).into();
        // tri = Translate::new(tri, Vec3(200.0, 250.0, 200.0)).into();

        let cbox = empty_cornell_box();

        let world = HittableList::new(vec![
                box1, box2, // tri,
                cbox.world,
        ]);

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: cbox.lights,
        }
    }

    #[must_use]
    pub fn cornell_sphere() -> Scene {
        let cbox = empty_cornell_box();

        let white: Arc<dyn Material + Sync + Send> = Lambertian::new(WHITE).into();
        // let copper = Lambertian::new(COPPER);
        // let copper = Metal::new(Color(0.7, 0.6, 0.5), 0.0);

        let earth: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Image::new("earthmap.jpg").into()
        ).into();

        let mut copper: Arc<dyn Material + Sync + Send> = Corroded::new(10.0,
            Metal::new(COPPER, 0.7).into()
        ).into();
        copper = Corroded::new(10.0,
            Lambertian::new(COPPER).into()
        ).into();
        let glass: Arc<dyn Material + Sync + Send> = Corroded::new(10.0,
            Dielectric::new(1.5, 0.005, COPPER).into()
        ).into();
        copper = Lambertian::new(COPPER).into();
        // copper = Metal::new(COPPER, 0.2).into();

        let pertext: Arc<dyn Texture + Sync + Send> =
            texture::Marble::with_point_scaling(1.0, 10.0).into();
        let stone = Corroded::new(10.0, Lambertian::from_texture(pertext.clone()).into());
        // let stone = Lambertian::from_texture(pertext.clone());
        let voronoi: Arc<dyn Texture + Sync + Send> = texture::Voronoi::new(
            Color(1.0, 1.0, 1.0), 200
        ).into();

        let light: Arc<dyn Material + Sync + Send> =
            DiffuseLight::new(Color(15.0, 15.0, 15.0)).into();

        let mut box1: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 330.0, 165.0),
            white.clone(),
        ).into();
        box1 = Rotate::rotate_y(box1, 15.0).into();
        box1 = Translate::new(box1, Vec3(265.0, 0.0, 295.0)).into();

        let phong: Arc<dyn Material + Sync + Send> = AnisotropicPhong::new(
            texture::SolidColor::new(COPPER).into(),
            texture::SolidColor::new(RED).into(),
            10.0, 500.0,
        ).into();

        let mut sphere: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(0.0, 0.0, 0.0), 90.0,
            // light.clone(),
            // copper.clone(),
            phong.clone(),
        ).into();

        // sphere = Rotate::rotate_y(sphere, 180.).into();
        sphere = Translate::new(sphere,Point3(190.0, 90.0, 190.0)).into();

        let mut cylinder: Arc<dyn Hittable + Sync + Send> = HittableList::new(vec![
            Cylinder::new(90.0, -90.0, 90.0, copper.clone()).into(),
            Disk::new(80.0, 20.0, 90.0, copper.clone()).into(),
        ]).into();

        // cylinder = Rotate::rotate_x(cylinder, -45.0).into();
        cylinder = Translate::new(cylinder, Point3(190.0, 90.0, 190.0)).into();

        let world = HittableList::new(vec![
            box1,
            // cylinder.clone(),
            sphere.clone(),
            cbox.world,
        ]);

        let lights = HittableList::new(vec![
            cbox.lights.clone(),
            // sphere.clone(),
        ]);

        Scene {
            lookfrom: cbox.lookfrom  // + Point3(0.0, 0.0, 600.0)
                ,
            lookat: cbox.lookat // - Point3(50.0, 100.0, 0.0)
                ,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: lights.into(),
        }
    }

    #[must_use]
    pub fn two_perlin_spheres() -> Scene {
        let background = Color(0.7, 0.8, 1.0);
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;

        let copper = Corroded::new(1.0,
            Metal::new(COPPER, 0.0).into()
        );
        // let copper = Metal::new(COPPER, 0.0);
        // let copper = Corroded::new(
        //     Lambertian::new(COPPER).into()
        // );
        // let copper = Corroded::new(
        //     Dielectric::new(0.5, 0.4, COPPER).into()
        // );

        let pertext: Arc<dyn Texture + Sync + Send> = texture::Marble::new(4.).into();
        // let copper = Corroded::new(
        //     Lambertian::from_texture(pertext.clone()).into()
        // );

        let world = HittableList {
            objects: vec![
                Sphere::new(
                    Point3(0.0, -1000.0, 0.0),
                    1000.0,
                    Lambertian::from_texture(pertext.clone()).into()
                ).into(),
                Sphere::new(
                    Point3(0.0, 2.0, 0.0),
                    2.0,
                    // Lambertian::from_texture(pertext.clone()).into()
                    copper.into()
                    // // Corroded::new(Lambertian::from_texture(pertext.clone()).into()).into()
                ).into(),
            ]
        };

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&world, 0.0, 1.0).into(),
            lights: HittableList::new(vec![]).into(),
        }
    }

    #[must_use]
    pub fn earth() -> Scene {
        let background = Color(0.7, 0.8, 1.0);
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;

        let earth_texture: Arc<dyn Texture + Sync + Send> = texture::Image::new(
            "earthmap.jpg"
        ).into();

        let world = HittableList {
            objects: vec![
                Sphere::new(
                    Point3(0.0, 0.0, 0.0),
                    2.0,
                    Lambertian::from_texture(earth_texture.clone()).into()
                ).into(),
            ],
        };

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&world, 0.0, 1.0).into(),
            lights: HittableList::new(vec![]).into(),
        }
    }

    #[must_use]
    pub fn simple_light() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(26.0, 3.0, 6.0);
        let lookat = Point3(0.0, 2.0, 0.0);
        let vfov = 20.0;

        let pertext: Arc<dyn Texture + Sync + Send> = texture::Marble::new(4.).into();
        // let difflight = DiffuseLight::new(Color(4., 4., 4.));
        let difflight: Arc<dyn Material + Sync + Send> =
            DiffuseLight::new(Color(7., 7., 7.)).into();

        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xy_rect(
            3.0, 5.0, 1.0, 3.0, -2.0,
            difflight.clone(),
        ).into();

        let world = HittableList {
            objects: vec![
                Sphere::new(
                    Point3(0.0, -1000.0, 0.0),
                    1000.0,
                    Lambertian::from_texture(pertext.clone()).into()
                ).into(),
                Sphere::new(
                    Point3(0.0, 2.0, 0.0),
                    2.0,
                    Lambertian::from_texture(pertext.clone()).into()
                ).into(),
                light_panel.clone(),
            ]
        };

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&world, 0.0, 1.0).into(),
            lights: HittableList::new(vec![light_panel]).into(),
        }
    }

    #[must_use]
    pub fn cornell_smoke() -> Scene {
        let white: Arc<dyn Material + Sync + Send> = Lambertian::new(WHITE).into();

        let cbox = empty_cornell_box();

        let mut box1: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 330.0, 165.0),
            white.clone(),
        ).into();
        box1 = Rotate::rotate_y(box1, 15.0).into();
        box1 = Translate::new(box1, Vec3(265.0, 0.0, 295.0)).into();

        let mut box2: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 165.0, 165.0),
            white.clone(),
        ).into();
        box2 = Rotate::rotate_y(box2, -18.0).into();
        box2 = Translate::new(box2, Vec3(130.0, 0.0, 65.0)).into();

        box1 = ConstantMedium::new(box1, 0.01, Color::new()).into();
        box2 = ConstantMedium::new(box2, 0.01, Color(1.0, 1.0, 1.0)).into();

        let world = HittableList::new(vec![
                box1, box2,
                cbox.world,
        ]);

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: cbox.lights,
        }
    }

    #[must_use]
    pub fn fancy_random_scene() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(13.0, 2.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 20.0;

        let mut world = HittableList::default();

        let checker: Arc<dyn Texture + Sync + Send> = texture::Checker::new(
            Color(0.2, 0.3, 0.1),
            Color(0.9, 0.9, 0.9)
        ).into();

        let ground_material: Arc<dyn Material + Sync + Send> =
            Lambertian::from_texture(checker.clone()).into();

        world.add(Sphere::new(
            Point3(0., -1000., 0.),
            1000.,
            ground_material.clone()
        ).into());

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
                            Sphere::new(
                                center + Vec3(0.0, rand_off, 0.0),
                                0.2 + rand_off,      // radius
                                Lambertian::new(Color::random() * Color::random()).into(),
                            ).into()
                        },
                        j if j < 0.95 => {
                            Sphere::new(
                                center, 0.2,
                                Metal::new(
                                    Color::random_range(0.5, 1.),
                                    random::double_range(0., 0.5),
                                ).into()
                            ).into()
                        },
                        _ => {
                            Sphere::new(
                                center, 0.2,
                                Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
                            ).into()
                        }
                    }
                    );
                }
            }
        }

        world.add(Sphere::new(
            Point3(0., 1., 0.), 1.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
        ).into());

        world.add(Sphere::new(
            Point3(-4., 1., 0.), 1.0,
            Lambertian::new(Color(0.4, 0.2, 0.1)).into()
        ).into());

        world.add(Sphere::new(
            Point3(4., 1., 0.), 1.0,
            Metal::new(Color(0.7, 0.6, 0.5), 0.0).into()
        ).into());

        let _light_sphere: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(3.0, 2.0, 2.0), 0.6,
            DiffuseLight::new(Color(30.0, 30.0, 30.0)).into()
        ).into();

        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xz_rect(
            -6.0, 6.0, -6.0, 6.0, 10.0,
            DiffuseLight::new(Color(9.0, 4.0, 9.0)).into()
        ).into();

        // world.add(light_sphere.clone());
        world.add(FlipFace::new(light_panel.clone()).into());

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&world, 0.0, 1.0).into(),
            lights: HittableList::new(vec![// light_sphere, 
                                           light_panel]).into()
        }
    }

    #[must_use]
    pub fn final_scene() -> Scene {
        const BOXES_PER_SIDE : i32 = 20;

        let background = Color::new();
        let lookfrom = Point3(478.0, 278.0, -600.0);
        let lookat = Point3(278.0, 278.0, 0.0);
        let vfov = 40.0;

        let mut boxes1 = HittableList::default();
        let ground: Arc<dyn Material + Sync + Send> = Lambertian::new(Color(0.48, 0.83, 0.53)).into();

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

                boxes1.add(Boxx::new(p0, p1, ground.clone()).into());
            }
        };

        let mut objects = HittableList::default();

        objects.add(BVHNode::new(&boxes1, 0.0, 1.0).into());

        let light: Arc<dyn Material + Sync + Send> = DiffuseLight::new(Color(7.0, 7.0, 7.0)).into();
        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xz_rect(
            123.0, 423.0, 147.0, 412.0, 554.0, light.clone()
        ).into();

        objects.add(FlipFace::new(light_panel.clone()).into());

        let center1 = Point3(400.0, 400.0, 200.0);
        let center2 = center1 + Vec3(30.0, 0.0, 0.0);
        let moving_sphere_mat: Arc<dyn Material + Sync + Send> = Lambertian::new(
            Color(0.7, 0.3, 0.1)
        ).into();
        objects.add(MovingSphere::new(
            center1, center2, 0.0, 1.0, 50.0, moving_sphere_mat.clone()
        ).into());


        objects.add(Sphere::new(
            Point3(260.0, 150.0, 45.0), 50.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
        ).into());
        objects.add(Sphere::new(
            Point3(0.0, 150.0, 145.0), 50.0,
            Metal::new(Color(0.8, 0.8, 0.9), 1.0).into()
        ).into());

        let boundary: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(360.0, 150.0, 145.0), 70.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
        ).into();
        objects.add(boundary.clone());
        objects.add(ConstantMedium::new(
            boundary.clone(), 0.2, Color(0.2, 0.4, 0.9),
        ).into());
        let boundary: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3::new(), 5000.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into(),
        ).into();
        objects.add(ConstantMedium::new(
            boundary.clone(), 0.0001, Color(1.0, 1.0, 1.0),
        ).into());

        let emat: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Image::new("earthmap.jpg").into()
        ).into();
        objects.add(Sphere::new(
            Point3(400.0, 200.0, 400.0), 100.0, emat.clone(),
        ).into());

        let pertext: Arc<dyn Texture + Sync + Send> = texture::Marble::new(0.1).into();
        objects.add(Sphere::new(
            Point3(220.0, 280.0, 300.0), 80.0,
            Lambertian::from_texture(pertext.clone()).into()
        ).into());

        let mut boxes2 = HittableList::default();
        let white: Arc<dyn Material + Sync + Send> = Lambertian::new(Color(0.73, 0.73, 0.73)).into();
        let ns = 1000;
        for _j in 0..ns {
            boxes2.add(Sphere::new(
                Point3::random_range(0.0, 165.0), 10.0, white.clone(),
            ).into());
        }

        objects.add(Translate::new(
            Rotate::rotate_y(
                BVHNode::new(&boxes2, 0.0, 1.0).into(), 15.0
            ).into(),
            Vec3(-100.0, 270.0, 395.0)).into()
        );

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&objects, 0.0, 1.0).into(),
            lights: HittableList::new(vec![light_panel]).into(),
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn wacky_cornell_box() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(278.0, 278.0, -800.0);
        let lookat = Point3(278.0, 278.0, 0.0);
        let vfov = 40.0;

        let red: Arc<dyn Material + Sync + Send> = Lambertian::new(
            Color(0.65, 0.05, 0.05)
        ).into();
        let white: Arc<dyn Material + Sync + Send> = Lambertian::new(
            Color(0.73, 0.73, 0.73)
        ).into();
        let green: Arc<dyn Material + Sync + Send> = Lambertian::new(
            Color(0.12, 0.45, 0.15)
        ).into();
        let light: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(5.0, 5.0, 5.0)
        ).into();
        let mirror_back: Arc<dyn Material + Sync + Send> = Metal::new(
            Color(0.73, 0.73, 0.73), 0.0
        ).into();
        let mirror_front: Arc<dyn Material + Sync + Send> =
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into();
        let earth: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Image::new("earthmap.jpg").into()
        ).into();
        let wood: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Wood::new(Vec3(4.0, 0.1, 1.0), Color(0.7, 0.3, 0.1)).into()
        ).into();

        let voronoi_texture: Arc<dyn Texture + Sync + Send> = texture::Voronoi::new(
            Color(1.0, 1.0, 1.0), 200
        ).into();
        let voronoi: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            voronoi_texture.clone()
        ).into();

        let fun_noise: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Noise::from_texture(voronoi_texture.clone()).into()
        ).into();
            
        // let mirror = Metal::new(Color(1.0, 1.0, 1.0), 0.0);

        let mut box1: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 330.0, 165.0),
            white.clone(),
        ).into();
        box1 = Rotate::rotate_y(box1, 15.0).into();
        box1 = Translate::new(box1, Vec3(265.0, 0.0, 295.0)).into();

        let mut box2: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(0.0, 0.0, 0.0),
            Point3(165.0, 165.0, 165.0),
            fun_noise.clone(),
        ).into();
        box2 = Rotate::rotate_y(box2, -18.0).into();
        box2 = Translate::new(box2, Vec3(130.0, 0.0, 65.0)).into();

        let mut mirror: Arc<dyn Hittable + Sync + Send> = HittableList::new(vec![
            AARect::xy_rect(
                113.0, 443.0, 127.0, 432.0, 554.0, mirror_back.clone()
            ).into(),
            // AARect::xy_rect(
            //     113.0, 443.0, 127.0, 432.0, 553.99, mirror_front.clone()
            // ),
        ]).into();

        mirror = Translate::new(mirror, Vec3(-100.0, 0.0, 0.0)).into();
        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xz_rect(
            113.0, 443.0, 127.0, 432.0, 554.0, light.clone()
        ).into();
        // AARect::xz_rect(
        //     013.0, 543.0, 027.0, 532.0, 554.0, light.clone()
        // ),

        let world = HittableList::new(
            vec![
                AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, green.clone()
                ).into(),
                AARect::yz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, red.clone()
                ).into(),
                mirror,
                FlipFace::new(light_panel.clone()).into(),
                AARect::xz_rect(
                    0.0, 555.0, 0.0, 555.0, 0.0, wood.clone()
                ).into(),
                AARect::xz_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
                ).into(),
                AARect::xy_rect(
                    0.0, 555.0, 0.0, 555.0, 555.0, white.clone()
                ).into(),
                box1,
                box2,
            ]
        );

        Scene {
            lookfrom, lookat, background, vfov,
            world: world.into(),
            lights: HittableList::new(vec![light_panel]).into(),
        }
    }

    #[must_use]
    pub fn subsurface_perlin_spheres() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(13.0, 5.0, 3.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 40.0;

        let pertext: Arc<dyn Texture + Sync + Send> = texture::Marble::new(4.0).into();
        let turq_light: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(0.0, 12., 10.)
        ).into();
        let red_light: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(12.0, 0.0, 5.0)
        ).into();

        let mut objects = HittableList::default();

        objects.add(Sphere::new(
            Point3(0.0, -1000.0, 0.0),
            999.5,
            Lambertian::from_texture(pertext.clone()).into()
        ).into());

        objects.add(Sphere::new(
            Point3(0.0, 2.0, 0.0),
            2.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
        ).into());

        objects.add(Sphere::new(
            Point3(0.0, 2.0, 0.0),
            1.5,
            Lambertian::from_texture(pertext.clone()).into()
        ).into());

        let turq_sphere: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(6.0, 4.0, -4.0),
            2.0,
            turq_light.clone(),
        ).into();

        let red_sphere: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(-3.0, 3.0, 4.0),
            1.0,
            red_light.clone(),
        ).into();

        objects.add(turq_sphere.clone());
        objects.add(red_sphere.clone());

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&objects, 0.0, 1.0).into(),
            lights: HittableList::new(vec![turq_sphere, red_sphere]).into(),

        }
    }

    #[must_use]
    pub fn solids() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(-4.0, 4.0, 15.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 40.0;

        let wood: Arc<dyn Texture + Sync + Send> = texture::Wood::new(
            Vec3(4.0, 0.1, 1.0), Color(0.7, 0.3, 0.1)
        ).into();
        let light: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(7.0, 7.0, 7.0)
        ).into();
        let sunlight: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(20.0, 15.5, 11.0)
        ).into();
        let white: Arc<dyn Material + Sync + Send> = Metal::new(
            Color(0.73, 0.73, 0.73), 0.95
        ).into();

        let mut objects = HittableList{
            objects: vec![
                AARect::yz_rect(
                    -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
                ).into(),
                AARect::yz_rect(
                    -10.0, 10.0, -10.0, 10.0, 10.0, white.clone(),
                ).into(),
                AARect::xz_rect(
                    -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
                ).into(),
                AARect::xz_rect(
                    -10.0, 10.0, -10.0, 10.0, 10.0, white.clone(),
                ).into(),
                AARect::xy_rect(
                    -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
                ).into(),
            ]
        };

        let panel: Arc<dyn Hittable + Sync + Send> =
            AARect::xz_rect(-3.0, 3.0, -2.0, 2.0, -3.0, light.clone()).into();
        let mut right_panel: Arc<dyn Hittable + Sync + Send> =
            Rotate::rotate_z(panel.clone(), 45.0).into();
        right_panel = Translate::new(right_panel.clone(), Point3(2.0, 0.0, 0.0)).into();
        objects.add(right_panel.clone());

        // TODO(oren): this panel has the weird black section issue that we sometimes
        // see on rotated objects. Don't really know what the deal is there, but this
        // one is an excellent example. I think it has something to do with the sign
        // of the surface normal
        let mut left_panel: Arc<dyn Hittable + Sync + Send> =
            Rotate::rotate_z(panel.clone(), 315.0).into();
        left_panel = Translate::new(left_panel.clone(), Point3(-2.0, 0.0, 0.0)).into();
        objects.add(left_panel.clone());

        let mut front_panel: Arc<dyn Hittable + Sync + Send> =
            Rotate::rotate_x(panel.clone(), -45.0).into();
        front_panel = Translate::new(front_panel.clone(), Point3(1.0, 1.0, 4.0)).into();
        objects.add(front_panel.clone());

        let sun: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(0.0, 3.5, 0.0), 1.0,
            sunlight.clone()
        ).into();

        objects.add(sun.clone());

        let sphere: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(0.0, 0.0, 0.0), 2.0,
            Lambertian::from_texture(wood.clone()).into()
        ).into();

        objects.add(sphere);

        let boundary: Arc<dyn Hittable + Sync + Send> = Sphere::new(
            Point3(3.0, 3.0, 0.0), 1.0,
            Dielectric::new(1.5, 0.0, Color(1.0, 1.0, 1.0)).into()
        ).into();
        objects.add(boundary.clone());
        objects.add(ConstantMedium::new(
            boundary.clone(), 1.0, Color(0.2, 0.4, 0.9),
        ).into());

        // let mut block = Boxx::new(
        //     Point3(-2.0, -2.0, -2.0), Point3(2.0, 2.0, 2.0),
        //     Lambertian::from_texture(wood.clone()),
        // );

        // block = Rotate::rotate_y(block, 18.0);
        // block = Rotate::rotate_z(block, 10.0);

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&objects, 0.0, 1.0).into(),
            lights: HittableList::new(
               vec![sun, right_panel, left_panel, front_panel]
            ).into(),
        }
    }

    #[allow(unused)]
    #[must_use]
    pub fn noise_experiments() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(0.0, 0.0, 18.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 60.0;

        let wood: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Wood::new(Vec3(4.0, 0.1, 1.0), Color(0.7, 0.3, 0.1)).into()
        ).into();
        let marble: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            texture::Marble::new(4.).into()
        ).into();
        let aluminum: Arc<dyn Material + Sync + Send> = Metal::new(
            Color(0.8, 0.85, 0.88), 0.0
        ).into();
        let voronoi: Arc<dyn Texture + Sync + Send> = texture::Voronoi::new(
            Color(1.0, 1.0, 1.0), 200
        ).into();
        let fun_noise: Arc<dyn Texture + Sync + Send> = texture::Noise::from_texture(
            voronoi.clone()
        ).into();
        let noise: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            fun_noise.clone()
        ).into();
        let light: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(10.0, 10.0, 10.0)
        ).into();
        let sunlight: Arc<dyn Material + Sync + Send> = DiffuseLight::new(
            Color(20.0, 15.5, 11.0)
        ).into();
        let white: Arc<dyn Material + Sync + Send> = Lambertian::new(
            Color(0.73, 0.73, 0.73)
        ).into();
        let green: Arc<dyn Material + Sync + Send> = Lambertian::new(
            Color(0.12, 0.45, 0.15)
        ).into();

        let fun: Arc<dyn Material + Sync + Send> = Lambertian::from_texture(
            voronoi.clone()
        ).into();

        let mut objects = HittableList{
            objects: vec![
                AARect::yz_rect(
                    -10.0, 10.0, -10.0, 10.0, -10.0, green.clone(),
                ).into(),
                AARect::yz_rect(
                    -10.0, 10.0, -10.0, 10.0, 10.0, green.clone(),
                ).into(),
                AARect::xz_rect(
                    -10.0, 10.0, -10.0, 10.0, -10.0, fun.clone(),
                ).into(),
                AARect::xz_rect(
                    -10.0, 10.0, -10.0, 10.0, 10.0, white.clone(),
                ).into(),
                AARect::xy_rect(
                    -10.0, 10.0, -10.0, 10.0, -10.0, white.clone(),
                ).into(),
            ]
        };

        let panel: Arc<dyn Hittable + Sync + Send> =
            AARect::xz_rect(-5.0, 5.0, -5.0, 5.0, 9.99, light.clone()).into();
        objects.add(FlipFace::new(panel.clone()).into());

        // panel = 
        //     AARect::xz_rect(-5.0, 5.0, -5.0, 5.0, -9.99, light.clone());
        let mut panel2: Arc<dyn Hittable + Sync + Send> = 
            AARect::yz_rect(-2.0, 2.0, -2.0, 2.0, 2.1, light.clone()).into();
        panel2 = Rotate::rotate_y(panel2, -45.0).into();
        panel2 = Translate::new(panel2, Point3(2.0, 0.0, 2.0)).into();
        // objects.add(FlipFace::new(panel2.clone()));

        let mut block: Arc<dyn Hittable + Sync + Send> = Boxx::new(
            Point3(-2.0, -2.0, -2.0),
            Point3(2.0, 2.0, 2.0),
            aluminum.clone(),
            // marble.clone(),
        ).into();

        // block = Rotate::rotate_z(block, -45.0);
        // block = Rotate::rotate_x(block, -90.0);
        block = Rotate::rotate_y(block, 45.0).into();

        // block = Translate::new(block, Point3(4.0, -2.0, -8.0));
        // block = Translate::new(block, Point3(-4.0, 0.0, 0.0));

        objects.add(block);

        Scene {
            lookfrom, lookat, background, vfov,
            world: BVHNode::new(&objects, 0.0, 1.0).into(),
            lights: HittableList::new(vec![
                panel,
                //panel2
            ]).into(),
        }
    }

    #[must_use]
    pub fn teapot() -> Scene {
        let _white: Arc<dyn Material + Sync + Send> = Lambertian::new(WHITE).into();
        let _green: Arc<dyn Material + Sync + Send> = Lambertian::new(GREEN).into();
        let lavender: Arc<dyn Material + Sync + Send> =
            Lambertian::new(Color(191.0 / 256.0, 64.0 / 256.0, 191.0 / 256.0)).into();
        let copper = Corroded::new(20.0,
            Metal::new(COPPER, 0.0).into()
        );
        // let copper = Metal::new(COPPER, 0.0);
        // let copper = Corroded::new(20.,
        //     Lambertian::new(COPPER).into()
        // );
        let copper = Lambertian::new(COPPER);
        // let copper = Lambertian::from_texture(
        //     texture::Marble::new(0.5).into()
        // );

        let cbox = empty_cornell_box();

        let mut teapot: Arc<dyn Hittable + Sync + Send> =
            WfObject::new("data/teapot.obj", 40.0, copper.into()).into();

        teapot = Rotate::rotate_x(teapot, -15.0).into();
        teapot = Rotate::rotate_y(teapot, 15.0).into();

        teapot = Translate::new(
            teapot, Point3(275.0, 200.0, 227.0),
        ).into();

        let world = HittableList::new(vec![
            teapot,
            cbox.world,
        ]);

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: cbox.lights,
        }
    }

    #[must_use]
    pub fn obj_in_cornell_box(fname: &str, scale: f64, translate: Vec3) -> Scene {
        let _white: Arc<dyn Material + Sync + Send> = Lambertian::new(WHITE).into();
        let _green: Arc<dyn Material + Sync + Send> = Lambertian::new(GREEN).into();
        let lavender: Arc<dyn Material + Sync + Send> =
            Lambertian::new(Color(191.0 / 256.0, 64.0 / 256.0, 191.0 / 256.0)).into();

        let cbox = empty_cornell_box();

        let mut obj: Arc<dyn Hittable + Sync + Send> =
        WfObject::new(
            fname, scale, lavender.clone()
        ).into();

        // obj = Rotate::rotate_x(obj, -15.0).into();
        obj = Rotate::rotate_y(obj, 180.0).into();

        obj = Translate::new(obj, translate).into();

        let world = HittableList::new(vec![
            obj,
            cbox.world,
        ]);

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: cbox.lights,
        }
    }

    #[must_use]
    pub fn tree() -> Scene {
        let _white: Arc<dyn Material + Sync + Send> = Lambertian::new(WHITE).into();
        let _green: Arc<dyn Material + Sync + Send> = Lambertian::new(GREEN).into();
        let lavender: Arc<dyn Material + Sync + Send> =
            Lambertian::new(Color(191.0 / 256.0, 64.0 / 256.0, 191.0 / 256.0)).into();

        let cbox = empty_cornell_box();

        let mut tree: Arc<dyn Hittable + Sync + Send> =
            WfObject::new(
                "data/low_poly_tree/Lowpoly_tree_sample.obj", 10.0, lavender.clone()
            ).into();

        tree = Translate::new(
            tree, Point3(272.0, 10.0, 272.0),
        ).into();

        let world = HittableList::new(vec![
            tree,
            cbox.world,
        ]);

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: cbox.lights,
        }
    }

    #[must_use]
    pub fn purple_flower() -> Scene {
        let white: Arc<dyn Material + Sync + Send> = Lambertian::new(WHITE).into();
        let _green: Arc<dyn Material + Sync + Send> = Lambertian::new(GREEN).into();

        let cbox = empty_cornell_box();

        let mut flower: Arc<dyn Hittable + Sync + Send> =
            WfObject::new(
                "data/purple_flower/purple_flower_mm.obj",
                2.0, white.into()).into();

        flower = Rotate::rotate_x(flower, -80.0).into();
        flower = Rotate::rotate_y(flower, 140.0).into();

        flower = Translate::new(
            flower, Point3(272.0, 272.0, 227.0),
        ).into();

        let world = HittableList::new(vec![
            flower,
            cbox.world,
        ]);

        Scene {
            lookfrom: cbox.lookfrom,
            lookat: cbox.lookat,
            background: cbox.background,
            vfov: cbox.vfov,
            world: world.into(),
            lights: cbox.lights,
        }
    }

    pub fn knob1() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(-1.0, 1.8, -5.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 40.0;

        let white: Arc<dyn Material + Sync + Send> =
            Lambertian::new(GREEN).into();

        let light: Arc<dyn Material + Sync + Send> =
            DiffuseLight::new(Color(15.0, 15.0, 15.0)).into();

        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xz_rect(
            -2.0, 2.0, -2.0, 2.0, 4.0, light.clone()
        ).into();

        let mut knob1: Arc<dyn Hittable + Sync + Send> =
            WfObject::new("data/knob1/testObj.obj", 1.0, white.into()).into();

        let world = HittableList::new(vec![
            knob1, FlipFace::new(light_panel.clone()).into()
        ]);

        Scene {
            lookfrom, lookat, background, vfov, world: world.into(),
            lights: HittableList::new(vec![light_panel]).into()
        }
    }

    pub fn knob2() -> Scene {
        let background = Color(0.0, 0.0, 0.0);
        let lookfrom = Point3(2.0, 6.0, 12.0);
        let lookat = Point3(0.0, 0.0, 0.0);
        let vfov = 40.0;

        let white: Arc<dyn Material + Sync + Send> =
            Lambertian::new(GREEN).into();

        let light: Arc<dyn Material + Sync + Send> =
            DiffuseLight::new(Color(15.0, 15.0, 15.0)).into();

        let light_panel: Arc<dyn Hittable + Sync + Send> = AARect::xz_rect(
            -2.0, 2.0, -2.0, 2.0, 10.0, light.clone()
        ).into();

        let mut knob2: Arc<dyn Hittable + Sync + Send> =
            WfObject::new("data/knob2/mitsuba.obj", 1.0, white.into()).into();

        let world = HittableList::new(vec![
            knob2, FlipFace::new(light_panel.clone()).into()
        ]);

        Scene {
            lookfrom, lookat, background, vfov, world: world.into(),
            lights: HittableList::new(vec![light_panel]).into()
        }
    }
}
