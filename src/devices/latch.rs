use macroquad::{
    math::Vec2,
    prelude::Color,
    shapes::{draw_rectangle, draw_rectangle_lines},
};
use serde::{Deserialize, Serialize};

use egui::{FontId, RichText};

use crate::{
    app::{DrawContext, UpdateContext},
    math::V2,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Latch {
    is_on: bool,
    prev_input: bool,
}

impl Latch {
    pub fn new() -> Self {
        Latch {
            is_on: false,
            prev_input: false,
        }
    }

    pub fn update(&mut self, _ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        let input_on = inputs.first().map(|x| *x).unwrap_or(false);

        if input_on && !self.prev_input {
            self.is_on = !self.is_on;
        }
        self.prev_input = input_on;

        Some(self.is_on)
    }

    pub fn draw(&self, ctx: &DrawContext, position: V2, size: f32, color: Color) {
        let Vec2 { x, y } = position.into();
        let radius = size / 2.0;

        let inner_padding = 0.3 * radius;
        let inner_width = 0.7 * radius;

        if self.is_on {
            draw_rectangle_lines(x - radius, y - radius * 0.67, size, size * 0.67, 2.0, color);
            draw_rectangle(
                (x + radius) - (inner_padding + inner_width),
                y - inner_width * 0.5,
                inner_width,
                inner_width,
                color,
            );
        } else {
            draw_rectangle_lines(x - radius, y - radius * 0.67, size, size * 0.67, 2.0, color);
            draw_rectangle(
                (x - radius) + inner_padding,
                y - inner_width * 0.5,
                inner_width,
                inner_width,
                color,
            );
        }
    }

    pub fn reset(&mut self) {
        self.is_on = false;
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Latch")
                .font(FontId::proportional(16.0))
                .strong(),
        );
        ui.separator();

        ui.checkbox(&mut self.is_on, "On");
    }
}
