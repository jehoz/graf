use egui::{DragValue, FontId, RichText, Slider};
use macroquad::math::Vec2;
use macroquad::shapes::{draw_arc, draw_circle, draw_circle_lines};

use crate::{app::DrawContext, math::V2, session::UpdateContext};

#[derive(Clone)]
pub struct Clock {
    // if true, the clock's cycle duration is a fraction of a note length
    bpm_sync: bool,

    // duration of clock cycle in milliseconds if not BPM synced
    free_duration: f32,

    // duration of clock cycle as fraction of note length if BPM synced
    bpm_duration: (u32, u32),

    // what proportion of the cycle is output "on" (value from 0 to 1)
    gate: f32,

    offset: f32,

    cycle_position: f32,
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            bpm_sync: true,
            free_duration: 500.0,
            bpm_duration: (1, 4),
            gate: 0.5,
            offset: 0.,

            cycle_position: 0.0,
        }
    }

    pub fn update(&mut self, ctx: &mut UpdateContext, _inputs: Vec<bool>) -> Option<bool> {
        if self.bpm_sync {
            let (numerator, denominator) = self.bpm_duration;
            let beat_period = (numerator as f32 / denominator as f32) * 4.0;

            self.cycle_position = ((ctx.beat_clock / beat_period) + self.offset) % 1.0;
        } else {
            self.cycle_position =
                ((ctx.free_clock.as_secs_f32() * 1000.0 / self.free_duration) + self.offset) % 1.0;
        }

        if self.cycle_position <= self.gate {
            Some(true)
        } else {
            Some(false)
        }
    }

    pub fn draw(&self, ctx: &DrawContext, position: V2, size: f32, is_selected: bool) {
        let radius = size / 2.0;
        let Vec2 { x, y } = position.into();

        if is_selected {
            draw_circle_lines(x, y, radius + 4.0, 1.0, ctx.colors.fg_0.with_alpha(0.5));
        }

        draw_circle_lines(x, y, radius, 1.0, ctx.colors.fg_0);
        draw_circle(x, y, radius, ctx.colors.bg_1);

        draw_arc(
            x,
            y,
            32,
            0.0,
            360.0 * (self.cycle_position - self.gate) - 90.0,
            radius,
            360.0 * self.gate,
            ctx.colors.fg_0,
        );
    }

    pub fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Clock")
                .font(FontId::proportional(16.0))
                .strong(),
        );
        ui.separator();

        ui.label("Rate");
        ui.checkbox(&mut self.bpm_sync, "BPM Sync");
        if self.bpm_sync {
            let (n, d) = &mut self.bpm_duration;
            ui.horizontal(|ui| {
                ui.label("Note Length");
                ui.add(DragValue::new(n).range(1..=256));
                ui.label("/");
                ui.add(DragValue::new(d).range(1..=256));
            });
        } else {
            ui.add(Slider::new(&mut self.free_duration, 1f32..=10000f32).text("Period"));
        }

        ui.add(Slider::new(&mut self.gate, 0f32..=1.0f32).text("Gate"));
        ui.add(Slider::new(&mut self.offset, 0f32..=1.0f32).text("Offset"));
    }

    pub fn reset(&mut self) {
        self.cycle_position = 0.0;
    }
}
