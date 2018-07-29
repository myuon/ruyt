use std::ops::Add;

#[derive(Clone, Copy)]
pub struct V3(pub f32, pub f32, pub f32);

impl V3 {
    pub fn new(x: f32, y: f32, z: f32) -> V3 {
        V3(x,y,z)
    }

    pub fn dot(self, other: V3) -> f32 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
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

    pub fn x(&self) -> f32 {
        self.0
    }

    pub fn y(&self) -> f32 {
        self.1
    }

    pub fn z(&self) -> f32 {
        self.2
    }
}

impl Add<V3> for V3 {
    type Output = V3;

    fn add(self, other: V3) -> V3 {
        V3(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

#[derive(Clone, Copy)]
pub struct V3U(V3);

impl V3U {
    pub fn new(v: V3) -> V3U {
        V3U(v.scale(1.0 / v.norm()))
    }

    pub fn x(&self) -> f32 {
        self.0.x()
    }

    pub fn y(&self) -> f32 {
        self.0.y()
    }

    pub fn z(&self) -> f32 {
        self.0.z()
    }
}

