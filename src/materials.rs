use crate::vector::*;

#[derive(Clone)]
pub struct HitRecord {
    pub at: f32,
    pub point: V3,
    pub normal: V3,
}

trait Material {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterRecord;
}

#[derive(Clone)]
pub struct ScatterRecord {
    pub attenuation: V3,
    pub scattered: Ray,
    pub is_scattered: bool,
}

pub trait Rendering {
    fn value(&self, u: f32, v: f32, point: &V3) -> V3;
}

struct SolidTexture {
    color: V3,
}

impl SolidTexture {
    fn new(color: V3) -> SolidTexture {
        SolidTexture {
            color: color
        }
    }
}

impl Rendering for SolidTexture {
    fn value(&self, u: f32, v: f32, point: &V3) -> V3 {
        self.color
    }
}

struct CheckerTexture {
    odd: Box<Textures>,
    even: Box<Textures>,
}

impl CheckerTexture {
    fn new(odd: Textures, even: Textures) -> CheckerTexture {
        CheckerTexture {
            odd: Box::new(odd),
            even: Box::new(even),
        }
    }
}

impl Rendering for CheckerTexture {
    fn value(&self, u: f32, v: f32, point: &V3) -> V3 {
        let sines = (10.0 * point.x()).sin() * (10.0 * point.y()).sin() * (10.0 * point.z()).sin();
        if sines < 0.0 {
            self.odd.value(u, v, point)
        } else {
            self.even.value(u, v, point)
        }
    }
}

pub enum Textures {
    Solid(SolidTexture),
    Checker(CheckerTexture),
}

impl Textures {
    pub fn solid(color: V3) -> Textures {
        Textures::Solid(SolidTexture::new(color))
    }

    pub fn checker(even: Textures, odd: Textures) -> Textures {
        Textures::Checker(CheckerTexture::new(odd, even))
    }
}

impl Rendering for Textures {
    fn value(&self, u: f32, v: f32, point: &V3) -> V3 {
        match self {
            Textures::Solid(t) => t.value(u, v, point),
            Textures::Checker(t) => t.value(u, v, point),
        }
    }
}

pub struct Lambertian {
    albedo: Textures,
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        let target = rec.point + rec.normal + V3::new_in_unit_sphere();
        ScatterRecord {
            attenuation: self.albedo.value(0.0, 0.0, &rec.point),
            scattered: Ray {
                origin: rec.point,
                direction: target - rec.point,
            },
            is_scattered: true,
        }
    }
}

pub struct Metal {
    albedo: V3,
    fuzz: f32,
}

impl Metal {
    fn reflect(v: &V3, n: &V3) -> V3 {
        *v - n.scale(2.0 * v.dot(*n))
    }
}

impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        let reflected = Metal::reflect(&ray_in.direction.normalize(), &rec.normal);
        let ray = Ray {
            origin: rec.point,
            direction: reflected + V3::new_in_unit_sphere().scale(self.fuzz),
        };
        let is_scattered = ray.direction.dot(rec.normal) > 0.0;

        ScatterRecord {
            attenuation: self.albedo,
            scattered: ray,
            is_scattered: is_scattered,
        }
    }
}

pub struct Dielectric {
    ref_idx: f32,
}

impl Dielectric {
    fn reflect(v: &V3, n: &V3) -> V3 {
        *v - n.scale(2.0 * v.dot(*n))
    }

    fn refract(v: &V3, n: V3, ni_over_nt: f32) -> Option<V3> {
        let uv = v.normalize();
        let dt = uv.dot(n);
        let discriminant = 1.0 - ni_over_nt * ni_over_nt * (1.0 - dt * dt);

        if discriminant > 0.0 {
            Some((uv - n.scale(dt)).scale(ni_over_nt) - n.scale(discriminant.sqrt()))
        } else {
            None
        }
    }

    fn schlick(&self, cosine: f32) -> f32 {
        let r0 = (1.0 - self.ref_idx) / (1.0 + self.ref_idx);
        r0 * r0 + (1.0 - r0 * r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        let reflected = Dielectric::reflect(&ray_in.direction, &rec.normal);
        let (outward_normal, ni_over_nt, cosine) =
            if ray_in.direction.dot(rec.normal) > 0.0 {
                let cosine = self.ref_idx * ray_in.direction.dot(rec.normal) / ray_in.direction.norm();
                (-rec.normal, self.ref_idx, cosine)
            } else {
                let cosine = - ray_in.direction.dot(rec.normal) / ray_in.direction.norm();
                (rec.normal, 1.0 / self.ref_idx, cosine)
            };

        if let Some(refracted) = Dielectric::refract(&ray_in.direction, outward_normal, ni_over_nt) {
            let reflect_prob = self.schlick(cosine);

            ScatterRecord {
                attenuation: V3(1.0, 1.0, 1.0),
                scattered: Ray { origin: rec.point, direction: if rand::random::<f32>() < reflect_prob { reflected } else { refracted } },
                is_scattered: true,
            }
        } else {
            ScatterRecord {
                attenuation: V3(1.0, 1.0, 1.0),
                scattered: Ray { origin: rec.point, direction: reflected },
                is_scattered: true,
            }
        }
    }
}

pub enum Materials {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
}

impl Materials {
    pub fn lambertian(albedo: Textures) -> Materials {
        Materials::Lambertian(Lambertian {
            albedo: albedo,
        })
    }

    pub fn metal(albedo: V3, fuzz: f32) -> Materials {
        Materials::Metal(Metal {
            albedo: albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        })
    }

    pub fn dielectric(ref_idx: f32) -> Materials {
        Materials::Dielectric(Dielectric {
            ref_idx: ref_idx
        })
    }

    pub fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterRecord {
        match self {
            Materials::Lambertian(m) => m.scatter(ray_in, hit_record),
            Materials::Metal(m) => m.scatter(ray_in, hit_record),
            Materials::Dielectric(m) => m.scatter(ray_in, hit_record),
        }
    }
}

