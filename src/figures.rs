use crate::vector::*;
use crate::materials::*;

#[derive(Clone)]
pub struct Aabb {
    min: V3,
    max: V3,
}

impl Aabb {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> bool {
        let invD = 1.0 / ray.direction.0;
        let mut t0 = (self.min.0 - ray.origin.0) * invD;
        let mut t1 = (self.max.0 - ray.origin.0) * invD;

        if invD < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let tmin = if t0 > tmin { t0 } else { tmin };
        let tmax = if t1 < tmax { t1 } else { tmax };

        if tmax <= tmin {
            return false;
        }

        let invD = 1.0 / ray.direction.1;
        let mut t0 = (self.min.1 - ray.origin.1) * invD;
        let mut t1 = (self.max.1 - ray.origin.1) * invD;

        if invD < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let tmin = if t0 > tmin { t0 } else { tmin };
        let tmax = if t1 < tmax { t1 } else { tmax };

        if tmax <= tmin {
            return false;
        }

        let invD = 1.0 / ray.direction.2;
        let mut t0 = (self.min.2 - ray.origin.2) * invD;
        let mut t1 = (self.max.2 - ray.origin.2) * invD;

        if invD < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }

        let tmin = if t0 > tmin { t0 } else { tmin };
        let tmax = if t1 < tmax { t1 } else { tmax };

        if tmax <= tmin {
            return false;
        }

        true
    }

    pub fn surround(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: V3(
                self.min.x().min(other.min.x()),
                self.min.y().min(other.min.y()),
                self.min.z().min(other.min.z()),
            ),
            max: V3(
                self.max.x().max(other.max.x()),
                self.max.y().max(other.max.y()),
                self.max.z().max(other.max.z()),
            ),
        }
    }
}

trait Hit {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord>;
    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb>;
}

pub struct Sphere {
    center: V3,
    radius: f32,
}

impl Hit for Sphere {
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
                        normal: (point - self.center).scale(1.0 / self.radius),
                        u: 1.0,
                        v: 1.0,
                    })
                } else {
                    None
                }
            };

            check((-b - discriminant.sqrt()) / a).or(check((-b + discriminant.sqrt()) / a))
        } else {
            None
        }
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        Some(Aabb {
            min: self.center - V3(self.radius, self.radius, self.radius),
            max: self.center + V3(self.radius, self.radius, self.radius),
        })
    }
}

struct XYRect {
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    k: f32,
}

impl Hit for XYRect {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.z()) / ray.direction.z();
        if t < tmin || t > tmax {
            return None;
        }

        let x = ray.origin.x() + t * ray.direction.x();
        let y = ray.origin.y() + t * ray.direction.y();
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }

        Some(HitRecord {
            at: t,
            point: ray.extend_at(t),
            normal: V3(0.0, 0.0, 1.0),
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (y - self.y0) / (self.y1 - self.y0),
        })
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        Some(Aabb {
            min: V3(self.x0, self.y0, self.k - 0.0001),
            max: V3(self.x1, self.y1, self.k + 0.0001),
        })
    }
}

struct YZRect {
    y0: f32,
    y1: f32,
    z0: f32,
    z1: f32,
    k: f32,
}

impl Hit for YZRect {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.x()) / ray.direction.x();
        if t < tmin || t > tmax {
            return None;
        }

        let y = ray.origin.y() + t * ray.direction.y();
        let z = ray.origin.z() + t * ray.direction.z();
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        Some(HitRecord {
            at: t,
            point: ray.extend_at(t),
            normal: V3(0.0, 0.0, 1.0),
            u: (y - self.y0) / (self.y1 - self.y0),
            v: (z - self.z0) / (self.z1 - self.z0),
        })
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        Some(Aabb {
            min: V3(self.k - 0.0001, self.y0, self.z0),
            max: V3(self.k + 0.0001, self.y1, self.z1),
        })
    }
}

struct XZRect {
    x0: f32,
    x1: f32,
    z0: f32,
    z1: f32,
    k: f32,
}

impl Hit for XZRect {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.y()) / ray.direction.y();
        if t < tmin || t > tmax {
            return None;
        }

        let x = ray.origin.x() + t * ray.direction.x();
        let z = ray.origin.z() + t * ray.direction.z();
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }

        Some(HitRecord {
            at: t,
            point: ray.extend_at(t),
            normal: V3(0.0, 0.0, 1.0),
            u: (x - self.x0) / (self.x1 - self.x0),
            v: (z - self.z0) / (self.z1 - self.z0),
        })
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        Some(Aabb {
            min: V3(self.x0, self.k - 0.0001, self.z0),
            max: V3(self.x1, self.k + 0.0001, self.z1),
        })
    }
}

