use super::geom::{Point, Rectangle};
use std::{self, cmp::min};

// TODO: Possibly useful later: https://docs.rs/fast-math/latest/fast_math/fn.exp.html
pub fn sigmoid(x: f32, offset: f32, width: f32) -> f32 {
    let sig_change_factor = 8.0 / width;

    1.0 / (1.0 + (sig_change_factor * (offset - x)).exp())
}

pub fn rect_sigmoid(rect: Rectangle, p: &Point, width: f32) -> f32 {
    let x_offset = rect.centre().x;
    let y_offset = rect.centre().y;
    let x_size = rect.len_x() / 2.0;
    let y_size = rect.len_y() / 2.0;
    let x = p.x;
    let y = p.y;
    let x_sigmoid =
        sigmoid(p.x, x_offset + x_size, -width).min(sigmoid(x, x_offset - x_size, width));
    let y_sigmoid =
        sigmoid(p.y, y_offset + y_size, -width).min(sigmoid(y, y_offset - y_size, width));
    x_sigmoid * y_sigmoid
}
