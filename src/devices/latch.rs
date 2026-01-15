use macroquad::{
    math::Vec2,
    prelude::Color,
    shapes::{draw_poly, draw_poly_lines},
};

use egui::{FontId, RichText};

use crate::{
    app::{DrawContext, UpdateContext},
    math::V2,
};

#[derive(Clone)]
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

        draw_poly_lines(x, y, 3, radius, -90.0, 2.0, color);
        draw_poly(x, y, 3, radius, -90.0, ctx.colors.bg_1);

        if self.is_on {
            draw_poly(x, y, 3, radius / 2.0, -90.0, color);
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
