extern crate rand;

use std::fs;
use std::io::{BufWriter, Write};

mod vector;
use vector::*;

pub struct Ray {
    origin: V3,
    direction: V3,
}

impl Ray {
    pub fn extend_at(&self, scaler: f32) -> V3 {
        self.origin + self.direction.scale(scaler)
    }
}

struct Color(u8,u8,u8);

impl Color {
    fn red(&self) -> u8 {
        self.0
    }

    fn green(&self) -> u8 {
        self.1
    }

    fn blue(&self) -> u8 {
        self.2
    }

    fn from_f32(r: f32, g: f32, b: f32) -> Color {
        Color(
            (r * 255.99) as u8,
            (g * 255.99) as u8,
            (b * 255.99) as u8,
        )
    }

    fn from_v3(v: V3) -> Color {
        Color::from_f32(v.x(), v.y(), v.z())
    }
}

struct Renderer {
    renderer: Box<Fn(i32,i32) -> Color>,
    width: i32,
    height: i32,
}

impl Renderer {
    fn render(&self, file_name: &str) {
        let mut f = BufWriter::new(fs::File::create(file_name).unwrap());
        f.write(format!("P3\n{} {}\n255\n", self.width, self.height).as_bytes()).unwrap();

        for j in 0..self.height {
            for i in 0..self.width {
                let c = (self.renderer)(i,j);

                f.write(format!(
                    "{} {} {}\n",
                    c.red(),
                    c.green(),
                    c.blue(),
                ).as_bytes()).unwrap();
            }
        }
    }
}

pub struct HitRecord {
    at: f32,
    point: V3,
    normal: V3,
}

pub trait Object {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord>;
}

struct Sphere {
    center: V3,
    radius: f32,
}

impl Object for Sphere {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.square_norm();
        let b = oc.dot(ray.direction);
        let c = oc.square_norm() - self.radius * self.radius;
        let discriminant = b*b - a*c;

        if discriminant > 0.0 {
            let check = |at| {
                if tmin < at && at < tmax {
                    let point = ray.extend_at(at);

                    Some(HitRecord {
                        at: at,
                        point: point,
                        normal: (point - self.center).scale(1.0 / self.radius)
                    })
                } else {
                    None
                }
            };

            check((-b - (b*b-a*c).sqrt()) / a).or(check((-b + (b*b-a*c).sqrt()) / a))
        } else {
            None
        }
    }
}

struct Scene {
    objects: Vec<Box<dyn Object>>
}

impl Scene {
    pub fn color(&self, ray: Ray) -> V3 {
        for object in &self.objects {
            if let Some(rec) = object.hit(&ray, 0.0, std::f32::MAX) {
                return (rec.normal + V3(1.0, 1.0, 1.0)).scale(0.5);
            }
        }

        let t = 0.5 * (ray.direction.normalize().y() + 1.0);
        V3(1.0, 1.0, 1.0).scale(1.0 - t) + V3(0.5, 0.7, 1.0).scale(t)
    }
}

struct Camera {
    origin: V3,
    lower_left_corner: V3,
    horizontal: V3,
    vertical: V3,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            origin: V3(0.0, 0.0, 0.0),
            lower_left_corner: V3(-2.0, -1.0, -1.0),
            horizontal: V3(4.0, 0.0, 0.0),
            vertical: V3(0.0, 2.0, 0.0),
        }
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + self.horizontal.scale(u) + self.vertical.scale(v)
        }
    }
}

fn main() {
    let w = 200;
    let h = 100;
    let ns = 100;

    let camera = Camera::new();
    let scene = Scene {
        objects: vec![
            Box::new(Sphere {
                center: V3(0.0, 0.0, -1.0),
                radius: 0.5,
            }),
            Box::new(Sphere {
                center: V3(0.0, -100.5, -1.0),
                radius: 100.0,
            }),
        ]
    };

    let renderer = Renderer {
        renderer: Box::new(move |i,j| {
            let c = (0..ns).map(|_| {
                let u = (i as f32 + rand::random::<f32>()) / w as f32;
                let v = ((h - 1 - j) as f32 + rand::random::<f32>()) / h as f32;
                let ray = camera.get_ray(u,v);

                scene.color(ray)
            }).sum::<V3>().scale(1.0 / ns as f32);

            Color::from_v3(c)
        }),
        width: w,
        height: h,
    };

    renderer.render("out.ppm");
}

