use core::panic;
use rand::prelude::*;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use egui::{menu, style::WidgetVisuals, style::Widgets, Align2, CornerRadius, Stroke, Visuals};
use macroquad::{
    input::{
        is_key_down, is_key_pressed, is_mouse_button_pressed, is_mouse_button_released,
        mouse_position, KeyCode, MouseButton,
    },
    shapes::draw_rectangle_lines,
    window::clear_background,
};
use rfd::FileDialog;

use crate::{
    circuit::Circuit,
    dag::{DeviceId, WireType},
    devices::{
        clock::Clock, gate::Gate, latch::Latch, note::Note, random::Random, trigger::Trigger,
        Device,
    },
    drawing_utils::{
        color_to_color32, draw_wire_between_devices, draw_wire_from_device, ColorPalette,
    },
    keybinds::{default_keybinds, process_inputs, Action, Input},
    math::{Rect, V2},
    midi::MidiConfig,
    session::Session,
};

enum CursorState {
    Idle,
    DraggingSelectedDevices(V2),
    DraggingLooseWire(DeviceId, WireType),
    DraggingConnectedWire(DeviceId, DeviceId, WireType),
    DraggingInvalidWire(DeviceId, WireType),
    DraggingSelectBox(V2),
    PanningViewport(V2),
    PendingDevices(V2),
}

const INSPECTOR_WIDTH: f32 = 200.0;

pub struct UpdateContext {
    pub beat_clock: f32,
    pub free_clock: Duration,

    pub this_update: Instant,
    pub last_update: Instant,

    pub is_paused: bool,

    pub rng: ThreadRng,
    pub midi_config: MidiConfig,

    // BPM is updated by the session when context is passed to the update method so that devices
    // have access to the session tempo.
    // Feels hacky... should think about a more elegant way to do this
    pub bpm: u32,
}

impl UpdateContext {
    pub fn new() -> Self {
        UpdateContext {
            beat_clock: 0.0,
            free_clock: Duration::ZERO,

            this_update: Instant::now(),
            last_update: Instant::now(),

            is_paused: false,

            rng: rand::rng(),
            midi_config: MidiConfig::new(),

            bpm: 0,
        }
    }
}

pub struct DrawContext {
    pub colors: ColorPalette,
    pub viewport_offset: V2,
    pub egui_visuals: Visuals,
}

impl DrawContext {
    pub fn new(colors: ColorPalette) -> Self {
        let bg_0 = color_to_color32(colors.bg_0);
        let bg_1 = color_to_color32(colors.bg_1);
        let bg_2 = color_to_color32(colors.bg_2);
        let bg_3 = color_to_color32(colors.bg_3);
        let fg_0 = color_to_color32(colors.fg_0);
        let fg_1 = color_to_color32(colors.fg_1);
        let fg_2 = color_to_color32(colors.fg_2);
        let fg_3 = color_to_color32(colors.fg_3);

        let visuals = egui::Visuals {
            extreme_bg_color: bg_0,
            faint_bg_color: bg_2,
            override_text_color: Some(fg_0),
            window_fill: bg_1,
            panel_fill: bg_1,

            window_corner_radius: CornerRadius::ZERO,
            menu_corner_radius: CornerRadius::ZERO,
            widgets: Widgets {
                noninteractive: WidgetVisuals {
                    bg_fill: bg_0,
                    weak_bg_fill: bg_0,
                    bg_stroke: Stroke::new(1.0, bg_2),
                    fg_stroke: Stroke::new(1.0, fg_3),
                    corner_radius: CornerRadius::ZERO,
                    expansion: 0.0,
                },
                inactive: WidgetVisuals {
                    bg_fill: bg_2,
                    weak_bg_fill: bg_2,
                    bg_stroke: Default::default(),
                    fg_stroke: Stroke::new(1.0, fg_2),
                    corner_radius: CornerRadius::ZERO,
                    expansion: 0.0,
                },
                hovered: WidgetVisuals {
                    bg_fill: bg_3,
                    weak_bg_fill: bg_3,
                    bg_stroke: Stroke::new(1.0, fg_3),
                    fg_stroke: Stroke::new(1.0, fg_1),
                    corner_radius: CornerRadius::ZERO,
                    expansion: 0.0,
                },
                active: WidgetVisuals {
                    bg_fill: bg_1,
                    weak_bg_fill: bg_1,
                    bg_stroke: Stroke::new(1.0, fg_0),
                    fg_stroke: Stroke::new(1.0, fg_0),
                    corner_radius: CornerRadius::ZERO,
                    expansion: 0.0,
                },
                open: WidgetVisuals {
                    bg_fill: bg_1,
                    weak_bg_fill: bg_0,
                    bg_stroke: Stroke::new(1.0, bg_2),
                    fg_stroke: Stroke::new(1.0, fg_1),
                    corner_radius: CornerRadius::ZERO,
                    expansion: 0.0,
                },
            },
            ..Default::default()
        };

        DrawContext {
            colors,
            viewport_offset: V2::ZERO,
            egui_visuals: visuals,
        }
    }

