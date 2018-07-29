use std::fs;
use std::io::{BufWriter, Write};

mod vector;
use vector::*;

struct Ray {
    point: V3,
    direction: V3,
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

fn main() {
    let w = 200;
    let h = 100;

    let lower_left_corner = V3(-2.0, -1.0, -1.0);
    let horizontal = V3(4.0, 0.0, 0.0);
    let vertical = V3(0.0, 2.0, 0.0);
    let origin = V3(0.0, 0.0, 0.0);

    let renderer = Renderer {
        renderer: Box::new(move |i,j| {
            let u = i as f32 / w as f32;
            let v = (h - 1 - j) as f32 / h as f32;
            let ray = Ray {
                point: origin,
                direction: lower_left_corner + horizontal.scale(u) + vertical.scale(v),
            };
            let c = {
                let t = 0.5 * (V3U::new(ray.direction).y() + 1.0);
                V3(1.0, 1.0, 1.0).scale(1.0 - t) + V3(0.5, 0.7, 1.0).scale(t)
            };

            Color::from_v3(c)
        }),
        width: w,
        height: h,
    };

    renderer.render("out.ppm");
}

