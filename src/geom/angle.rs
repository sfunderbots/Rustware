use std::f32::consts::{PI};

pub struct Angle {
    radians: f32
}

impl Angle {
    pub fn new() -> Angle {
        Angle::new()
    }

    pub fn zero() -> Angle {
        Angle{radians: 0.0}
    }

    pub fn half() -> Angle {
        Angle{radians: 0.0}
    }

    pub fn from_degrees(degrees: f32) -> Angle {
        Angle{radians: degrees / 180.0 * PI}
    }

    pub fn from_radians(radians: f32) -> Angle {
        Angle{radians: radians}
    }

    pub fn degrees(&self) -> f32 {
        // TODO: possible optimization. Cache this value in the struct?
        self.radians * 180.0 / PI
    }

    pub fn radians(&self) -> f32 {
        self.radians
    }
}