struct FlipNormals {
    figure: Box<Figures>,
}

impl Hit for FlipNormals {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        self.figure.hit(ray, tmin, tmax).map(|mut rec| {
            rec.normal = -rec.normal;
            rec
        })
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        self.figure.bounding_box(t0, t1)
    }
}

struct Cuboid {
    pmin: V3,
    pmax: V3,
    figure: Box<Figures>,
}

impl Cuboid {
    fn new(p0: V3, p1: V3) -> Cuboid {
        Cuboid {
            pmin: p0,
            pmax: p1,
            figure: Box::new(Figures::Figures(vec![
                Figures::xy_rect(p0.x(), p1.x(), p0.y(), p1.y(), p1.z()),
                Figures::flip_normals(Figures::xy_rect(p0.x(), p1.x(), p0.y(), p1.y(), p0.z())),
                Figures::xz_rect(p0.x(), p1.x(), p0.z(), p1.z(), p1.y()),
                Figures::flip_normals(Figures::xz_rect(p0.x(), p1.x(), p0.z(), p1.z(), p0.y())),
                Figures::yz_rect(p0.y(), p1.y(), p0.z(), p1.z(), p1.x()),
                Figures::flip_normals(Figures::yz_rect(p0.y(), p1.y(), p0.z(), p1.z(), p0.x())),
            ]))
        }
    }
}

impl Hit for Cuboid {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        self.figure.hit(ray, tmin, tmax)
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        Some(Aabb {
            min: self.pmin,
            max: self.pmax,
        })
    }
}

struct Translate {
    offset: V3,
    figure: Box<Figures>,
}

impl Hit for Translate {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let moved_ray = Ray { origin: ray.origin - self.offset, direction: ray.direction };
        self.figure.hit(&moved_ray, tmin, tmax).map(|mut rec| {
            rec.point = rec.point + self.offset;
            rec
        })
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        self.figure.bounding_box(t0, t1).map(|mut bbox| {
            bbox = Aabb {
                min: bbox.min,
                max: bbox.max,
            };
            bbox
        })
    }
}

struct RotateY {
    sin_theta: f32,
    cos_theta: f32,
    figure: Box<Figures>,
    bbox: Aabb,
}

impl RotateY {
    fn new(angle: f32, figure: Figures) -> RotateY {
        let radians = (std::f32::consts::PI / 180.0) * angle;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let bbox = figure.bounding_box(0.0, 1.0).unwrap();
        let mut min = V3(std::f32::MAX, std::f32::MAX, std::f32::MAX);
        let mut max = V3(-std::f32::MAX, -std::f32::MAX, -std::f32::MAX);
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f32 * bbox.max.x() + (1.0 - i as f32) * bbox.min.x();
                    let y = j as f32 * bbox.max.y() + (1.0 - j as f32) * bbox.min.y();
                    let z = k as f32 * bbox.max.z() + (1.0 - k as f32) * bbox.min.z();
                    let newx = cos_theta * x + sin_theta * z;
                    let newz = - sin_theta * x + cos_theta * z;

                    let tester = V3(newx, y, newz);
                    max = V3(
                        tester.0.max(max.0),
                        tester.1.max(max.1),
                        tester.2.max(max.2),
                    );
                    min = V3(
                        tester.0.min(min.0),
                        tester.1.min(min.1),
                        tester.2.min(min.2),
                    );
                }
            }
        }

        RotateY {
            sin_theta: sin_theta,
            cos_theta: cos_theta,
            figure: Box::new(figure),
            bbox: Aabb { min: min, max: max },
        }
    }
}

impl Hit for RotateY {
    fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;
        origin.0 = self.cos_theta * ray.origin.0 - self.sin_theta * ray.origin.2;
        origin.2 = self.sin_theta * ray.origin.0 + self.cos_theta * ray.origin.2;
        direction.0 = self.cos_theta * ray.direction.0 - self.sin_theta * ray.direction.2;
        direction.2 = self.sin_theta * ray.direction.0 + self.cos_theta * ray.direction.2;
        let rotated_r = Ray { origin: origin, direction: direction };

