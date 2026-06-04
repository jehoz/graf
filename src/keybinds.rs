use std::collections::HashMap;

use bitflags::bitflags;
use macroquad::prelude::{get_keys_pressed, is_key_down, KeyCode};

bitflags! {
    #[derive(PartialEq, Eq, Hash, Clone, Copy)]
    pub struct Modifier: u8 {
        const None = 0b00000000;
        const Ctrl = 0b00000001;
        const Shift = 0b00000010;
        const Alt = 0b00000100;
        const Super = 0b00001000;
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct Input {
    modifiers: Modifier,
    key: KeyCode,
}

impl Input {
    fn new(modifiers: Modifier, key: KeyCode) -> Self {
        Input { modifiers, key }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Action {
    LoadSession,
    SaveSession,
    NewSession,

    CopySelected,
    PasteClipboard,
    DeleteSelected,

    TogglePause,

    CreateClock,
    CreateGate,
    CreateTrigger,
    CreateRandom,
    CreateLatch,
    CreateNote,
}

pub fn process_inputs(keybinds: &HashMap<Input, Action>) -> Vec<Action> {
    let mut modifier = Modifier::None;
    if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
        modifier |= Modifier::Ctrl;
    }
    if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
        modifier |= Modifier::Shift;
    }
    if is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt) {
        modifier |= Modifier::Alt;
    }
    if is_key_down(KeyCode::LeftSuper) || is_key_down(KeyCode::RightSuper) {
        modifier |= Modifier::Super;
    }

    let mut actions = vec![];
    for key in get_keys_pressed().iter() {
        if let Some(action) = keybinds.get(&Input::new(modifier, *key)) {
            actions.push(*action);
        }
    }

    actions
}

pub fn default_keybinds() -> HashMap<Input, Action> {
    use Action::*;

    HashMap::from([
        (Input::new(Modifier::Ctrl, KeyCode::O), LoadSession),
        (Input::new(Modifier::Ctrl, KeyCode::S), SaveSession),
        (Input::new(Modifier::Ctrl, KeyCode::N), NewSession),
        (Input::new(Modifier::Ctrl, KeyCode::C), CopySelected),
        (Input::new(Modifier::Ctrl, KeyCode::V), PasteClipboard),
        (Input::new(Modifier::None, KeyCode::Delete), DeleteSelected),
        (Input::new(Modifier::None, KeyCode::Space), TogglePause),
        (Input::new(Modifier::Shift, KeyCode::C), CreateClock),
        (Input::new(Modifier::Shift, KeyCode::G), CreateGate),
        (Input::new(Modifier::Shift, KeyCode::R), CreateRandom),
        (Input::new(Modifier::Shift, KeyCode::T), CreateTrigger),
        (Input::new(Modifier::Shift, KeyCode::L), CreateLatch),
        (Input::new(Modifier::Shift, KeyCode::N), CreateNote),
    ])
}
