use crate::vector::*;
use crate::materials::*;

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

pub enum Figures {
    Sphere(Sphere),
    XYRect(XYRect),
    YZRect(YZRect),
    XZRect(XZRect),
    FlipNormals(FlipNormals),
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

    pub fn hit(&self, ray: &Ray, tmin: f32, tmax: f32) -> Option<HitRecord> {
        match self {
            Figures::Sphere(f) => f.hit(ray, tmin, tmax),
            Figures::XYRect(f) => f.hit(ray, tmin, tmax),
            Figures::YZRect(f) => f.hit(ray, tmin, tmax),
            Figures::XZRect(f) => f.hit(ray, tmin, tmax),
            Figures::FlipNormals(f) => f.hit(ray, tmin, tmax),
        }
    }

    pub fn bounding_box(&self, tmin: f32, tmax: f32) -> Option<Aabb> {
        match self {
            Figures::Sphere(f) => f.bounding_box(tmin, tmax),
            Figures::XYRect(f) => f.bounding_box(tmin, tmax),
            Figures::YZRect(f) => f.bounding_box(tmin, tmax),
            Figures::XZRect(f) => f.bounding_box(tmin, tmax),
            Figures::FlipNormals(f) => f.bounding_box(tmin, tmax),
        }
    }
}