        self.figure.hit(&rotated_r, tmin, tmax).map(|mut rec| {
            let mut point = rec.point;
            let mut normal = rec.normal;
            point.0 = self.cos_theta * rec.point.0 + self.sin_theta * rec.point.2;
            point.2 = - self.sin_theta * rec.point.0 + self.cos_theta * rec.point.2;
            normal.0 = self.cos_theta * rec.normal.0 + self.sin_theta * rec.normal.2;
            normal.2 = - self.sin_theta * rec.normal.0 + self.cos_theta * rec.normal.2;
            rec.point = point;
            rec.normal = normal;
            rec
        })
    }

    fn bounding_box(&self, t0: f32, t1: f32) -> Option<Aabb> {
        Some(self.bbox.clone())
    }
}

pub enum Figures {
    Sphere(Sphere),
    XYRect(XYRect),
    YZRect(YZRect),
    XZRect(XZRect),
    FlipNormals(FlipNormals),
    Cuboid(Cuboid),
    Translate(Translate),
    RotateY(RotateY),
    Figures(Vec<Figures>),
}

impl Figures {
    pub fn sphere(center: V3, radius: f32) -> Figures {
        Figures::Sphere(Sphere {
            center: center,
            radius: radius,
        })
    }

    pub fn xy_rect(x0: f32, x1: f32, y0: f32, y1: f32, k: f32) -> Figures {
        Figures::XYRect(XYRect {
            x0: x0,
            x1: x1,
            y0: y0,
            y1: y1,
            k: k,
        })
    }

    pub fn yz_rect(y0: f32, y1: f32, z0: f32, z1: f32, k: f32) -> Figures {
        Figures::YZRect(YZRect {
            y0: y0,
            y1: y1,
            z0: z0,
            z1: z1,
            k: k,
        })
    }

    pub fn xz_rect(x0: f32, x1: f32, z0: f32, z1: f32, k: f32) -> Figures {
        Figures::XZRect(XZRect {
            x0: x0,
            x1: x1,
            z0: z0,
            z1: z1,
            k: k,
        })
    }

    pub fn flip_normals(figure: Figures) -> Figures {
        Figures::FlipNormals(FlipNormals {
            figure: Box::new(figure),
        })
    }

    pub fn cuboid(p0: V3, p1: V3) -> Figures {
        Figures::Cuboid(Cuboid::new(p0, p1))
    }

    pub fn translate(offset: V3, figure: Figures) -> Figures {
        Figures::Translate(Translate {
            offset: offset,
            figure: Box::new(figure),
        })
    }

    pub fn rotate_y(angle: f32, figure: Figures) -> Figures {
        Figures::RotateY(RotateY::new(angle, figure))
    }

    pub fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        match self {
            Figures::Sphere(f) => f.hit(ray, tmin, tmax),
            Figures::XYRect(f) => f.hit(ray, tmin, tmax),
            Figures::YZRect(f) => f.hit(ray, tmin, tmax),
            Figures::XZRect(f) => f.hit(ray, tmin, tmax),
            Figures::FlipNormals(f) => f.hit(ray, tmin, tmax),
            Figures::Cuboid(f) => f.hit(ray, tmin, tmax),
            Figures::Translate(f) => f.hit(ray, tmin, tmax),
            Figures::RotateY(f) => f.hit(ray, tmin, tmax),
            Figures::Figures(fs) => {
                let mut closest_parameter = tmax;
                let mut record = None;

                for object in fs {
                    if let Some(rec) = object.hit(ray, tmin, closest_parameter) {
                        closest_parameter = rec.at;
                        record = Some(rec);
                    }
                }

                record
            },
        }
    }

    pub fn bounding_box(&self, tmin: f32, tmax: f32) -> Option<Aabb> {
        match self {
            Figures::Sphere(f) => f.bounding_box(tmin, tmax),
            Figures::XYRect(f) => f.bounding_box(tmin, tmax),
            Figures::YZRect(f) => f.bounding_box(tmin, tmax),
            Figures::XZRect(f) => f.bounding_box(tmin, tmax),
            Figures::FlipNormals(f) => f.bounding_box(tmin, tmax),
            Figures::Cuboid(f) => f.bounding_box(tmin, tmax),
            Figures::Translate(f) => f.bounding_box(tmin, tmax),
            Figures::RotateY(f) => f.bounding_box(tmin, tmax),
            Figures::Figures(fs) => unimplemented!(),
        }
    }
}

