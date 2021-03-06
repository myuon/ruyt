use crate::vector::*;
use crate::textures::*;
use crate::pdf::*;

#[derive(Clone)]
pub struct HitRecord {
    pub at: f32,
    pub point: V3,
    pub normal: V3,
    pub u: f32,
    pub v: f32,
}

trait Material {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterRecord;

    fn scattering_pdf(&self, ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        0.0
    }

    fn emitted(&self, u: f32, v: f32, point: &V3) -> V3 {
        V3(0.0, 0.0, 0.0)
    }
}

#[derive(Clone)]
pub struct ScatterRecord {
    pub attenuation: V3,
    pub specular_ray: Option<Ray>,
    pub pdf: Option<Pdfs>,
    pub is_scattered: bool,
}

pub struct Lambertian {
    albedo: Textures,
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        ScatterRecord {
            attenuation: self.albedo.value(rec.u, rec.v, &rec.point),
            specular_ray: None,
            pdf: Some(Pdfs::CosinePdf(CosinePdf::new(&rec.normal))),
            is_scattered: true,
        }
    }

    fn scattering_pdf(&self, ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        let cosine = hit_record.normal.dot(scattered.direction);
        if cosine < 0.0 { 0.0 } else { cosine / std::f32::consts::PI }
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
        let reflected = Metal::reflect(&ray_in.direction.as_V3(), &rec.normal);
        let specular_ray = Ray {
            origin: rec.point,
            direction: V3U::new(reflected + V3::new_in_unit_sphere().scale(self.fuzz)),
        };

        ScatterRecord {
            attenuation: self.albedo,
            specular_ray: Some(specular_ray),
            pdf: None,
            is_scattered: true,
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
        let reflected = Dielectric::reflect(&ray_in.direction.as_V3(), &rec.normal);
        let (outward_normal, ni_over_nt, cosine) =
            if ray_in.direction.dot(rec.normal) > 0.0 {
                let cosine = self.ref_idx * ray_in.direction.dot(rec.normal);
                (-rec.normal, self.ref_idx, cosine)
            } else {
                let cosine = - ray_in.direction.dot(rec.normal);
                (rec.normal, 1.0 / self.ref_idx, cosine)
            };

        if let Some(refracted) = Dielectric::refract(&ray_in.direction.as_V3(), outward_normal, ni_over_nt) {
            let reflect_prob = self.schlick(cosine);

            ScatterRecord {
                attenuation: V3(1.0, 1.0, 1.0),
                specular_ray: Some(Ray { origin: rec.point, direction: if rand::random::<f32>() < reflect_prob { V3U::new(reflected) } else { V3U::new(refracted) } }),
                is_scattered: true,
                pdf: None,
            }
        } else {
            ScatterRecord {
                attenuation: V3(1.0, 1.0, 1.0),
                specular_ray: Some(Ray { origin: rec.point, direction: V3U::new(reflected) }),
                is_scattered: true,
                pdf: None,
            }
        }
    }
}

struct DiffuseLight {
    emit: Textures,
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> ScatterRecord {
        ScatterRecord {
            attenuation: V3(0.0, 0.0, 0.0),
            specular_ray: Some(Ray { origin: V3(0.0, 0.0, 0.0), direction: V3U::new(V3(1.0, 0.0, 0.0)) }),
            is_scattered: false,
            pdf: None,
        }
    }

    fn emitted(&self, u: f32, v: f32, point: &V3) -> V3 {
        self.emit.value(u, v, point)
    }
}

pub enum Materials {
    Lambertian(Lambertian),
    Metal(Metal),
    Dielectric(Dielectric),
    DiffuseLight(DiffuseLight),
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

    pub fn diffuse_light(emit: Textures) -> Materials {
        Materials::DiffuseLight(DiffuseLight {
            emit: emit
        })
    }

    pub fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> ScatterRecord {
        match self {
            Materials::Lambertian(m) => m.scatter(ray_in, hit_record),
            Materials::Metal(m) => m.scatter(ray_in, hit_record),
            Materials::Dielectric(m) => m.scatter(ray_in, hit_record),
            Materials::DiffuseLight(m) => m.scatter(ray_in, hit_record),
        }
    }

    pub fn scattering_pdf(&self, ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        match self {
            Materials::Lambertian(m) => m.scattering_pdf(ray_in, hit_record, scattered),
            Materials::Metal(m) => m.scattering_pdf(ray_in, hit_record, scattered),
            Materials::Dielectric(m) => m.scattering_pdf(ray_in, hit_record, scattered),
            Materials::DiffuseLight(m) => m.scattering_pdf(ray_in, hit_record, scattered),
        }
    }

    pub fn emitted(&self, u: f32, v: f32, point: &V3) -> V3 {
        match self {
            Materials::Lambertian(m) => m.emitted(u,v,point),
            Materials::Metal(m) => m.emitted(u,v,point),
            Materials::Dielectric(m) => m.emitted(u,v,point),
            Materials::DiffuseLight(m) => m.emitted(u,v,point),
        }
    }
}

