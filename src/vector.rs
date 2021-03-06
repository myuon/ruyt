use std::ops::*;
use std::iter::Sum;

pub trait Dim3 {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;
}

pub trait Dim3Dot<Other: Dim3>: Dim3 {
    fn dot(&self, other: Other) -> f32 {
        self.x() * other.x() +
        self.y() * other.y() +
        self.z() * other.z()
    }
}

#[derive(Clone, Copy)]
pub struct V3(pub f32, pub f32, pub f32);

impl V3 {
    pub fn new_in_unit_sphere() -> V3 {
        loop {
            let p = V3(rand::random::<f32>(), rand::random::<f32>(), rand::random::<f32>()).scale(2.0) - V3(1.0, 1.0, 1.0);
            if p.square_norm() < 1.0 {
                return p;
            }
        }
    }

    pub fn new_in_unit_disk() -> V3 {
        loop {
            let p = V3(rand::random::<f32>(), rand::random::<f32>(), 0.0).scale(2.0) - V3(1.0, 1.0, 0.0);
            if p.square_norm() < 1.0 {
                return p;
            }
        }
    }

    pub fn cross(self, other: V3) -> V3 {
        V3(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }

    pub fn square_norm(self) -> f32 {
        self.dot(self)
    }

    pub fn norm(self) -> f32 {
        self.square_norm().sqrt()
    }

    pub fn scale(self, coeff: f32) -> V3 {
        V3(self.0 * coeff, self.1 * coeff, self.2 * coeff)
    }

    pub fn normalize(self) -> V3 {
        self.scale(1.0 / self.norm())
    }

    pub fn map(self, f: &Fn(f32) -> f32) -> V3 {
        V3(f(self.0), f(self.1), f(self.2))
    }
}

impl Dim3 for V3 {
    fn x(&self) -> f32 {
        self.0
    }

    fn y(&self) -> f32 {
        self.1
    }

    fn z(&self) -> f32 {
        self.2
    }
}

impl Dim3Dot<V3> for V3 {}

impl Add<V3> for V3 {
    type Output = V3;

    fn add(self, other: V3) -> V3 {
        V3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Sub<V3> for V3 {
    type Output = V3;

    fn sub(self, other: V3) -> V3 {
        V3(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl Neg for V3 {
    type Output = V3;

    fn neg(self) -> V3 {
        V3(0.0, 0.0, 0.0) - self
    }
}

impl Mul<V3> for V3 {
    type Output = V3;

    fn mul(self, other: V3) -> V3 {
        V3(self.0 * other.0, self.1 * other.1, self.2 * other.2)
    }
}

impl Sum<V3> for V3 {
    fn sum<I>(iter: I) -> V3 where I: Iterator<Item = V3> {
        let mut r = V3(0.0, 0.0, 0.0);
        for i in iter {
            r = r + i;
        }

        r
    }
}

#[derive(Clone, Copy)]
pub struct V3U(V3);

impl V3U {
    pub fn new(v: V3) -> V3U {
        V3U(v.normalize())
    }

    pub fn as_V3(self) -> V3 {
        self.0
    }

    pub fn scale(self, coeff: f32) -> V3 {
        V3(self.x() * coeff, self.y() * coeff, self.z() * coeff)
    }
}

impl Dim3 for V3U {
    fn x(&self) -> f32 {
        self.0.x()
    }

    fn y(&self) -> f32 {
        self.0.y()
    }

    fn z(&self) -> f32 {
        self.0.z()
    }
}

impl Dim3Dot<V3U> for V3U {}
impl Dim3Dot<V3> for V3U {}
impl Dim3Dot<V3U> for V3 {}

#[derive(Clone)]
pub struct Ray {
    pub origin: V3,
    pub direction: V3U,
}

impl Ray {
    pub fn extend_at(&self, scaler: f32) -> V3 {
        self.origin + self.direction.scale(scaler)
    }
}


