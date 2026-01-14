use egui::Ui;

use crate::{app::DrawContext, math::V2, session::UpdateContext};

pub mod clock;
pub mod gate;
pub mod latch;
pub mod note;
pub mod trigger;

pub use clock::Clock;
pub use gate::Gate;
pub use latch::Latch;
pub use note::Note;
pub use trigger::Trigger;

#[derive(PartialEq)]
pub enum Arity {
    Nullary,
    Unary,
    NAry,
}

pub trait Device {
    fn update(&mut self, ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool>;
    fn draw(&self, ctx: &DrawContext, position: V2, size: f32, is_selected: bool);
    fn reset(&mut self) {}

    fn inspector(&mut self, ui: &mut Ui);

    // number of input wires that can be plugged into the device
    fn input_arity(&self) -> Arity;

    // can there be wires coming out of this device?
    fn has_output(&self) -> bool;
}

pub enum AnyDevice {
    Clock(Clock),
    Gate(Gate),
    Latch(Latch),
    Note(Note),
    Trigger(Trigger),
}

impl Device for AnyDevice {
    fn update(&mut self, ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        match self {
            AnyDevice::Clock(ref mut d) => d.update(ctx, inputs),
            AnyDevice::Gate(ref mut d) => d.update(ctx, inputs),
            AnyDevice::Latch(ref mut d) => d.update(ctx, inputs),
            AnyDevice::Note(ref mut d) => d.update(ctx, inputs),
            AnyDevice::Trigger(ref mut d) => d.update(ctx, inputs),
        }
    }

    fn draw(&self, ctx: &DrawContext, position: V2, size: f32, is_selected: bool) {
        match self {
            AnyDevice::Clock(d) => d.draw(ctx, position, size, is_selected),
            AnyDevice::Gate(d) => d.draw(ctx, position, size, is_selected),
            AnyDevice::Latch(d) => d.draw(ctx, position, size, is_selected),
            AnyDevice::Note(d) => d.draw(ctx, position, size, is_selected),
            AnyDevice::Trigger(d) => d.draw(ctx, position, size, is_selected),
        }
    }

    fn reset(&mut self) {
        match self {
            AnyDevice::Clock(ref mut d) => d.reset(),
            AnyDevice::Gate(ref mut d) => d.reset(),
            AnyDevice::Latch(ref mut d) => d.reset(),
            AnyDevice::Note(ref mut d) => d.reset(),
            AnyDevice::Trigger(ref mut d) => d.reset(),
        }
    }

    fn inspector(&mut self, ui: &mut Ui) {
        match self {
            AnyDevice::Clock(ref mut d) => d.inspector(ui),
            AnyDevice::Gate(ref mut d) => d.inspector(ui),
            AnyDevice::Latch(ref mut d) => d.inspector(ui),
            AnyDevice::Note(ref mut d) => d.inspector(ui),
            AnyDevice::Trigger(ref mut d) => d.inspector(ui),
        }
    }

    // number of input wires that can be plugged into the device
    fn input_arity(&self) -> Arity {
        match self {
            AnyDevice::Clock(_) => Arity::Nullary,
            AnyDevice::Gate(_) => Arity::NAry,
            AnyDevice::Latch(_) => Arity::Unary,
            AnyDevice::Note(_) => Arity::Unary,
            AnyDevice::Trigger(_) => Arity::Unary,
        }
    }

    // can there be wires coming out of this device?
    fn has_output(&self) -> bool {
        if let AnyDevice::Note(_) = self {
            false
        } else {
            true
        }
    }
}
