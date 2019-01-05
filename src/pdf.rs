use crate::vector::*;
use crate::figures::*;

#[derive(Clone)]
pub struct OnbPdf {
    uvw: Onb,
}

impl OnbPdf {
    pub fn new(vec: &V3) -> OnbPdf {
        OnbPdf {
            uvw: Onb::new_from_w(vec),
        }
    }

    pub fn value(&self, direction: &V3) -> f32 {
        let cosine = direction.normalize().dot(self.uvw.w());
        if cosine > 0.0 {
            cosine / std::f32::consts::PI
        } else {
            0.0
        }
    }

    pub fn generate(&self) -> V3 {
        self.uvw.local(&Onb::random_cosine_direction())
    }
}

#[derive(Clone)]
pub struct HitPdf {
    figure: Figures,
    origin: V3,
}

impl HitPdf {
    pub fn new(figure: Figures, origin: V3) -> HitPdf {
        HitPdf {
            figure: figure,
            origin: origin,
        }
    }

    pub fn value(&self, direction: V3) -> f32 {
        self.figure.pdf_value(self.origin, direction)
    }

    pub fn generate(&self) -> V3 {
        self.figure.random(self.origin)
    }
}

#[derive(Clone)]
pub struct MixPdf {
    pdf: (Box<Pdfs>, Box<Pdfs>),
}

impl MixPdf {
    pub fn new(p0: Pdfs, p1: Pdfs) -> MixPdf {
        MixPdf {
            pdf: (Box::new(p0), Box::new(p1))
        }
    }

    pub fn value(&self, direction: V3) -> f32 {
        0.5 * self.pdf.0.value(direction) + 0.5 * self.pdf.1.value(direction)
    }

    pub fn generate(&self) -> V3 {
        if rand::random::<f32>() < 0.5 {
            self.pdf.0.generate()
        } else {
            self.pdf.1.generate()
        }
    }
}

#[derive(Clone)]
pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: &V3) -> CosinePdf {
        CosinePdf {
            uvw: Onb::new_from_w(w)
        }
    }

    pub fn value(&self, direction: V3) -> f32 {
        let cosine = direction.normalize().dot(self.uvw.w());
        if cosine > 0.0 {
            cosine / std::f32::consts::PI
        } else {
            0.0
        }
    }

    pub fn generate(&self) -> V3 {
        self.uvw.local(&Onb::random_cosine_direction())
    }
}

#[derive(Clone)]
pub enum Pdfs {
    MixPdf(MixPdf),
    CosinePdf(CosinePdf),
    HitPdf(HitPdf),
}

impl Pdfs {
    pub fn value(&self, direction: V3) -> f32 {
        match self {
            Pdfs::MixPdf(p) => p.value(direction),
            Pdfs::CosinePdf(p) => p.value(direction),
            Pdfs::HitPdf(p) => p.value(direction),
        }
    }

    pub fn generate(&self) -> V3 {
        match self {
            Pdfs::MixPdf(p) => p.generate(),
            Pdfs::CosinePdf(p) => p.generate(),
            Pdfs::HitPdf(p) => p.generate(),
        }
    }
}