    pub fn world_to_viewport(&self, world_coords: V2) -> V2 {
        world_coords + self.viewport_offset
    }

    pub fn viewport_to_world(&self, viewport_coords: V2) -> V2 {
        viewport_coords - self.viewport_offset
    }
}

pub struct App {
    session: Session,
    session_path: Option<PathBuf>,

    keybinds: HashMap<Input, Action>,

    cursor: CursorState,
    selected: Vec<DeviceId>,
    clipboard: Circuit,
    pending: Option<Circuit>,

    update_ctx: UpdateContext,
    draw_ctx: DrawContext,

    context_menu: Option<V2>,
}

impl App {
    pub fn new(colors: ColorPalette) -> Self {
        App {
            session: Session::new(),
            session_path: None,
            keybinds: default_keybinds(),
            cursor: CursorState::Idle,
            selected: Vec::new(),
            clipboard: Circuit::new(),
            pending: None,
            update_ctx: UpdateContext::new(),
            draw_ctx: DrawContext::new(colors),
            context_menu: None,
        }
    }

    pub fn handle_inputs(&mut self) {
        let (mx, my) = mouse_position();
        let m_pos = V2::new(mx, my);
        let device_under_mouse = self
            .session
            .circuit
            .get_device_at(self.draw_ctx.viewport_to_world(m_pos));

        match self.cursor {
            CursorState::Idle => {
                if is_mouse_button_pressed(MouseButton::Middle) {
                    self.cursor = CursorState::PanningViewport(m_pos);
                }

                match device_under_mouse {
                    Some(id) => {
                        if is_mouse_button_pressed(MouseButton::Left) {
                            if !self.selected.contains(&id) {
                                // if clicking a non-selected device, select it exclusively
                                self.selected = vec![id];
                            }
                            self.cursor = CursorState::DraggingSelectedDevices(m_pos);
                            // TODO snap here?
                        }

                        if is_mouse_button_pressed(MouseButton::Right) {
                            let dev = self.session.circuit.get_device(id).unwrap();
                            if dev.has_output() {
                                if is_key_down(KeyCode::LeftShift)
                                    || is_key_down(KeyCode::RightShift)
                                {
                                    self.cursor =
                                        CursorState::DraggingLooseWire(id, WireType::Negated);
                                } else {
                                    self.cursor =
                                        CursorState::DraggingLooseWire(id, WireType::Normal);
                                }
                            }
                        }
                    }
                    None => {
                        if is_mouse_button_pressed(MouseButton::Right) {
                            let wire_under_mouse = self
                                .session
                                .circuit
                                .get_wire_at(self.draw_ctx.viewport_to_world(m_pos));

                            match wire_under_mouse {
                                Some(edge) => {
                                    self.session.circuit.disconnect_devices(edge.from, edge.to);
                                    self.cursor =
                                        CursorState::DraggingLooseWire(edge.from, edge.wire_type);
                                }
                                None => {
                                    self.context_menu = Some(m_pos);
                                }
                            }
                        }
                        if is_mouse_button_pressed(MouseButton::Left) {
                            self.cursor = CursorState::DraggingSelectBox(m_pos);
                        }
                    }
                }
            }

            CursorState::DraggingSelectedDevices(from) => {
                self.move_selected_devices(m_pos - from);
                self.cursor = CursorState::DraggingSelectedDevices(m_pos);

                if is_mouse_button_released(MouseButton::Left) {
                    self.snap_selected_devices();
                    self.cursor = CursorState::Idle;
                }
            }

            CursorState::DraggingLooseWire(from_id, wire_type) => {
                if is_mouse_button_released(MouseButton::Right) {
                    self.cursor = CursorState::Idle;
                } else if let Some(to_id) = device_under_mouse {
                    if self.session.circuit.can_connect(from_id, to_id) {
                        self.cursor = CursorState::DraggingConnectedWire(from_id, to_id, wire_type);
                    } else {
                        self.cursor = CursorState::DraggingInvalidWire(from_id, wire_type);
                    }
                }
            }

            CursorState::DraggingConnectedWire(from_id, to_id, wire_type) => {
                if is_mouse_button_released(MouseButton::Right) {
                    self.session
                        .circuit
                        .connect_devices(from_id, to_id, wire_type);
                    self.cursor = CursorState::Idle;
                } else {
                    match device_under_mouse {
                        Some(to_id) => {
                            if !self.session.circuit.can_connect(from_id, to_id) {
                                self.cursor = CursorState::DraggingInvalidWire(from_id, wire_type);
                            }
                        }
                        None => self.cursor = CursorState::DraggingLooseWire(from_id, wire_type),
                    }
                }
            }

            CursorState::DraggingInvalidWire(from_id, wire_type) => {
                if is_mouse_button_released(MouseButton::Right) {
                    self.cursor = CursorState::Idle;
                } else {
                    match device_under_mouse {
                        Some(to_id) => {
                            if self.session.circuit.can_connect(from_id, to_id) {
                                self.cursor =
                                    CursorState::DraggingConnectedWire(from_id, to_id, wire_type);
                            }
                        }
                        None => self.cursor = CursorState::DraggingLooseWire(from_id, wire_type),
                    }
                }
            }

            CursorState::DraggingSelectBox(starting_corner) => {
                if is_mouse_button_released(MouseButton::Left) {
                    self.cursor = CursorState::Idle;
                } else {
                    let top = f32::min(starting_corner.y, m_pos.y);
                    let left = f32::min(starting_corner.x, m_pos.x);
                    let delta = (m_pos - starting_corner).abs();
                    let rect = Rect::new(left, top, delta.x, delta.y);

                    self.selected = self
                        .session
                        .circuit
                        .get_devices_in_rect(rect.offset(self.draw_ctx.viewport_offset * -1.0));
                }
            }

            CursorState::PanningViewport(from) => {
                self.draw_ctx.viewport_offset += m_pos - from;
                self.cursor = CursorState::PanningViewport(m_pos);

                if is_mouse_button_released(MouseButton::Middle) {
                    self.cursor = CursorState::Idle;
                }
            }

            CursorState::PendingDevices(from) => {
                if let Some(pending) = &self.pending {
                    if is_mouse_button_pressed(MouseButton::Left) {
                        self.selected = self.session.circuit.import_subcircuit(&pending);

                        self.pending = None;
                        self.cursor = CursorState::Idle;
                    } else if is_mouse_button_pressed(MouseButton::Right) {
                        self.pending = None;
                        self.cursor = CursorState::Idle;
                    } else {
                        self.move_pending_devices(m_pos - from);
                        self.cursor = CursorState::PendingDevices(m_pos);
                    }
                } else {
                    self.cursor = CursorState::Idle;
                }
            }
        }

        for action in process_inputs(&self.keybinds) {
            use Action::*;

            match action {
                LoadSession => self.open_session(),
                SaveSession => self.save_session(),
                NewSession => self.new_session(),
                CopySelected => self.copy_selected_devices(),
                PasteClipboard => self.set_pending(self.clipboard.clone()),
                DeleteSelected => self.delete_selected_devices(),
                TogglePause => self.toggle_pause(),

                CreateClock => self.set_pending(Device::Clock(Clock::new()).into()),
                CreateGate => self.set_pending(Device::Gate(Gate::new()).into()),
                CreateTrigger => self.set_pending(Device::Trigger(Trigger::new()).into()),
                CreateLatch => self.set_pending(Device::Latch(Latch::new()).into()),
                CreateNote => self.set_pending(Device::Note(Note::new()).into()),
            }
        }
    }

