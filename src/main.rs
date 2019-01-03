use std::fs;
use std::io::{BufWriter, Write};

mod vector;
use crate::vector::*;

mod figures;
use crate::figures::*;

mod textures;
use crate::textures::*;

mod materials;
use crate::materials::*;

pub struct Objects {
    figure: Figures,
    material: Materials,
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

struct Scene {
    objects: Vec<Objects>,
}

impl Scene {
    pub fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<(HitRecord, &Objects)> {
        let mut closest_parameter = t_max;
        let mut record = None;

        for object in &self.objects {
            if let Some(rec) = object.figure.hit(ray, t_min, closest_parameter) {
                closest_parameter = rec.at;
                record = Some((rec,object));
            }
        }

        record
    }

    pub fn color(&self, ray: Ray, depth: i32) -> V3 {
        match self.hit(&ray, 0.001, std::f32::MAX) {
            Some((rec, object)) => {
                let sc = object.material.scatter(&ray, &rec);
                if depth < 50 && sc.is_scattered {
                    sc.attenuation * self.color(sc.scattered, depth + 1)
                } else {
                    V3(0.0, 0.0, 0.0)
                }
            },
            None => {
                let t = 0.5 * (ray.direction.normalize().y() + 1.0);
                V3(1.0, 1.0, 1.0).scale(1.0 - t) + V3(0.5, 0.7, 1.0).scale(t)
            },
        }
    }
}

struct Camera {
    origin: V3,
    lower_left_corner: V3,
    horizontal: V3,
    vertical: V3,
    lens_radius: f32,
    camera_pose: (V3, V3, V3),
}

impl Camera {
    pub fn new(lookfrom: V3, lookat: V3, vup: V3, vfov: f32, aspect: f32, apertune: f32, focus_dist: f32) -> Camera {
        let lens_radius = apertune / 2.0;
        let theta = vfov * std::f32::consts::PI / 180.0;
        let half_height = (theta / 2.0).tan();
        let half_width = aspect * half_height;
        let w = (lookfrom - lookat).normalize();
        let u = vup.cross(w).normalize();
        let v = w.cross(u);

        Camera {
            origin: lookfrom,
            lower_left_corner: lookfrom - u.scale(half_width * focus_dist) - v.scale(half_height * focus_dist) - w.scale(focus_dist),
            horizontal: u.scale(2.0 * half_width * focus_dist),
            vertical: v.scale(2.0 * half_height * focus_dist),
            lens_radius: lens_radius,
            camera_pose: (u,v,w),
        }
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        let rd = V3::new_in_unit_disk().scale(self.lens_radius);
        let offset = self.camera_pose.0.scale(rd.x()) + self.camera_pose.1.scale(rd.y());

        Ray {
            origin: self.origin + offset,
            direction: self.lower_left_corner + self.horizontal.scale(u) + self.vertical.scale(v) - self.origin - offset
        }
    }
}

fn create_random_scene() -> Scene {
    let mut objects = vec![];
    objects.push(
        Objects {
            figure: Figures::sphere(V3(0.0, -1000.0, 0.0), 1000.0),
            material: Materials::lambertian(Textures::solid(V3(0.5, 0.5, 0.5))),
        }
    );

    for a in -11..11 {
        for b in -11..11 {
            let material = rand::random::<f32>();
            let center = V3(
                a as f32 + 0.9 * rand::random::<f32>(),
                0.2,
                b as f32 + 0.9 * rand::random::<f32>(),
            );

            if (center - V3(4.0, 0.2, 0.0)).norm() > 0.9 {
                if material < 0.8 {
                    objects.push(
                        Objects {
                            figure: Figures::sphere(center, 0.2),
                            material: Materials::lambertian(Textures::solid(V3(
                                rand::random::<f32>() * rand::random::<f32>(),
                                rand::random::<f32>() * rand::random::<f32>(),
                                rand::random::<f32>() * rand::random::<f32>(),
                            )))
                        }
                    );
                } else if material < 0.95 {
                    objects.push(
                        Objects {
                            figure: Figures::sphere(center, 0.2),
                            material: Materials::metal(V3(
                                0.5 * (1.0 + rand::random::<f32>()),
                                0.5 * (1.0 + rand::random::<f32>()),
                                0.5 * (1.0 + rand::random::<f32>()),
                            )
                            , 0.5 * rand::random::<f32>())
                        }
                    );
                } else {
                    objects.push(
                        Objects {
                            figure: Figures::sphere(center, 0.2),
                            material: Materials::dielectric(1.5),
                        }
                    );
                }
            }
        }
    }

    objects.push(
        Objects {
            figure: Figures::sphere(V3(0.0, 1.0, 0.0), 1.0),
            material: Materials::dielectric(1.5),
        }
    );
    objects.push(
        Objects {
            figure: Figures::sphere(V3(-4.0, 1.0, 0.0), 1.0),
            material: Materials::lambertian(Textures::solid(V3(0.4, 0.2, 0.1))),
        }
    );
    objects.push(
        Objects {
            figure: Figures::sphere(V3(4.0, 1.0, 0.0), 1.0),
            material: Materials::metal(V3(0.7, 0.6, 0.5), 0.0),
        }
    );

    Scene {
        objects: objects,
    }
}

fn main() {
    let w = 400;
    let h = 250;
    let ns = 100;

    let lookfrom = V3(13.0, 2.0, 3.0);
    let lookat = V3(0.0, 0.0, 0.0);
    let dist_to_focus = 10.0;
    let apertune = 0.0;

    let camera = Camera::new(lookfrom, lookat, V3(0.0, 1.0, 0.0), 20.0, w as f32 / h as f32, apertune, dist_to_focus);
//    let scene = create_random_scene();
    let scene = Scene {
        objects: vec![
            Objects {
                figure: Figures::sphere(V3(0.0, -1000.0, 0.0), 1000.0),
                material: Materials::lambertian(Textures::noise()),
            },
            Objects {
                figure: Figures::sphere(V3(0.0, 2.0, 0.0), 2.0),
                material: Materials::lambertian(Textures::noise()),
            },
        ]
    };

    let renderer = Renderer {
        renderer: Box::new(move |i,j| {
            let c = (0..ns).map(|_| {
                let u = (i as f32 + rand::random::<f32>()) / w as f32;
                let v = ((h - 1 - j) as f32 + rand::random::<f32>()) / h as f32;
                let ray = camera.get_ray(u,v);

                scene.color(ray, 0)
            }).sum::<V3>().scale(1.0 / ns as f32).map(&|x| x.sqrt());

            Color::from_v3(c)
        }),
        width: w,
        height: h,
    };

    renderer.render("out.ppm");
}

