use crate::vector::*;
use crate::textures::*;

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
    pub albedo: V3,
    pub scattered: Ray,
    pub is_scattered: bool,
    pub pdf: f32,
}

pub struct Onb {
    axis: (V3, V3, V3),
}

impl Onb {
    pub fn random_cosine_direction() -> V3 {
        let r1 = rand::random::<f32>();
        let r2 = rand::random::<f32>();
        let z = (1.0 - r2).sqrt();
        let phi = 2.0 * ::std::f32::consts::PI * r1;
        let x = phi.cos() * 2.0 * r2.sqrt();
        let y = phi.sin() * 2.0 * r2.sqrt();
        V3(x,y,z)
    }

    pub fn new_from_w(n: &V3) -> Onb {
        let w = n.normalize();
        let a = if w.x().abs() > 0.9 {
            V3(0.0, 1.0, 0.0)
        } else {
            V3(1.0, 0.0, 0.0)
        };
        let v = w.cross(a).normalize();
        let u = w.cross(v);

        Onb {
            axis: (u,v,w),
        }
    }

    pub fn local(&self, vec: &V3) -> V3 {
        self.axis.0.scale(vec.x()) + self.axis.1.scale(vec.y()) + self.axis.2.scale(vec.z())
    }

    pub fn u(&self) -> V3 {
        self.axis.0
    }

    pub fn v(&self) -> V3 {
        self.axis.1
    }

    pub fn w(&self) -> V3 {
        self.axis.2
    }
}

pub struct Lambertian {
    albedo: Textures,
}

impl Material for Lambertian {
    fn scatter(&self, _ray_in: &Ray, rec: &HitRecord) -> ScatterRecord {
        let uvw = Onb::new_from_w(&rec.normal);
        let direction = uvw.local(&Onb::random_cosine_direction());
        let scattered = Ray {
            origin: rec.point,
            direction: direction.normalize(),
        };
        let pdf = uvw.axis.2.dot(scattered.direction) / ::std::f32::consts::PI;

        ScatterRecord {
            albedo: self.albedo.value(rec.u, rec.v, &rec.point),
            scattered: scattered,
            is_scattered: true,
            pdf: pdf,
        }
    }

    fn scattering_pdf(&self, ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        let cosine = hit_record.normal.dot(scattered.direction.normalize());
        if cosine < 0.0 { 0.0 } else { cosine / ::std::f32::consts::PI }
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
            albedo: self.albedo,
            scattered: ray,
            is_scattered: is_scattered,
            pdf: 1.0,
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
                albedo: V3(1.0, 1.0, 1.0),
                scattered: Ray { origin: rec.point, direction: if rand::random::<f32>() < reflect_prob { reflected } else { refracted } },
                is_scattered: true,
                pdf: 1.0,
            }
        } else {
            ScatterRecord {
                albedo: V3(1.0, 1.0, 1.0),
                scattered: Ray { origin: rec.point, direction: reflected },
                is_scattered: true,
                pdf: 1.0,
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
            albedo: V3(0.0, 0.0, 0.0),
            scattered: Ray { origin: V3(0.0, 0.0, 0.0), direction: V3(0.0, 0.0, 0.0) },
            is_scattered: false,
            pdf: 1.0,
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

