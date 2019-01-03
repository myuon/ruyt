use crate::vector::*;

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

struct Perlin {
    ranfloat: Vec<f32>,
    perm_x: Vec<u8>,
    perm_y: Vec<u8>,
    perm_z: Vec<u8>,
}

impl Perlin {
    fn new() -> Perlin {
        Perlin {
            ranfloat: Perlin::perlin_generate(),
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    fn perlin_generate() -> Vec<f32> {
        (0..256).map(|_| rand::random()).collect()
    }

    fn permute(vec: &mut Vec<u8>, n: usize) {
        for i in (1..n).rev() {
            let target = (rand::random::<f32>() * (i + 1) as f32).floor() as usize;
            let (x,y) = (vec[target],vec[i]);
            vec[target] = y;
            vec[i] = x;
        }
    }

    fn perlin_generate_perm() -> Vec<u8> {
        let mut vec = (0..=255).collect();
        Perlin::permute(&mut vec, 256);
        vec
    }

    fn trilinear_interp(vec: Vec<f32>, u: f32, v: f32, w: f32) -> f32 {
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    accum +=
                        (i as f32 * u + (1.0 - i as f32) * (1.0 - u)) *
                        (j as f32 * v + (1.0 - j as f32) * (1.0 - v)) *
                        (k as f32 * w + (1.0 - k as f32) * (1.0 - w)) * vec[i * 4 + j * 2 + k];
                }
            }
        }

        accum
    }

    fn noise(&self, point: &V3) -> f32 {
        let mut u = point.x() - point.x().floor();
        let mut v = point.y() - point.y().floor();
        let mut w = point.z() - point.z().floor();
        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let i = point.x().floor() as u8;
        let j = point.y().floor() as u8;
        let k = point.z().floor() as u8;

        let mut vec = vec![];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    vec.push(self.ranfloat[(self.perm_x[(i + di) as usize] ^ self.perm_y[(j + dj) as usize] ^ self.perm_z[(k + dk) as usize]) as usize]);
                }
            }
        }
        Perlin::trilinear_interp(vec, u, v, w)
    }
}

struct NoiseTexture {
    noise: Perlin,
    scaler: f32,
}

impl NoiseTexture {
    fn new(scaler: f32) -> NoiseTexture {
        NoiseTexture {
            noise: Perlin::new(),
            scaler: scaler,
        }
    }
}

impl Rendering for NoiseTexture {
    fn value(&self, _u: f32, _v: f32, point: &V3) -> V3 {
        V3(1.0, 1.0, 1.0).scale(self.noise.noise(&point.scale(self.scaler)))
    }
}

pub enum Textures {
    Solid(SolidTexture),
    Checker(CheckerTexture),
    Noise(NoiseTexture),
}

impl Textures {
    pub fn solid(color: V3) -> Textures {
        Textures::Solid(SolidTexture::new(color))
    }

    pub fn checker(even: Textures, odd: Textures) -> Textures {
        Textures::Checker(CheckerTexture::new(odd, even))
    }

    pub fn noise(scaler: f32) -> Textures {
        Textures::Noise(NoiseTexture::new(scaler))
    }
}

impl Rendering for Textures {
    fn value(&self, u: f32, v: f32, point: &V3) -> V3 {
        match self {
            Textures::Solid(t) => t.value(u, v, point),
            Textures::Checker(t) => t.value(u, v, point),
            Textures::Noise(t) => t.value(u, v, point),
        }
    }
}

