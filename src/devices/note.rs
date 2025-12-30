use egui::{DragValue, FontId, RichText};
use macroquad::{math::Vec2, shapes::draw_hexagon};

use crate::{
    app::DrawContext, midi::MidiEventSender, session::UpdateContext,
    widgets::note_picker::NotePicker,
};

use super::{Arity, Device};

#[derive(Clone, Copy, PartialEq)]
pub enum PitchClass {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl ToString for PitchClass {
    fn to_string(&self) -> String {
        match *self {
            PitchClass::C => "C".to_string(),
            PitchClass::Cs => "C#".to_string(),
            PitchClass::D => "D".to_string(),
            PitchClass::Ds => "D#".to_string(),
            PitchClass::E => "E".to_string(),
            PitchClass::F => "F".to_string(),
            PitchClass::Fs => "F#".to_string(),
            PitchClass::G => "G".to_string(),
            PitchClass::Gs => "G#".to_string(),
            PitchClass::A => "A".to_string(),
            PitchClass::As => "A#".to_string(),
            PitchClass::B => "B".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Note {
    midi_channel: u8,
    octave: u8,
    pitch_class: PitchClass,
    velocity: u8,

    event_sender: MidiEventSender,

    is_on: bool,
}

impl Note {
    pub fn new(event_sender: MidiEventSender) -> Self {
        Note {
            midi_channel: 0,
            octave: 4,
            pitch_class: PitchClass::C,
            velocity: 100,

            event_sender,

            is_on: false,
        }
    }

    fn midi_key(&self) -> u8 {
        self.pitch_class as u8 + self.octave * 12
    }

    fn turn_on(&mut self) {
        if self.is_on {
            return;
        }

        let event = (
            self.midi_channel.into(),
            midly::MidiMessage::NoteOn {
                key: self.midi_key().into(),
                vel: self.velocity.into(),
            },
        );
        self.event_sender.send(event);

        self.is_on = true;
    }

    fn turn_off(&mut self) {
        if !self.is_on {
            return;
        }

        let event = (
            self.midi_channel.into(),
            midly::MidiMessage::NoteOff {
                key: self.midi_key().into(),
                vel: self.velocity.into(),
            },
        );
        self.event_sender.send(event);

        self.is_on = false;
    }
}

impl Drop for Note {
    fn drop(&mut self) {
        self.turn_off();
    }
}

impl Device for Note {
    fn update(&mut self, ctx: &mut UpdateContext, inputs: Vec<bool>) -> Option<bool> {
        if ctx.is_paused {
            self.turn_off();
            return None;
        }

        if let Some(input_on) = inputs.first() {
            if *input_on {
                self.turn_on();
            } else {
                self.turn_off();
            }
        } else {
            self.turn_off();
        }
        None
    }

    fn draw(&self, ctx: &DrawContext, position: Vec2, size: f32, is_selected: bool) {
        let Vec2 { x, y } = position;
        let radius = size / 2.0;

        if is_selected {
            draw_hexagon(
                x,
                y,
                radius + 4.0,
                1.0,
                false,
                ctx.colors.fg_0,
                ctx.colors.bg_0.with_alpha(0.0),
            );
        }

        draw_hexagon(x, y, radius, 1.0, false, ctx.colors.fg_0, ctx.colors.bg_1);

        if self.is_on {
            draw_hexagon(
                x,
                y,
                radius / 2.0,
                0.0,
                false,
                ctx.colors.bg_1,
                ctx.colors.fg_0,
            );
        }
    }

    fn reset(&mut self) {
        self.turn_off();
    }

    fn inspector(&mut self, ui: &mut egui::Ui) {
        let mut octave = self.octave;
        let mut pitch = self.pitch_class;

        ui.label(
            RichText::new("Note")
                .font(FontId::proportional(16.0))
                .strong(),
        );
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Octave");

            let btn_size = egui::vec2(20.0, 20.0);
            let dec_btn = ui.add_sized(btn_size, egui::Button::new("-"));
            if dec_btn.clicked() {
                octave -= 1;
            }

            ui.add(DragValue::new(&mut octave).range(0..=8));

            let inc_btn = ui.add_sized(btn_size, egui::Button::new("+"));
            if inc_btn.clicked() {
                octave += 1;
            }
        });

        ui.add_space(2.0);

        ui.add(NotePicker::new(&mut pitch));

        if self.octave != octave || self.pitch_class != pitch {
            self.turn_off();
            self.octave = octave;
            self.pitch_class = pitch;
        }

        ui.add(egui::Slider::new(&mut self.velocity, 0..=127).text("Velocity"));

        ui.add_space(2.0);

        ui.horizontal(|ui| {
            ui.label("MIDI Channel");
            ui.add(DragValue::new(&mut self.midi_channel).range(0..=15));
        });
    }

    fn input_arity(&self) -> Arity {
        Arity::Unary
    }

    fn has_output(&self) -> bool {
        false
    }

    fn clone_dyn(&self) -> Box<dyn Device> {
        Box::new(self.clone())
    }
}
