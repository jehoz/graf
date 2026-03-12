use std::collections::HashMap;

use bitflags::bitflags;
use macroquad::input::KeyCode;

bitflags! {
    #[derive(PartialEq, Eq, Hash)]
    pub struct Modifier: u8 {
        const None = 0b00000000;
        const Ctrl = 0b00000001;
        const Shift = 0b00000010;
        const Alt = 0b00000100;
        const Super = 0b00001000;
    }
}

#[derive(PartialEq, Eq, Hash)]
struct Input {
    modifiers: Modifier,
    key: KeyCode,
}

impl Input {
    fn new(modifiers: Modifier, key: KeyCode) -> Self {
        Input { modifiers, key }
    }
}

enum Action {
    LoadSession,
    SaveSession,
    NewSession,

    CopySelected,
    PasteClipboard,
    DeleteSelected,

    TogglePause,
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
    ])
}
