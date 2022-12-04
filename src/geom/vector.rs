use super::Angle;

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn new() -> Vector {
        Vector { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn orientation(&self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }

    pub fn rotate(&self, angle: Angle) -> Vector{
        Vector{
            x: self.x * angle.cos() - self.y * angle.sin(),
            y: self.x * angle.sin() + self.y * angle.cos()
        }
    }

    pub fn norm(&self, dist: f32) -> Vector{
        Vector{
            x: self.x / self.length() * dist,
            y: self.y / self.length() * dist
        }
    }
}
