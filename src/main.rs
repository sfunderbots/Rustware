mod geom;
mod math;
mod world;
mod motion;

use crate::geom::{Rectangle, Point};
use crate::world::{Field, Robot};
use crate::math::{sigmoid, rect_sigmoid};

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


fn main() {
    let p = geom::Point::new();
    // println!("Hello, world! {}", math::sigmoid());

}
