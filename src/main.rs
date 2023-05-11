#![allow(unused)]

use std::io;
use hawxide::*;

use std::io::{Write, BufWriter};
use std::sync::Arc;
use std::sync::Mutex;
use rayon::prelude::*;

fn ray_color(r : &Ray,
             scene: &Scene,
             depth: i32) -> Color {
    if depth <= 0 {
        return Color(0., 0., 0.);
    }
    if let Some(hr) = scene.world.hit(r, 0.001, INFINITY) {
        let emitted = hr.mat.emitted(r, &hr, hr.u, hr.v, hr.p);
        if let Some(sr) =  hr.mat.scatter(r, &hr) {
            if let Some(spec_r) = sr.specular_ray {
                return sr.attenuation
                    * ray_color(&spec_r, scene, depth - 1);
            }
            let light_pdf = if scene.lights.empty() {
                sr.pdf.clone()
            } else {
                Arc::new(HittablePDF::new(scene.lights.clone(), hr.p))
            };
            let mix_pdf = MixturePDF::new(light_pdf.clone(), sr.pdf.clone());
            let scattered = Ray::new(hr.p, mix_pdf.generate(), r.time);
            let pdf_val = mix_pdf.value(scattered.dir);

            assert!(pdf_val > 0.0, "PDF val {:12} < 0", pdf_val);

            emitted +
                sr.attenuation *
                hr.mat.scattering_pdf(r, &hr, &scattered) *
                ray_color(&scattered, scene, depth-1) /
                pdf_val

        } else {
            emitted
        }
    } else {
        scene.background
    }
}

fn main() {

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

    let mut aspect_ratio : f64 = 16.0 / 9.0;
    let mut image_width : i32 = 400;
    let mut samples_per_pixel : i32 = 400;

    let scene_select: usize = 4;

    let scene = match scene_select {
        1 => {
            aperture = 0.1;
            scene::defs::random_scene()
        },
        2 => scene::defs::two_spheres(),
        3 => {
            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 100;
            scene::defs::cornell_sphere()
        },
        4 => {
            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 100;
            scene::defs::cornell_box()
        },
        5 => scene::defs::two_perlin_spheres(),
        6 => scene::defs::earth(),
        7 => {
            samples_per_pixel = 200;
            scene::defs::simple_light()
        },
        8 => {
            aspect_ratio = 1.0;
            image_width = 600;
            samples_per_pixel = 200;
            scene::defs::cornell_smoke()
        },
        9 => {
            aperture = 0.05;
            samples_per_pixel = 1000;
            scene::defs::fancy_random_scene()
        },
        10 => {
            aspect_ratio = 1.0;
            // image_width = 600;
            image_width = 600;
            samples_per_pixel = 1000;
            scene::defs::wacky_cornell_box()
        },
        11 => {
            samples_per_pixel = 1000;
            scene::defs::subsurface_perlin_spheres()
        },
        12 => scene::defs::solids(),
        13 => scene::defs::noise_experiments(),
        _ => {
            aspect_ratio = 1.0;
            image_width = 400;
            samples_per_pixel = 100;
            scene::defs::final_scene()
        }
    };

    let image_height : i32 = ((image_width as f64) / aspect_ratio) as i32;
    let vup = Vec3(0., 1., 0.);
    let dist_to_focus = 10.0;

    let cam =
        Camera::new(&scene, vup, aspect_ratio, aperture, dist_to_focus, 0.0, 1.0);

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
                    ray_color(&r, &scene, MAX_DEPTH)
            };
            pixel_color
        }).collect();
        for pc in &colors {
            write_color(&mut stdout, *pc, samples_per_pixel);
        }
    }
    write!(stderr, "\nDone\n");
}

/************ PDF Examples *************/

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
