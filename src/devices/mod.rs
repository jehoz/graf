use egui::Ui;

use crate::{
    app::{DrawContext, UpdateContext},
    math::V2,
};

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

#[derive(Clone)]
pub enum Device {
    Clock(Clock),
    Gate(Gate),
    Latch(Latch),
    Note(Note),
    Trigger(Trigger),
}

impl Device {
    pub fn update(&mut self, ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        match self {
            Device::Clock(ref mut d) => d.update(ctx, inputs),
            Device::Gate(ref mut d) => d.update(ctx, inputs),
            Device::Latch(ref mut d) => d.update(ctx, inputs),
            Device::Note(ref mut d) => d.update(ctx, inputs),
            Device::Trigger(ref mut d) => d.update(ctx, inputs),
        }
    }

    pub fn draw(&self, ctx: &DrawContext, position: V2, size: f32, is_selected: bool) {
        match self {
            Device::Clock(d) => d.draw(ctx, position, size, is_selected),
            Device::Gate(d) => d.draw(ctx, position, size, is_selected),
            Device::Latch(d) => d.draw(ctx, position, size, is_selected),
            Device::Note(d) => d.draw(ctx, position, size, is_selected),
            Device::Trigger(d) => d.draw(ctx, position, size, is_selected),
        }
    }

    pub fn reset(&mut self) {
        match self {
            Device::Clock(ref mut d) => d.reset(),
            Device::Gate(_) => {} // gates do not have any state that needs to be reset
            Device::Latch(ref mut d) => d.reset(),
            Device::Note(ref mut d) => d.reset(),
            Device::Trigger(ref mut d) => d.reset(),
        }
    }

    pub fn inspector(&mut self, ui: &mut Ui) {
        match self {
            Device::Clock(ref mut d) => d.inspector(ui),
            Device::Gate(ref mut d) => d.inspector(ui),
            Device::Latch(ref mut d) => d.inspector(ui),
            Device::Note(ref mut d) => d.inspector(ui),
            Device::Trigger(ref mut d) => d.inspector(ui),
        }
    }

    // number of input wires that can be plugged into the device
    pub fn input_arity(&self) -> Arity {
        match self {
            Device::Clock(_) => Arity::Nullary,
            Device::Gate(_) => Arity::NAry,
            Device::Latch(_) => Arity::Unary,
            Device::Note(_) => Arity::Unary,
            Device::Trigger(_) => Arity::Unary,
        }
    }

    // can there be wires coming out of this device?
    pub fn has_output(&self) -> bool {
        if let Device::Note(_) = self {
            false
        } else {
            true
        }
    }
}
