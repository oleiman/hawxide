pub mod vec3;
pub mod util;
pub mod ray;
pub mod hit;
pub mod sphere;
pub mod hittable_list;
pub mod camera;
pub mod material;
pub mod moving_sphere;
pub mod aabb;
pub mod bvh;
pub mod texture;
pub mod perlin;

// TODO(oren): janky, lazy

pub use vec3::Vec3;
pub use vec3::Color;
pub use vec3::write_color;
pub use vec3::Point3;
pub use ray::Ray;
pub use sphere::Sphere;
pub use moving_sphere::MovingSphere;
pub use hittable_list::HittableList;
pub use hit::Hittable;
pub use camera::Camera;
pub use util::INFINITY;
pub use util::random;
pub use material::{Material, Lambertian, Metal, Dielectric};
pub use bvh::BVHNode;
pub use texture::{SolidColor, CheckerTexture, NoiseTexture,ImageTexture};


