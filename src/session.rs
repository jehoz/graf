use serde::{Deserialize, Serialize};

use crate::{
    app::{DrawContext, UpdateContext},
    circuit::Circuit,
    dag::DeviceId,
};

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub circuit: Circuit,
    pub bpm: u32,
}

impl Session {
    pub fn new() -> Self {
        Session {
            circuit: Circuit::new(),
            bpm: 120,
        }
    }

    pub fn update(&mut self, ctx: &mut UpdateContext) {
        ctx.bpm = self.bpm;
        self.circuit.update(ctx);
    }

    pub fn draw(&self, ctx: &DrawContext, selected: &[DeviceId]) {
        self.circuit
            .draw(ctx, ctx.colors.fg_1, selected, ctx.colors.selected);
    }
}
