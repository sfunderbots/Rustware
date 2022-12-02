mod geom;
mod math;
mod world;
mod motion;

use crate::geom::{Rectangle, Point, Vector};
use crate::world::{Field, Robot};
use crate::math::{sigmoid, rect_sigmoid};
use std::time::Instant;

struct Pass {
    start: Point,
    end: Point,
    speed: f32,
    time_offset: f32
}

fn static_score(p: &Point, field: &Field) -> f32 {
    let enemy_defense_cost = 1.0 - rect_sigmoid(field.enemy_defense_area(), p, 0.5);
    let friendly_defense_cost = 1.0 - rect_sigmoid(field.enemy_defense_area(), p, 0.5);
    let on_field_score = rect_sigmoid(field.touch_lines(), p, 0.5);
    let field_progress_score = sigmoid(p.x, 0.0, field.x_length) / 10.0 + 0.9;
    on_field_score * enemy_defense_cost * friendly_defense_cost * field_progress_score
}

fn score_pass(p: &Pass, field: &Field) -> f32 {
    let static_score = static_score(&p.end, field);
    static_score
}

fn generate_random_passes(num: usize) -> Vec<Pass> {
    let mut result: Vec<Pass> = Vec::new();
    for i in (0..num-1) {
        result.push(Pass{
             start: Point::new(),
            end: Point::new(),
            speed: 4.0,
            time_offset: 0.15
        })
    }
    result
}



fn main() {
    let p = geom::Point::new();
    let passes = generate_random_passes(1000);
    let field = Field::ssl_div_b();
    // println!("Hello, world! {}", math::sigmoid());

    let start = Instant::now();
    for p in &passes {
        let score = score_pass(p, &field);
        if score == 0.392 {
            println!("preventing too much compiler optimization :)");
        }
    }
    let end = Instant::now();
    let total_time_ns = (end-start).as_nanos();
    let total_time_ms = total_time_ns as f64 / 1_000_000.0;
    let time_per_call_ms = total_time_ms / passes.len() as f64;
    println!("Total time: {total_time_ms}ms. Time per call: {time_per_call_ms}ms");
}
