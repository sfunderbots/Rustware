use std::f32::consts::PI;
use std::ops::{Sub, Neg};
use std::cmp::{Eq, PartialOrd};
use float_cmp;

#[derive(Debug, PartialOrd)]
pub struct Angle {
    radians: f32
}

impl Angle {
    pub fn new() -> Angle {
        Angle::zero()
    }

    pub fn zero() -> Angle {
        Angle{radians: 0.0}
    }

    pub fn half() -> Angle {
        Angle{radians: PI}
    }

    pub fn full() -> Angle {
        Angle{radians: 2.0*PI}
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

    pub fn clamp2pi(&self) -> Angle {
        Angle::from_radians(self.radians.rem_euclid(PI*2.0))
    }
    pub fn clamp(&self) -> Angle {
        let twopi = self.clamp2pi();
        if twopi > Angle::half() {
            twopi - Angle::full()
        }else {
            twopi
        }
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        float_cmp::approx_eq!(f32, self.radians, other.radians, float_cmp::F32Margin{epsilon: 5.0*f32::EPSILON, ulps: 5})
    }
}
impl Eq for Angle {}

impl Neg for Angle {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Angle::from_radians(-self.radians)
    }
}

impl Sub for Angle {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.radians - rhs.radians)
    }
}


#[cfg(test)]
mod tests {
    use super::Angle;

    #[test]
    fn clamp2pi_small_positive() {
        let result = Angle::half().clamp2pi();
        assert_eq!(result, Angle::half());
    }

    #[test]
    fn clamp2pi_large_positive() {
        let result = Angle::from_degrees(365.0).clamp2pi();
        assert_eq!(result, Angle::from_degrees(5.0));
    }

    #[test]
    fn clamp2pi_very_large_positive() {
        let result = Angle::from_degrees(731.0).clamp2pi();
        assert_eq!(result, Angle::from_degrees(11.0));
    }

    #[test]
    fn clamp2pi_zero() {
        let result = Angle::zero().clamp2pi();
        assert_eq!(result, Angle::zero());
    }

    #[test]
    fn clamp2pi_small_negative() {
        let result = Angle::from_degrees(-6.0).clamp2pi();
        assert_eq!(result, Angle::from_degrees(354.0));
    }

    #[test]
    fn clamp2pi_large_negative() {
        let result = Angle::from_degrees(-375.0).clamp2pi();
        assert_eq!(result, Angle::from_degrees(345.0));
    }

    #[test]
    fn clamp_large_negative() {
        let result = Angle::from_degrees(-199.0).clamp();
        assert_eq!(result, Angle::from_degrees(161.0));
    }

    #[test]
    fn clamp_small_negative() {
        let result = Angle::from_degrees(-5.0).clamp();
        assert_eq!(result, Angle::from_degrees(-5.0));
    }

    #[test]
    fn clamp_zero() {
        let result = Angle::from_degrees(0.0).clamp();
        assert_eq!(result, Angle::from_degrees(0.0));
    }

    #[test]
    fn clamp_small_positive() {
        let result = Angle::from_degrees(6.0).clamp();
        assert_eq!(result, Angle::from_degrees(6.0));
    }

    #[test]
    fn clamp_large_positive() {
        let result = Angle::from_degrees(370.0).clamp();
        assert_eq!(result, Angle::from_degrees(10.0));
    }
}