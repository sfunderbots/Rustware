pub mod planner;
mod tracker;

use crate::geom::{Angle, Point, Vector};

pub fn bb_time_to_position_1d(xi: f32, vi: f32, xf: f32, vf: f32, a: f32, v_max: f32) -> f32 {
    assert!(vf == 0.0);
    assert!(a > 1.0e-3);
    assert!(v_max > 0.0);

    let mut total_time = 0.0;
    let mut current_x = xi;
    let mut remaining_dist = (xf - xi).abs();
    let vi_sign = if vi >= 0.0 { 1.0 } else { -1.0 };
    let mut current_v = vi;

    if (xf > xi && current_v < 0.0) || (xf < xi && current_v > 0.0) {
        // current_v not in direction of xf, ned to stop first
        let time_to_stop = current_v.abs() / a;
        total_time += time_to_stop;
        let dist_to_stop = current_v.abs() / 2.0 * time_to_stop;
        current_x += vi_sign * dist_to_stop;
        remaining_dist += dist_to_stop;
        current_v = 0.0;
    } else if current_v.abs() > v_max {
        // Starting off very fast in direction of xf, need to slow down to stay within v max
        let v_diff = current_v.abs() - v_max;
        let time_to_slow_to_v_max = v_diff / a;
        total_time += time_to_slow_to_v_max;
        let dist_to_slow_to_v_max = v_diff / 2.0 * time_to_slow_to_v_max;
        remaining_dist -= dist_to_slow_to_v_max;
        current_x += vi_sign * dist_to_slow_to_v_max;
        current_v = vi_sign * v_max;
    }

    let time_to_stop_from_vi = current_v.abs() / a;
    let dist_to_stop_from_vi = current_v.abs() / 2.0 * time_to_stop_from_vi;
    let time_to_stop_from_vmax = v_max / a;
    let dist_to_stop_from_vmax = v_max / 2.0 * time_to_stop_from_vmax;

    if dist_to_stop_from_vi > remaining_dist {
        // Overshoot, can't slow down in time
        current_x += vi_sign * dist_to_stop_from_vi;
        return total_time
            + time_to_stop_from_vi
            + bb_time_to_position_1d(current_x, 0.0, xf, vf, a, v_max);
    }

    let time_to_accel_to_v_max = (v_max - current_v.abs()) / a;
    let dist_to_accel_to_v_max = (v_max - current_v.abs()) / 2.0 * time_to_accel_to_v_max;

    if dist_to_accel_to_v_max + dist_to_stop_from_vmax > remaining_dist {
        // Not enough time to hit v_max
        let dist = remaining_dist - dist_to_stop_from_vi;
        let vf_at_mid_dist = (current_v.powi(2) + 2.0 * a * dist / 2.0).sqrt();
        let time_to_reach_vf_at_mid = (vf_at_mid_dist - current_v.abs()) / a;
        return total_time + 2.0 * time_to_reach_vf_at_mid + time_to_stop_from_vi;
    } else {
        // hit v_max
        let dist_at_v_max = remaining_dist + dist_to_accel_to_v_max - dist_to_stop_from_vmax;
        let time_at_v_max = dist_at_v_max / v_max;
        return total_time + time_to_accel_to_v_max + time_at_v_max + time_to_stop_from_vmax;
    }
}

pub fn bb_time_to_position(
    start: &Point,
    initial_velocity: &Vector,
    end: &Point,
    a: f32,
    max_speed: f32,
) -> f32 {
    let time_for_x = bb_time_to_position_1d(start.x, initial_velocity.x, end.x, 0.0, a, max_speed);
    let time_for_y = bb_time_to_position_1d(start.y, initial_velocity.y, end.y, 0.0, a, max_speed);
    time_for_x.max(time_for_y)
}

pub struct KinematicState {
    pub position: Point,
    pub orientation: Angle,
    pub velocity: Vector,
    pub angular_velocity: Angle,
}

pub struct Trajectory {
    points: Vec<Point>,
    final_orientation: Angle,
    dribble: bool,
    autokick_speed: Option<f32>,
    autochip_distance: Option<f32>,
}

impl Trajectory {
    fn new() -> Self {
        Trajectory {
            points: vec![],
            final_orientation: Angle::zero(),
            dribble: false,
            autokick_speed: None,
            autochip_distance: None,
        }
    }
}
