use egui::{FontId, RichText};
use macroquad::{
    math::Vec2,
    prelude::Color,
    shapes::{draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines},
};
use serde::{Deserialize, Serialize};

use crate::app::{DrawContext, UpdateContext};
use crate::math::V2;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum BooleanOperation {
    AND,
    OR,
    XOR,
    NAND,
    NOR,
    XNOR,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Gate {
    operation: BooleanOperation,
}

impl Gate {
    pub fn new() -> Self {
        Gate {
            operation: BooleanOperation::AND,
        }
    }

    pub fn update(&mut self, _ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        let out = match self.operation {
            BooleanOperation::AND => inputs.iter().fold(true, |acc, x| acc && *x),
            BooleanOperation::OR => inputs.iter().fold(false, |acc, x| acc || *x),
            BooleanOperation::XOR => inputs.iter().fold(false, |acc, x| acc != *x),
            BooleanOperation::NAND => !inputs.iter().fold(true, |acc, x| acc && *x),
            BooleanOperation::NOR => !inputs.iter().fold(false, |acc, x| acc || *x),
            BooleanOperation::XNOR => inputs.iter().fold(false, |acc, x| acc == *x),
        };
        Some(out)
    }

    pub fn draw(&self, ctx: &DrawContext, position: V2, size: f32, color: Color) {
        let Vec2 { x, y } = position.into();

        draw_rectangle(x - size / 2., y - size / 2., size, size, ctx.colors.bg_1);
        draw_rectangle_lines(x - size / 2., y - size / 2., size, size, 1.0, color);

        draw_symbol(ctx, x, y, size * 0.5, &self.operation, color);
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Gate")
                .font(FontId::proportional(16.0))
                .strong(),
        );
        ui.separator();

        egui::Grid::new("gate_buttons")
            .num_columns(3)
            .show(ui, |ui| {
                ui.add_enabled_ui(self.operation != BooleanOperation::AND, |ui| {
                    if ui.button("AND").clicked() {
                        self.operation = BooleanOperation::AND;
                    }
                });
                ui.add_enabled_ui(self.operation != BooleanOperation::OR, |ui| {
                    if ui.button("OR").clicked() {
                        self.operation = BooleanOperation::OR;
                    }
                });
                ui.add_enabled_ui(self.operation != BooleanOperation::XOR, |ui| {
                    if ui.button("XOR").clicked() {
                        self.operation = BooleanOperation::XOR;
                    }
                });
                ui.end_row();

                ui.add_enabled_ui(self.operation != BooleanOperation::NAND, |ui| {
                    if ui.button("NAND").clicked() {
                        self.operation = BooleanOperation::NAND;
                    }
                });
                ui.add_enabled_ui(self.operation != BooleanOperation::NOR, |ui| {
                    if ui.button("NOR").clicked() {
                        self.operation = BooleanOperation::NOR;
                    }
                });
                ui.add_enabled_ui(self.operation != BooleanOperation::XNOR, |ui| {
                    if ui.button("XNOR").clicked() {
                        self.operation = BooleanOperation::XNOR;
                    }
                });
                ui.end_row();
            });
    }
}

fn draw_symbol(ctx: &DrawContext, x: f32, y: f32, scale: f32, op: &BooleanOperation, color: Color) {
    let top = y - scale / 2.0;
    let bottom = y + scale / 2.0;
    let left = x - scale / 2.0;
    let right = x + scale / 2.0;

    match op {
        BooleanOperation::AND => {
            draw_line(left, bottom, x, top, 1.0, color);
            draw_line(x, top, right, bottom, 1.0, color);
        }
        BooleanOperation::OR => {
            draw_line(left, top, x, bottom, 1.0, color);
            draw_line(x, bottom, right, top, 1.0, color);
        }
        BooleanOperation::XOR => {
            draw_circle_lines(x, y, scale / 2.0, 1.0, color);
            draw_line(x, top, x, bottom, 1.0, color);
            draw_line(left, y, right, y, 1.0, color);
        }
        BooleanOperation::NAND => {
            draw_line(left, bottom, x, y, 1.0, color);
            draw_line(x, y, right, bottom, 1.0, color);
            draw_line(left, top, right, top, 1.0, color);
        }
        BooleanOperation::NOR => {
            draw_line(left, y, x, bottom, 1.0, color);
            draw_line(x, bottom, right, y, 1.0, color);
            draw_line(left, top, right, top, 1.0, color);
        }
        BooleanOperation::XNOR => {
            draw_line(left, top, right, top, 1.0, color);
            draw_line(left, y, right, y, 1.0, color);
            draw_line(left, bottom, right, bottom, 1.0, color);
        }
    }
}
