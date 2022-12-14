use crate::communication::Node;
use crate::geom::{Point, Vector};
use crate::math::{rect_sigmoid, sigmoid};
use crate::motion::bb_time_to_position;
use crate::world::{Field, Robot};
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
    speed: f64,
    time_offset: f64,
}

impl Pass {
    pub fn time_to_complete(&self) -> f64 {
        let pass_dist = (self.end.x - self.start.x).hypot(self.end.y - self.start.y);
        pass_dist / self.speed + self.time_offset
    }
}

fn static_score(p: &Point, field: &Field) -> f64 {
    let enemy_defense_cost = 1.0 - rect_sigmoid(field.enemy_defense_area(), p, 0.5);
    let friendly_defense_cost = 1.0 - rect_sigmoid(field.enemy_defense_area(), p, 0.5);
    let on_field_score = rect_sigmoid(field.touch_lines(), p, 0.5);
    let field_progress_score = sigmoid(p.x, 0.0, field.x_length) / 10.0 + 0.9;
    on_field_score * enemy_defense_cost * friendly_defense_cost * field_progress_score
}

fn friendly_intercept_score(p: &Pass, robots: &Vec<Robot>) -> f64 {
    if !robots.is_empty() {
        let mut times_to_pos: Vec<f64> = Vec::new();
        for r in robots {
            times_to_pos.push(bb_time_to_position(
                &r.position,
                &r.velocity,
                &p.end,
                3.0,
                3.0,
            ));
        }
        let min_time = times_to_pos
            .iter()
            .fold(f64::INFINITY, |prev, curr| prev.min(*curr));
        sigmoid(min_time, 0.5, 1.0)
    } else {
        0.0
    }
}

fn time_to_intercept(p: &Pass, r: &Robot) -> f64 {
    const REACTION_DELAY: f64 = 0.3;
    const NUM_STEPS: usize = 1;
    const ROBOT_RADIUS: f64 = 0.18;
    let x_incr = (p.end.x - p.start.x) / NUM_STEPS as f64;
    let y_incr = (p.end.y - p.start.y) / NUM_STEPS as f64;
    let mut min_diff = f64::INFINITY;
    for i in 0..NUM_STEPS {
        let pos = Point {
            x: p.start.x + i as f64 * x_incr + ROBOT_RADIUS,
            y: p.start.x + i as f64 * y_incr + ROBOT_RADIUS,
        };
        let ttp = bb_time_to_position(&r.position, &r.velocity, &pos, 3.0, 3.0);
        let diff = ttp - p.time_to_complete();
        min_diff = min_diff.min(diff);
    }
    min_diff + REACTION_DELAY
}

fn enemy_intercept_score(p: &Pass, robots: &Vec<Robot>) -> f64 {
    if !robots.is_empty() {
        let mut intercept_diffs: Vec<f64> = Vec::new();
        for r in robots {
            intercept_diffs.push(time_to_intercept(p, r));
        }
        let min_intercept_diff = intercept_diffs
            .iter()
            .fold(f64::INFINITY, |prev, curr| prev.min(*curr));
        sigmoid(min_intercept_diff, 0.2, 0.4)
    } else {
        1.0
    }
}

fn score_pass(
    p: &Pass,
    field: &Field,
    friendly_robots: &Vec<Robot>,
    enemy_robots: &Vec<Robot>,
) -> f64 {
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
) -> Vec<f64> {
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

pub fn profile_pass_function() {
    let passes = generate_random_passes(18);
    let field = Field::ssl_div_b();
    let friendly_robots: Vec<Robot> = vec![
        Robot {
            id: 0,
            position: Point::new(),
            velocity: Vector::new(),
        },
        Robot {
            id: 1,
            position: Point { x: 1.0, y: 2.0 },
            velocity: Vector::new(),
        },
        Robot {
            id: 2,
            position: Point { x: 3.0, y: -1.0 },
            velocity: Vector::new(),
        },
        Robot {
            id: 3,
            position: Point { x: -1.0, y: -3.0 },
            velocity: Vector::new(),
        },
    ];
    let enemy_robots: Vec<Robot> = vec![
        Robot {
            id: 0,
            position: Point { x: -1.0, y: 2.0 },
            velocity: Vector::new(),
        },
        Robot {
            id: 1,
            position: Point { x: 1.0, y: -2.0 },
            velocity: Vector::new(),
        },
        Robot {
            id: 2,
            position: Point { x: -3.0, y: 1.0 },
            velocity: Vector::new(),
        },
        Robot {
            id: 3,
            position: Point { x: 1.0, y: 3.0 },
            velocity: Vector::new(),
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
