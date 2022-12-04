use crate::geom::{Circle, Point, Angle};

fn angle_sweep_circles(circles: Vec<Circle>, origin: Point, start: Point, end: Point) -> Vec<(Angle, Angle)> {
    /*
    How this algorithm works:
    Sweep over all obstacles in counterclockwise order, from start to end.
    1. Collect all the tangent angles that fall within the sweep range, keeping track
       of if they are the start of end of a circle.
    2. Iterate over the list of angles, using the "start/end" data to keep track
       of how many obstacles are blocking the sweep at any given point
    Future optimization idea: If this function needs to be sped up, reduce the input
    range to be <= Angle.half(), which should simplify a lot of angle checks and remove
    the need for clamp2pi()
    */
    let angle_start = (start - origin).orientation().clamp_pos_neg_pi();
    let angle_end = (end - origin).orientation().clamp_pos_neg_pi();

    let mut all_angles: Vec<(Angle, i32)> = Vec::new();
    let mut current_num_blockers: i32 = 0;
    let shifted_angle_end = (angle_end-angle_start).clamp2pi();
    for circle in &circles {
        let (tangent_point_1, tangent_point_2) = match circle.tangent_points(origin) {
            Option::Some((t1, t2)) => (t1, t2),
            Option::None => return Vec::new()
        };
        let tangent_angle_1 = ((tangent_point_1 - origin).orientation() - angle_start).clamp_pos_neg_pi();
        let tangent_angle_2 = ((tangent_point_2 - origin).orientation() - angle_start).clamp_pos_neg_pi();
        // Only add the angles that are inside the sweep range
        if tangent_angle_1 <= Angle::zero() && tangent_angle_2 > Angle::zero() {
            // In this case, a circle is overlapping the start angle, meaning we start the sweep with +1 obstacles
            current_num_blockers+=1;
            // It's possible for the first tangent to still fall within the range,
            // so check if that's the case and add it here. Clamp to [0, 2pi] since in this case
            // the angle should be treated as if it's counterclockwise of the start
            if tangent_angle_1.clamp2pi() <= shifted_angle_end {
                all_angles.push((tangent_angle_1.clamp2pi(), 1));
            }
            if tangent_angle_2.clamp2pi() < shifted_angle_end {
                all_angles.push((tangent_angle_2.clamp2pi(), -1))
            }
        }else if Angle::zero() <= tangent_angle_1.clamp2pi() && tangent_angle_1.clamp2pi() < shifted_angle_end {
            // In this case the circle starts within the range
            all_angles.push((tangent_angle_1.clamp2pi(), 1));
            if tangent_angle_2.clamp2pi() < shifted_angle_end {
                all_angles.push((tangent_angle_2, -1));
            }
        }
    }

    all_angles.push((Angle::zero(), 0));
    // all_angles.sort_by(|x, y| x.0.partial_cmp(y.0));
    all_angles.sort_unstable_by_key(|x| x.0);

    assert!(all_angles[0].0 == Angle::zero(), "First angle in sweep should always be zero");

    let mut current_open_angle = None;
    let mut open_angles: Vec<(Angle, Angle)> = Vec::new();
    for (angle, num_blockers_change) in all_angles {
        current_num_blockers += num_blockers_change;
        if current_num_blockers <= 0 {
            current_num_blockers = 0;
            current_open_angle = Some(angle);
        }else if current_num_blockers > 0 && current_open_angle.is_some(){
            open_angles.push((current_open_angle.unwrap(), angle));
            current_open_angle = None;
        }
    }
    if current_num_blockers == 0 && current_open_angle.is_some() {
        open_angles.push((current_open_angle.unwrap(), shifted_angle_end));
    }

    let unshifted_open_angles = open_angles.into_iter()
        .filter(|(a, b)| (b-a).degrees().abs() > 1.0e-3)
        .map(|(a, b)| ((angle_start + a).clamp_pos_neg_pi(), (angle_start+b).clamp_pos_neg_pi()))
        .collect();
    unshifted_open_angles
}