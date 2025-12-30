use std::time::Duration;

use macroquad::{
    math::Vec2,
    shapes::{draw_poly, draw_poly_lines},
};

use egui::{DragValue, FontId, RichText, Slider};

use crate::session::UpdateContext;
use crate::{
    app::DrawContext,
    devices::{Arity, Device},
};

#[derive(Clone)]
pub struct Trigger {
    // duration in milliseconds that trigger will stay on after being set off
    duration: f32,

    bpm_sync: bool,
    bpm_duration: (u32, u32),

    // can the trigger be set off again before finishing?
    retrigger_mode: bool,

    ready_to_fire: bool,
    time_remaining: Option<f32>,

    prev_clock_time: Duration,
}

impl Trigger {
    pub fn new() -> Self {
        Trigger {
            duration: 500.0,
            bpm_sync: false,
            bpm_duration: (1, 4),
            retrigger_mode: false,

            ready_to_fire: true,
            time_remaining: None,

            prev_clock_time: Duration::ZERO,
        }
    }

    fn fire(&mut self, ctx: &UpdateContext) {
        let duration = if self.bpm_sync {
            let (numerator, denominator) = self.bpm_duration;
            let beats = (numerator as f32 / denominator as f32) * 4.0;
            let ms_per_beat = 60000.0 / ctx.bpm as f32;
            beats * ms_per_beat
        } else {
            self.duration
        };

        self.ready_to_fire = false;
        self.time_remaining = Some(duration);
    }
}

impl Device for Trigger {
    fn update(&mut self, ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        let input_on = inputs.first().map(|x| *x).unwrap_or(false);
        if self.retrigger_mode {
            if input_on && self.ready_to_fire {
                self.fire(ctx);
            }
        } else {
            if input_on && self.ready_to_fire && self.time_remaining == None {
                self.fire(ctx);
                // this is certainly not the cleanest solution,  but I want to guarantee that
                // non-retrigger-mode triggers output at least one frame of false before firing
                // again.
                // should probably refactor at some point
                return Some(false);
            }
        }

        if !input_on {
            self.ready_to_fire = true;
        }

        let delta_t = ctx.free_clock - self.prev_clock_time;
        self.prev_clock_time = ctx.free_clock;

        if let Some(t_prev) = self.time_remaining {
            let t = t_prev - delta_t.as_secs_f32() * 1000.0;
            if t > 0.0 {
                self.time_remaining = Some(t);
            } else {
                self.time_remaining = None;
            }
            Some(true)
        } else {
            Some(false)
        }
    }

    fn draw(&self, ctx: &DrawContext, position: Vec2, size: f32, is_selected: bool) {
        let Vec2 { x, y } = position;
        let radius = size / 2.0;

        if is_selected {
            draw_poly_lines(
                x,
                y,
                3,
                radius + 4.0,
                90.0,
                1.0,
                ctx.colors.fg_0.with_alpha(0.5),
            );
        }

        draw_poly_lines(x, y, 3, radius, 90.0, 2.0, ctx.colors.fg_0);
        draw_poly(x, y, 3, radius, 90.0, ctx.colors.bg_1);

        if let Some(t_rem) = self.time_remaining {
            let percent_done = (t_rem / self.duration).clamp(0.0, 1.0);
            draw_poly(x, y, 3, radius * percent_done, 90.0, ctx.colors.fg_0);
        }
    }

    fn reset(&mut self) {
        self.ready_to_fire = true;
        self.time_remaining = None;

        self.prev_clock_time = Duration::ZERO;
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Trigger")
                .font(FontId::proportional(16.0))
                .strong(),
        );
        ui.separator();

        ui.checkbox(&mut self.retrigger_mode, "Retrigger Mode");

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
            ui.add(
                Slider::new(&mut self.duration, 1f32..=10000f32)
                    .text("Duration")
                    .suffix("ms"),
            );
        }
    }

    fn input_arity(&self) -> super::Arity {
        Arity::Unary
    }

    fn has_output(&self) -> bool {
        true
    }

    fn clone_dyn(&self) -> Box<dyn Device> {
        Box::new(self.clone())
    }
}
