use crate::communication::Node;
use crate::geom::{Angle, Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::{bb_time_to_position, KinematicState};
use crate::perception::world::{Field, Robot};
use multiqueue2;
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

#[derive(Clone)]
struct Pass {
    start: Point,
    end: Point,
    speed: f32,
    time_offset: f32,
}

impl Pass {
    pub fn time_to_complete(&self) -> f32 {
        let pass_dist = (self.end.x - self.start.x).hypot(self.end.y - self.start.y);
        pass_dist / self.speed + self.time_offset
    }
}

fn static_score(p: &Point, field: &Field) -> f32 {
    let on_field_score = rect_sigmoid(field.touch_lines(), p, 0.5);
    let enemy_defense_score = 1.0 - rect_sigmoid(field.enemy_defense_area(), p, 0.5);
    let friendly_defense_score = 1.0 - rect_sigmoid(field.friendly_defense_area(), p, 0.5);
    let field_progress_score = sigmoid(p.x, 0.0, field.x_length) / 10.0 + 0.9;
    on_field_score * enemy_defense_score * friendly_defense_score * field_progress_score
}

fn friendly_intercept_score(p: &Pass, robots: &Vec<Robot>) -> f32 {
    if !robots.is_empty() {
        let mut times_to_pos: Vec<f32> = Vec::new();
        for r in robots {
            times_to_pos.push(bb_time_to_position(
                &r.state.position,
                &r.state.velocity,
                &p.end,
                3.0,
                3.0,
            ));
        }
        // TODO: consider time it takes robot to rotate to face pass
        // TODO: take into account robot radius when arriving at position
        // The robot has to get there before the ball is < 1 radius away
        let min_time_to_position = times_to_pos
            .iter()
            .fold(f32::INFINITY, |prev, curr| prev.min(*curr));
        // If positive, a friendly robot can get to the pass position before the ball will arrive there
        let time_to_position_diff = p.time_to_complete() - min_time_to_position;
        sigmoid(time_to_position_diff, 0.5, 1.0)
    } else {
        0.0
    }
}

fn enemy_min_tim_to_intercept(p: &Pass, r: &Robot) -> f32 {
    const REACTION_DELAY: f32 = 0.3;
    const NUM_STEPS: usize = 20;
    const ROBOT_RADIUS: f32 = 0.18;
    let x_incr = (p.end.x - p.start.x) / NUM_STEPS as f32;
    let y_incr = (p.end.y - p.start.y) / NUM_STEPS as f32;
    let mut min_diff = f32::INFINITY;
    for i in 0..NUM_STEPS {
        let pos = Point {
            x: p.start.x + i as f32 * x_incr + ROBOT_RADIUS,
            y: p.start.x + i as f32 * y_incr + ROBOT_RADIUS,
        };
        // TODO: Take into consideration the robot radius
        let ttp = bb_time_to_position(&r.state.position, &r.state.velocity, &pos, 3.0, 3.0);
        let diff = ttp - p.time_to_complete();
        min_diff = min_diff.min(diff);
    }
    min_diff + REACTION_DELAY
}

fn enemy_intercept_score(p: &Pass, robots: &Vec<Robot>) -> f32 {
    if !robots.is_empty() {
        let mut enemy_intercept_times: Vec<f32> = Vec::new();
        for r in robots {
            enemy_intercept_times.push(enemy_min_tim_to_intercept(p, r));
        }
        let min_intercept_time = enemy_intercept_times
            .iter()
            .fold(f32::INFINITY, |prev, curr| prev.min(*curr));
        // If positive, the ball should reach the pass destination before an enemy robot can
        // intercept. If the value is negative at all, this means an enemy robot can intercept
        // the pass (possibly before it arrives at it's destination)
        let intercept_time_diff = p.time_to_complete() - min_intercept_time;
        sigmoid(min_intercept_time, 0.2, 0.4)
    } else {
        1.0
    }
}

fn score_pass(
    p: &Pass,
    field: &Field,
    friendly_robots: &Vec<Robot>,
    enemy_robots: &Vec<Robot>,
) -> f32 {
    let static_score = static_score(&p.end, field);
    let friendly_score = friendly_intercept_score(&p, friendly_robots);
    let enemy_score = enemy_intercept_score(&p, enemy_robots);
    static_score * friendly_score * enemy_score
}

fn generate_random_passes(num: usize) -> Vec<Pass> {
    let mut result: Vec<Pass> = Vec::new();
    for _ in 0..num {
        result.push(Pass {
            start: Point {
                x: rand::thread_rng().gen(),
                y: rand::thread_rng().gen(),
            },
            end: Point {
                x: rand::thread_rng().gen(),
                y: rand::thread_rng().gen(),
            },
            speed: 4.0,
            time_offset: 0.15,
        })
    }
    result
}

fn pass_gradient(
    p: &Pass,
    field: &Field,
    friendly_robots: &Vec<Robot>,
    enemy_robots: &Vec<Robot>,
) -> Vec<f32> {
    let base = score_pass(&p, &field, &friendly_robots, &enemy_robots);
    let diff = 1.0e-3;
    let mut p1 = p.clone();
    p1.end.x += diff;
    let mut p2 = p.clone();
    p2.end.y += diff;
    let mut p3 = p.clone();
    p3.speed += diff;
    let mut p4 = p.clone();
    p4.time_offset += diff;

    vec![
        (score_pass(&p1, &field, &friendly_robots, &enemy_robots) - base) / diff,
        (score_pass(&p2, &field, &friendly_robots, &enemy_robots) - base) / diff,
        (score_pass(&p3, &field, &friendly_robots, &enemy_robots) - base) / diff,
        (score_pass(&p4, &field, &friendly_robots, &enemy_robots) - base) / diff,
    ]
}


#[cfg(test)]
mod tests {
    use std::fs;
    use super::*;
    use serde_json;
    use serde_json::Result;
    use serde_json::json;

    #[test]
    fn profile_score_function() {
        let passes = generate_random_passes(18);
        let field = Field::ssl_div_b();
        let friendly_robots: Vec<Robot> = vec![
            Robot {
                id: 0,
                state: KinematicState{
                    position: Point::new(),
                    orientation: Angle::zero(),
                    velocity: Vector::new(),
                    angular_velocity: Angle::zero()
                }
            },
            Robot {
                id: 1,
                state: KinematicState{
                    position: Point { x: 1.0, y: 2.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }
            },
            Robot {
                id: 2,
                state: KinematicState{
                    position: Point { x: 3.0, y: -1.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }
            },
            Robot {
                id: 3,
                state: KinematicState{
                    position: Point { x: -1.0, y: -3.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }
            },
        ];
        let enemy_robots: Vec<Robot> = vec![
            Robot {
                id: 0,
                state: KinematicState{
                    position: Point { x: -1.0, y: 2.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }

            },
            Robot {
                id: 1,
                state: KinematicState{
                    position: Point { x: 1.0, y: -2.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }

            },
            Robot {
                id: 2,
                state: KinematicState{
                    position: Point { x: -3.0, y: 1.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }

            },
            Robot {
                id: 3,
                state: KinematicState{
                    position: Point { x: 1.0, y: 3.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }

            },
        ];

        let start = Instant::now();
        for p in &passes {
            let grad = pass_gradient(p, &field, &friendly_robots, &enemy_robots);
            if grad[0] == 0.392 {
                println!("preventing too much compiler optimization :)");
            }
        }
        let end = Instant::now();
        let total_time_ns = (end - start).as_nanos();
        let total_time_ms = total_time_ns as f64 / 1_000_000.0;
        let time_per_call_ms = total_time_ms / passes.len() as f64;
        let num_passes = passes.len();
        println!("Total time for {num_passes}: {total_time_ms}ms. Time per call: {time_per_call_ms}ms");
    }

    #[test]
    fn plot_score_function() {
        const X_DIVISIONS: usize = 300;
        const Y_DIVISIONS: usize = 300;
        let start = Point{x: 0.5, y: 2.9};
        let speed = 5.0;
        let time_offset = 0.5;
        let field = Field::ssl_div_b();
        let friendly_robots: Vec<Robot> = vec![
            Robot {
                id: 0,
                state: KinematicState{
                    position: Point::new(),
                    orientation: Angle::zero(),
                    velocity: Vector::new(),
                    angular_velocity: Angle::zero()
                }
            },
            Robot {
                id: 1,
                state: KinematicState{
                    position: Point { x: 1.0, y: 2.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }
            },
            // Robot {
            //     id: 2,
            //     state: KinematicState{
            //         position: Point { x: 3.0, y: -1.0 },
            //         velocity: Vector::new(),
            //         orientation: Angle::zero(),
            //         angular_velocity: Angle::zero()
            //     }
            // },
            Robot {
                id: 3,
                state: KinematicState{
                    position: Point { x: -3.0, y: -2.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }
            },
        ];
        let enemy_robots: Vec<Robot> = vec![
            Robot {
                id: 1,
                state: KinematicState{
                    position: Point { x: -1.0, y: -2.0 },
                    velocity: Vector::new(),
                    orientation: Angle::zero(),
                    angular_velocity: Angle::zero()
                }
            },
            // Robot {
            //     id: 2,
            //     state: KinematicState{
            //         position: Point { x: -3.0, y: 1.0 },
            //         velocity: Vector::new(),
            //         orientation: Angle::zero(),
            //         angular_velocity: Angle::zero()
            //     }
            //
            // },
            // Robot {
            //     id: 3,
            //     state: KinematicState{
            //         position: Point { x: 1.0, y: 3.0 },
            //         velocity: Vector::new(),
            //         orientation: Angle::zero(),
            //         angular_velocity: Angle::zero()
            //     }
            //
            // },
        ];

        // let mut x = [[0.0; X_DIVISIONS +1]; Y_DIVISIONS +1];
        // let mut y = [[0.0; X_DIVISIONS +1]; Y_DIVISIONS +1];
        // let mut z = [[0.0; X_DIVISIONS +1]; Y_DIVISIONS +1];
        let mut x: Vec<Vec<f32>> = vec![];
        let mut y: Vec<Vec<f32>> = vec![];
        let mut z: Vec<Vec<f32>> = vec![];

        for yy in 0..Y_DIVISIONS +1 {
            x.push(Vec::with_capacity(X_DIVISIONS+1));
            y.push(Vec::with_capacity(X_DIVISIONS+1));
            z.push(Vec::with_capacity(X_DIVISIONS+1));
            for xx in 0..X_DIVISIONS +1 {
                let x_pos = -field.x_length/2.0 + (xx as f32 / X_DIVISIONS as f32)*field.x_length;
                let y_pos = field.y_length/2.0 - (yy as f32 / Y_DIVISIONS as f32)*field.y_length;
                x[yy].push(x_pos);
                y[yy].push(y_pos);
                let p = Pass{
                    start,
                    end: Point{x: x_pos, y: y_pos},
                    speed,
                    time_offset,
                };
                z[yy].push(score_pass(&p, &field, &friendly_robots, &enemy_robots));
            }
        }

        let mut enemy_robot_info: Vec<[f32; 4]> = vec![];
        for r in &enemy_robots {
            enemy_robot_info.push([
                r.state.position.x, r.state.position.y, r.state.velocity.x, r.state.velocity.y
            ])
        }
        let mut friendly_robot_info: Vec<[f32; 4]> = vec![];
        for r in &friendly_robots {
            friendly_robot_info.push([
                r.state.position.x, r.state.position.y, r.state.velocity.x, r.state.velocity.y
            ])
        }

        let data = json!({
            "speed": speed,
            "time_offset": time_offset,
            "start": [start.x, start.y],
            "enemy_robots": enemy_robot_info,
            "friendly_robots": friendly_robot_info,
            "x": x,
            "y": y,
            "z": z,
            // "z": [
            //     [1, 0, 0, 0, 1],
            //     [0, 0, 0, 0, 0],
            //     [0, 0, 0, 0, 0],
            //     [0, 0, 0, 0, 0],
            //     [1, 0, 0, 0, 1],
            // ],
        });
        // let foo = serde_json::to_string(&data)?;
        let pretty_string = serde_json::to_string_pretty(&data).unwrap();
        // println!("{}", pretty_string);
        // println!("{}, {}, {}", x[0][0], y[0][0], z[0][0]);
        // println!("{}, {}, {}", x[0][X_DIVISIONS], y[0][X_DIVISIONS], z[0][X_DIVISIONS]);
        // println!("{}, {}, {}", x[Y_DIVISIONS][X_DIVISIONS], y[Y_DIVISIONS][X_DIVISIONS], z[Y_DIVISIONS][X_DIVISIONS]);
        // println!("{}, {}, {}", x[Y_DIVISIONS][0], y[Y_DIVISIONS][0], z[Y_DIVISIONS][0]);
        fs::write("/tmp/underbots_passing_plot_data.json", pretty_string).expect("Unable to write pass plot data file");
    }
}
