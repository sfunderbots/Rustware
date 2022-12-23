use float_cmp;
use std::cmp::{Eq, Ord, Ordering, PartialOrd};
use std::f64::consts::PI;
use std::ops::{Add, Div, Neg, Sub};

#[derive(Clone, Copy, PartialOrd, Debug)]
pub struct Angle {
    radians: f64,
}

impl Angle {
    pub fn new() -> Angle {
        Angle::zero()
    }

    pub fn zero() -> Angle {
        Angle { radians: 0.0 }
    }

    pub fn half() -> Angle {
        Angle { radians: PI }
    }

    pub fn full() -> Angle {
        Angle { radians: 2.0 * PI }
    }

    pub fn from_degrees(degrees: f64) -> Angle {
        Angle {
            radians: degrees / 180.0 * PI,
        }
    }

    pub fn from_radians(radians: f64) -> Angle {
        Angle { radians: radians }
    }

    pub fn degrees(&self) -> f64 {
        // TODO: possible optimization. Cache this value in the struct?
        self.radians * 180.0 / PI
    }

    pub fn radians(&self) -> f64 {
        self.radians
    }

    pub fn sin(&self) -> f64 {
        self.radians.sin()
    }

    pub fn cos(&self) -> f64 {
        self.radians.cos()
    }

    pub fn clamp2pi(&self) -> Angle {
        Angle::from_radians(self.radians.rem_euclid(PI * 2.0))
    }
    pub fn clamp_pos_neg_pi(&self) -> Angle {
        let twopi = self.clamp2pi();
        if twopi > Angle::half() {
            twopi - Angle::full()
        } else {
            twopi
        }
    }
}

impl Ord for Angle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.radians.total_cmp(&other.radians)
    }
}

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        float_cmp::approx_eq!(
            f64,
            self.radians,
            other.radians,
            float_cmp::F64Margin {
                epsilon: 5.0 * f64::EPSILON,
                ulps: 5
            }
        )
    }
}
impl Eq for Angle {}

impl Neg for Angle {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Angle::from_radians(-self.radians)
    }
}

impl Add for Angle {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.radians + rhs.radians)
    }
}

impl Add for &Angle {
    type Output = Angle;
    fn add(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.radians + rhs.radians)
    }
}

impl Sub for Angle {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.radians - rhs.radians)
    }
}

impl Sub for &Angle {
    type Output = Angle;
    fn sub(self, rhs: Self) -> Self::Output {
        Angle::from_radians(self.radians - rhs.radians)
    }
}

impl Div<f64> for Angle {
    type Output = Angle;
    fn div(self, rhs: f64) -> Self::Output {
        Angle::from_radians(self.radians / rhs)
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
        let result = Angle::from_degrees(-199.0).clamp_pos_neg_pi();
        assert_eq!(result, Angle::from_degrees(161.0));
    }

    #[test]
    fn clamp_small_negative() {
        let result = Angle::from_degrees(-5.0).clamp_pos_neg_pi();
        assert_eq!(result, Angle::from_degrees(-5.0));
    }

    #[test]
    fn clamp_zero() {
        let result = Angle::from_degrees(0.0).clamp_pos_neg_pi();
        assert_eq!(result, Angle::from_degrees(0.0));
    }

    #[test]
    fn clamp_small_positive() {
        let result = Angle::from_degrees(6.0).clamp_pos_neg_pi();
        assert_eq!(result, Angle::from_degrees(6.0));
    }

    #[test]
    fn clamp_large_positive() {
        let result = Angle::from_degrees(370.0).clamp_pos_neg_pi();
        assert_eq!(result, Angle::from_degrees(10.0));
    }
}
