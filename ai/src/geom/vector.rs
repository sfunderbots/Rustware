use super::Angle;
use std::ops::Div;

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn new() -> Vector {
        Vector { x: 0.0, y: 0.0 }
    }

    pub fn from_angle(angle: Angle, length: f64) -> Vector {
        Vector {
            x: length * angle.cos(),
            y: length * angle.sin(),
        }
    }

    pub fn length(&self) -> f64 {
        self.x.hypot(self.y)
    }

    pub fn orientation(&self) -> Angle {
        Angle::from_radians(self.y.atan2(self.x))
    }

    pub fn rotate(&self, angle: &Angle) -> Vector {
        Vector {
            x: self.x * angle.cos() - self.y * angle.sin(),
            y: self.x * angle.sin() + self.y * angle.cos(),
        }
    }

    pub fn norm(&self, dist: f64) -> Vector {
        Vector {
            x: self.x / self.length() * dist,
            y: self.y / self.length() * dist,
        }
    }
}

impl Div<f64> for Vector {
    type Output = Vector;
    fn div(self, rhs: f64) -> Self::Output {
        Vector {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
