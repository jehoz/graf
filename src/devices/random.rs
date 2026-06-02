use egui::{FontId, RichText, Slider};
use macroquad::{math::Vec2, prelude::Color, shapes::draw_rectangle_lines};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    app::{DrawContext, UpdateContext},
    math::V2,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Random {
    // the probability (between 0 and 1) of generating a true value
    probability: f32,

    //
    value: bool,

    // the value of the input from the previous frame
    last_input: bool,
}

impl Random {
    pub fn new() -> Self {
        Random {
            probability: 0.5,
            value: false,
            last_input: false,
        }
    }

    pub fn update(&mut self, ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        let input = inputs.first().map(|x| *x).unwrap_or(false);
        if input == true && self.last_input == false {
            // sample new random value
            self.value = ctx.rng.random_bool(self.probability as f64);
        }

        self.last_input = input;

        return Some(self.value);
    }

    pub fn draw(&self, _ctx: &DrawContext, position: V2, size: f32, color: Color) {
        let Vec2 { x, y } = position.into();
        // let radius = size / 2.0;

        draw_rectangle_lines(x - size / 2., y - size / 2., size, size, 1.0, color);
    }

    pub fn reset(&mut self) {}

    pub fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Random")
                .font(FontId::proportional(16.0))
                .strong(),
        );
        ui.separator();

        ui.add(Slider::new(&mut self.probability, 0f32..=1.0f32).text("Probability"));
    }
}
