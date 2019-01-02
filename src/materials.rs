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

pub struct Lambertian {
    albedo: V3,
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        let target = rec.point + rec.normal + V3::new_in_unit_sphere();
        ScatterRecord {
            attenuation: self.albedo,
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

pub enum Materials {
    Lambertian(Lambertian),
    Metal(Metal),
}

impl Materials {
    pub fn lambertian(albedo: V3) -> Materials {
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

    pub fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterRecord {
        match self {
            Materials::Lambertian(m) => m.scatter(ray_in, hit_record),
            Materials::Metal(m) => m.scatter(ray_in, hit_record),
        }
    }
}

