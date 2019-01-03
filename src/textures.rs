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
    ranvec: Vec<V3>,
    perm_x: Vec<u8>,
    perm_y: Vec<u8>,
    perm_z: Vec<u8>,
}

impl Perlin {
    fn new() -> Perlin {
        Perlin {
            ranvec: Perlin::perlin_generate(),
            perm_x: Perlin::perlin_generate_perm(),
            perm_y: Perlin::perlin_generate_perm(),
            perm_z: Perlin::perlin_generate_perm(),
        }
    }

    fn perlin_generate() -> Vec<V3> {
        (0..256).map(|_|
            V3(
                -1.0 + 2.0 * rand::random::<f32>(),
                -1.0 + 2.0 * rand::random::<f32>(),
                -1.0 + 2.0 * rand::random::<f32>(),
            ).normalize()
        ).collect()
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

    fn perlin_interp(vec: Vec<V3>, u: f32, v: f32, w: f32) -> f32 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_vec = V3(u - i as f32, v - j as f32, w - k as f32);
                    accum +=
                        (i as f32 * uu + (1.0 - i as f32) * (1.0 - uu)) *
                        (j as f32 * vv + (1.0 - j as f32) * (1.0 - vv)) *
                        (k as f32 * ww + (1.0 - k as f32) * (1.0 - ww)) *
                        vec[4 * i + 2 * j + k].dot(weight_vec);
                }
            }
        }

        accum
    }

    fn turbulence(&self, point: &V3, depth: i32) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = *point;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p = temp_p.scale(2.0);
        }
        accum.abs()
    }

    fn noise(&self, point: &V3) -> f32 {
        let u = point.x() - point.x().floor();
        let v = point.y() - point.y().floor();
        let w = point.z() - point.z().floor();
        let i = point.x().floor() as u8;
        let j = point.y().floor() as u8;
        let k = point.z().floor() as u8;

        let mut vec = vec![];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    vec.push(self.ranvec[(self.perm_x[(i + di) as usize] ^ self.perm_y[(j + dj) as usize] ^ self.perm_z[(k + dk) as usize]) as usize]);
                }
            }
        }
        Perlin::perlin_interp(vec, u, v, w)
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
        V3(1.0, 1.0, 1.0).scale(0.5 * (1.0 + (self.scaler * point.z() + 10.0 * self.noise.turbulence(point, 7)).sin()))
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

