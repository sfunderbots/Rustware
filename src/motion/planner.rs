use crate::motion::{KinematicState, Trajectory};
use crate::geom::{Point, Angle};

pub fn stopping_trajectory(initial_state: &KinematicState) -> Trajectory {
    let mut traj = Trajectory::new();
    traj.points = vec![initial_state.position, initial_state.position];
    traj
}

pub fn straight_line(initial_state: &KinematicState, target_position: &Point, target_orientation: &Angle) -> Trajectory {
    let mut traj = Trajectory::new();
    traj.points = vec![initial_state.position, *target_position];
    traj.final_orientation = *target_orientation;
    traj
}