use crate::game::{X_SHIFT, Y_SHIFT};
use bevy::prelude::*;

pub fn col_to_x(col: i32, size: f32) -> f32 {
    col as f32 * size - X_SHIFT
}

pub fn row_to_y(row: i32, size: f32) -> f32 {
    -(row as f32) * size + Y_SHIFT
}

pub fn translation_to_rect(translation: Vec3, size: f32) -> Rect {
    Rect::new(
        translation.x - size / 2.,
        translation.y - size / 2.,
        translation.x + size / 2.,
        translation.y + size / 2.,
    )
}