    pub fn update(&mut self) {
        self.update_ctx.this_update = Instant::now();

        if !self.update_ctx.is_paused {
            let time_elapsed = self.update_ctx.this_update - self.update_ctx.last_update;
            let beats_elapsed = time_elapsed.as_secs_f32() * (self.session.bpm as f32 / 60.0);

            self.update_ctx.free_clock += time_elapsed;
            self.update_ctx.beat_clock += beats_elapsed;
        }

        self.session.update(&mut self.update_ctx);
        self.update_ctx.last_update = self.update_ctx.this_update;

        self.update_ctx.midi_config.process_events();
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        ctx.set_visuals(self.draw_ctx.egui_visuals.clone());
        if let Some(pos) = self.context_menu {
            egui::Window::new("context menu")
                .resizable(false)
                .title_bar(false)
                .fixed_pos(pos.to_array())
                .show(ctx, |ui| {
                    let world_pos = self.draw_ctx.viewport_to_world(pos);
                    if ui.button("Clock").clicked() {
                        let clock = Clock::new();
                        self.session
                            .circuit
                            .add_device(Device::Clock(clock), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Trigger").clicked() {
                        let trigger = Trigger::new();
                        self.session
                            .circuit
                            .add_device(Device::Trigger(trigger), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Latch").clicked() {
                        let latch = Latch::new();
                        self.session
                            .circuit
                            .add_device(Device::Latch(latch), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Random").clicked() {
                        let random = Random::new();
                        self.session
                            .circuit
                            .add_device(Device::Random(random), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Gate").clicked() {
                        let gate = Gate::new();
                        self.session
                            .circuit
                            .add_device(Device::Gate(gate), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Note").clicked() {
                        let note = Note::new();
                        self.session
                            .circuit
                            .add_device(Device::Note(note), world_pos);
                        self.context_menu = None;
                    }
                });

            if is_key_pressed(KeyCode::Escape) {
                self.context_menu = None;
            }
        }

        egui::TopBottomPanel::top("top bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New session").clicked() {
                        self.new_session();
                    }
                    if ui.button("Open session").clicked() {
                        self.open_session();
                    }
                    if ui.button("Save session").clicked() {
                        self.save_session();
                    }
                });
                ui.menu_button("MIDI Setup", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Ports: ");
                        if ui.button("🔃").clicked() {
                            self.update_ctx.midi_config.refresh_ports();
                        }
                    });

                    for (name, port, connected) in self.update_ctx.midi_config.ports.clone() {
                        if ui
                            .add_enabled(!connected, egui::Button::new(name))
                            .clicked()
                        {
                            self.update_ctx.midi_config.connect_to_port(&port);
                        }
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("BPM");
                ui.add(egui::DragValue::new(&mut self.session.bpm).range(20..=777));

                ui.separator();

                ui.label(format!("Beat {:.1}", self.update_ctx.beat_clock));

                let pause_play_text = if self.update_ctx.is_paused {
                    "Play "
                } else {
                    "Pause"
                };
                if ui.button(pause_play_text).clicked() {
                    self.toggle_pause();
                }

                if ui.button("Reset").clicked() {
                    self.reset();
                }
            });
        });

