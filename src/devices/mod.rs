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

    // need this so we can copy and paste devices in the session
    fn clone_dyn(&self) -> Box<dyn Device>;
}

/// Wrapper over all `Device` types for static dispatch (using `Box<dyn Device>` was causing some headaches).
///
/// A variant will need to be added for any new devices.
pub enum AnyDevice {
    Clock(Clock),
    Gate(Gate),
    Latch(Latch),
    Note(Note),
    Trigger(Trigger),
}
