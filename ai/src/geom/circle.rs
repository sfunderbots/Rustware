use super::Point;
use crate::geom::Angle;

#[derive(Clone, Copy, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn tangent_points(&self, point: Point) -> Option<(Point, Point)> {
        if self.radius < 1e-6 {
            return Option::Some((self.center.clone(), self.center.clone()));
        }

        let point_to_center = self.center - point;
        if point_to_center.length() < self.radius {
            return Option::None;
        }

        let angle_off_point = Angle::from_radians((self.radius / point_to_center.length()).asin());
        let dist_to_tangent_points = point_to_center.length().hypot(self.radius);
        let p1 = point
            + point_to_center
                .rotate(&-angle_off_point)
                .norm(dist_to_tangent_points);
        let p2 = point
            + point_to_center
                .rotate(&angle_off_point)
                .norm(dist_to_tangent_points);
        Option::Some((p1, p2))
    }
}