        if let [selected_id] = self.selected.as_slice() {
            match self.session.circuit.get_device_mut(*selected_id) {
                Some(dev) => {
                    egui::Window::new("Edit Device")
                        .anchor(Align2::RIGHT_TOP, [-10.0, 30.0])
                        .movable(false)
                        .title_bar(false)
                        .default_width(INSPECTOR_WIDTH)
                        .resizable(false)
                        .show(ctx, |ui| dev.inspector(ui));
                }
                None => {
                    panic!("Tried to inspect device that doesn't exist???")
                }
            }
        }
    }

    pub fn draw(&self) {
        let (mx, my) = mouse_position();
        let m_pos = V2::new(mx, my);

        clear_background(self.draw_ctx.colors.bg_0);

        match self.cursor {
            CursorState::Idle
            | CursorState::DraggingSelectedDevices(_)
            | CursorState::PanningViewport(_) => {}

            CursorState::DraggingLooseWire(from_id, wire_type) => {
                let from_pos = self.session.circuit.device_position(from_id).unwrap();
                draw_wire_from_device(
                    &self.draw_ctx,
                    from_pos,
                    m_pos,
                    wire_type,
                    self.draw_ctx.colors.fg_2,
                );
            }
            CursorState::DraggingConnectedWire(from_id, to_id, wire_type) => {
                let from_pos = self.session.circuit.device_position(from_id).unwrap();
                let to_pos = self.session.circuit.device_position(to_id).unwrap();
                draw_wire_between_devices(
                    &self.draw_ctx,
                    from_pos,
                    to_pos,
                    wire_type,
                    self.draw_ctx.colors.fg_0,
                );
            }
            CursorState::DraggingInvalidWire(from_id, wire_type) => {
                let from_pos = self.session.circuit.device_position(from_id).unwrap();
                draw_wire_from_device(
                    &self.draw_ctx,
                    from_pos,
                    m_pos,
                    wire_type,
                    self.draw_ctx.colors.error,
                );
            }
            CursorState::DraggingSelectBox(starting_corner) => {
                let top = f32::min(starting_corner.y, m_pos.y);
                let left = f32::min(starting_corner.x, m_pos.x);
                let delta = (m_pos - starting_corner).abs();
                draw_rectangle_lines(left, top, delta.x, delta.y, 1.0, self.draw_ctx.colors.fg_2);
            }
            CursorState::PendingDevices(_) => {
                if let Some(pending) = &self.pending {
                    pending.draw(
                        &self.draw_ctx,
                        self.draw_ctx.colors.selected.with_alpha(0.5),
                        &[],
                        self.draw_ctx.colors.fg_1,
                    )
                }
            }
        }

        self.session.draw(&self.draw_ctx, &self.selected);
    }
}

