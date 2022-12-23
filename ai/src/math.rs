use super::geom::{Point, Rectangle};

// TODO: Possibly useful later: https://docs.rs/fast-math/latest/fast_math/fn.exp.html
pub fn sigmoid(x: f64, offset: f64, width: f64) -> f64 {
    let sig_change_factor = 8.0 / width;

    1.0 / (1.0 + (sig_change_factor * (offset - x)).exp())
}

pub fn rect_sigmoid(rect: Rectangle, p: &Point, width: f64) -> f64 {
    let x_offset = rect.centre().x;
    let y_offset = rect.centre().y;
    let x_size = rect.len_x() / 2.0;
    let y_size = rect.len_y() / 2.0;
    let x_sigmoid = sigmoid(p.x, x_offset + x_size, -width).min(sigmoid(p.x, x_offset - x_size, width));
    let y_sigmoid = sigmoid(p.y, y_offset + y_size, -width).min(sigmoid(p.y, y_offset - y_size, width));
    x_sigmoid * y_sigmoid
}