impl App {
    fn toggle_pause(&mut self) {
        self.update_ctx.is_paused = !self.update_ctx.is_paused;
    }

    fn reset(&mut self) {
        self.update_ctx.beat_clock = 0.0;
        self.update_ctx.free_clock = Duration::ZERO;
        self.update_ctx.last_update = Instant::now();

        self.session.circuit.reset_devices();
    }

    fn set_pending(&mut self, c: Circuit) {
        let (mx, my) = mouse_position();
        let m_pos = V2::new(mx, my);
        let world_pos = self.draw_ctx.viewport_to_world(m_pos);

        self.pending = Some(c);
        self.move_pending_devices(world_pos);
        self.selected.clear();
        self.cursor = CursorState::PendingDevices(m_pos);
    }

    fn move_selected_devices(&mut self, delta: V2) {
        for dev_id in self.selected.iter() {
            self.session.circuit.move_device(*dev_id, delta);
        }
    }

    fn move_pending_devices(&mut self, delta: V2) {
        if let Some(ref mut pending) = self.pending {
            pending.move_all_devices(delta);
        }
    }

    fn snap_selected_devices(&mut self) {
        for dev_id in self.selected.iter() {
            self.session.circuit.snap_device(*dev_id);
        }
    }

    fn delete_selected_devices(&mut self) {
        for dev_id in &self.selected {
            self.session.circuit.delete_device(*dev_id);
        }

        self.selected.clear();
    }

    fn copy_selected_devices(&mut self) {
        self.clipboard = self.session.circuit.clone_subcircuit(&self.selected);
    }

    fn write_session_to_file(&self, path: &Path) {
        let serialized = serde_json::to_string(&self.session).unwrap();
        if let Err(e) = fs::write(path, serialized) {
            println!("{}", e);
        }
    }

    fn read_session_from_file(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.session = serde_json::from_reader(reader)?;
        Ok(())
    }

    fn save_session(&mut self) {
        match &self.session_path {
            None => {
                let dialog = FileDialog::new().set_file_name("Untitled.graf");
                if let Some(save_path) = dialog.save_file() {
                    self.write_session_to_file(&save_path);
                    self.session_path = Some(save_path);
                }
            }
            Some(save_path) => self.write_session_to_file(&save_path),
        }
    }

    fn open_session(&mut self) {
        let dialog = FileDialog::new();
        if let Some(load_path) = dialog.pick_file() {
            match self.read_session_from_file(&load_path) {
                Ok(()) => self.session_path = Some(load_path),
                Err(e) => eprintln!("Failed to load session from file: {}", e),
            }
        }
    }

    fn new_session(&mut self) {
        // TODO make user confirm if there are unsaved changes
        self.selected.clear();
        self.session = Session::new();
        self.session_path = None;
        self.reset();
    }
}
