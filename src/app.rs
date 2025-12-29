use core::panic;

use egui::{menu, style::WidgetVisuals, style::Widgets, Align2, CornerRadius, Stroke, Visuals};
use macroquad::{
    input::{
        is_key_down, is_key_pressed, is_mouse_button_pressed, is_mouse_button_released,
        mouse_position, KeyCode, MouseButton,
    },
    math::{vec2, Rect, Vec2},
    shapes::draw_rectangle_lines,
    window::clear_background,
};

use crate::{
    dag::{DeviceId, WireType},
    devices::{clock::Clock, gate::Gate, latch::Latch, note::Note, trigger::Trigger},
    drawing_utils::{
        color_to_color32, draw_wire_between_devices, draw_wire_from_device, ColorPalette,
    },
    midi::MidiConfig,
    session::Session,
};

enum CursorState {
    Idle,
    DraggingSelectedDevices(Vec2),
    DraggingLooseWire(DeviceId, WireType),
    DraggingConnectedWire(DeviceId, DeviceId, WireType),
    DraggingInvalidWire(DeviceId, WireType),
    DraggingSelectBox(Vec2),
    PanningViewport(Vec2),
}

const INSPECTOR_WIDTH: f32 = 200.0;

pub struct DrawContext {
    pub colors: ColorPalette,
    pub viewport_offset: Vec2,
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
            viewport_offset: Vec2::ZERO,
            egui_visuals: visuals,
        }
    }

    pub fn world_to_viewport(&self, world_coords: Vec2) -> Vec2 {
        world_coords + self.viewport_offset
    }

    pub fn viewport_to_world(&self, viewport_coords: Vec2) -> Vec2 {
        viewport_coords - self.viewport_offset
    }
}

pub struct App {
    session: Session,
    cursor: CursorState,
    draw_ctx: DrawContext,

    context_menu: Option<Vec2>,

    midi_config: MidiConfig,
}

impl App {
    pub fn new(colors: ColorPalette) -> Self {
        App {
            session: Session::new(),
            cursor: CursorState::Idle,
            draw_ctx: DrawContext::new(colors),
            context_menu: None,
            midi_config: MidiConfig::new(),
        }
    }

    pub fn handle_inputs(&mut self) {
        let (mx, my) = mouse_position();
        let m_pos = vec2(mx, my);
        let device_under_mouse = self
            .session
            .get_device_at(self.draw_ctx.viewport_to_world(m_pos));

        match self.cursor {
            CursorState::Idle => {
                if is_mouse_button_pressed(MouseButton::Middle) {
                    self.cursor = CursorState::PanningViewport(m_pos);
                }

                match device_under_mouse {
                    Some(id) => {
                        if is_mouse_button_pressed(MouseButton::Left) {
                            if !self.session.selected.contains(&id) {
                                self.session.clear_selection();
                                self.session.select_device(id);
                            }
                            self.cursor = CursorState::DraggingSelectedDevices(m_pos);
                        }

                        if is_mouse_button_pressed(MouseButton::Right) {
                            let dev = self.session.get_device(id).unwrap();
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
                                .get_wire_at(self.draw_ctx.viewport_to_world(m_pos));

                            match wire_under_mouse {
                                Some(edge) => {
                                    self.session.disconnect_devices(edge.from, edge.to);
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
                self.session.move_selected_devices(m_pos - from);
                self.cursor = CursorState::DraggingSelectedDevices(m_pos);

                if is_mouse_button_released(MouseButton::Left) {
                    self.session.snap_selected_devices();
                    self.cursor = CursorState::Idle;
                }
            }

            CursorState::DraggingLooseWire(from_id, wire_type) => {
                if is_mouse_button_released(MouseButton::Right) {
                    self.cursor = CursorState::Idle;
                } else if let Some(to_id) = device_under_mouse {
                    if self.session.can_connect(from_id, to_id) {
                        self.cursor = CursorState::DraggingConnectedWire(from_id, to_id, wire_type);
                    } else {
                        self.cursor = CursorState::DraggingInvalidWire(from_id, wire_type);
                    }
                }
            }

            CursorState::DraggingConnectedWire(from_id, to_id, wire_type) => {
                if is_mouse_button_released(MouseButton::Right) {
                    self.session.connect_devices(from_id, to_id, wire_type);
                    self.cursor = CursorState::Idle;
                } else {
                    match device_under_mouse {
                        Some(to_id) => {
                            if !self.session.can_connect(from_id, to_id) {
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
                            if self.session.can_connect(from_id, to_id) {
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

                    self.session.clear_selection();
                    self.session
                        .select_devices_in_rect(rect.offset(self.draw_ctx.viewport_offset * -1.0));
                }
            }

            CursorState::PanningViewport(from) => {
                self.draw_ctx.viewport_offset += m_pos - from;
                self.cursor = CursorState::PanningViewport(m_pos);

                if is_mouse_button_released(MouseButton::Middle) {
                    self.cursor = CursorState::Idle;
                }
            }
        }

        if is_key_pressed(KeyCode::Delete) {
            self.session.delete_selected_devices();
        }

        if is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl) {
            if is_key_pressed(KeyCode::C) {
                self.session.copy_selected_devices();
            }

            if is_key_pressed(KeyCode::V) {
                self.session
                    .paste_clipboard(self.draw_ctx.viewport_to_world(m_pos));
            }
        }

        if is_key_pressed(KeyCode::Space) {
            self.session.toggle_pause();
        }
    }

    pub fn update(&mut self) {
        self.session.update();

        self.midi_config.process_events();
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
                        let clock = Clock::new(self.draw_ctx.viewport_to_world(pos));
                        self.session.add_device(Box::new(clock), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Trigger").clicked() {
                        let trigger = Trigger::new(self.draw_ctx.viewport_to_world(pos));
                        self.session.add_device(Box::new(trigger), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Latch").clicked() {
                        let latch = Latch::new(self.draw_ctx.viewport_to_world(pos));
                        self.session.add_device(Box::new(latch), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Gate").clicked() {
                        let gate = Gate::new(self.draw_ctx.viewport_to_world(pos));
                        self.session.add_device(Box::new(gate), world_pos);
                        self.context_menu = None;
                    }
                    if ui.button("Note").clicked() {
                        let note = Note::new(
                            self.draw_ctx.viewport_to_world(pos),
                            self.midi_config.get_event_sender(),
                        );
                        self.session.add_device(Box::new(note), world_pos);
                        self.context_menu = None;
                    }
                });

            if is_key_pressed(KeyCode::Escape) {
                self.context_menu = None;
            }
        }

        egui::TopBottomPanel::top("top bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("MIDI Setup", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Ports: ");
                        if ui.button("🔃").clicked() {
                            self.midi_config.refresh_ports();
                        }
                    });

                    for (name, port, connected) in self.midi_config.ports.clone() {
                        if ui
                            .add_enabled(!connected, egui::Button::new(name))
                            .clicked()
                        {
                            self.midi_config.connect_to_port(&port);
                        }
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("BPM");
                ui.add(egui::DragValue::new(&mut self.session.update_ctx.bpm).range(20..=777));

                ui.separator();

                ui.label(format!("Beat {:.1}", self.session.update_ctx.beat_clock));

                let pause_play_text = if self.session.update_ctx.is_paused {
                    "Play "
                } else {
                    "Pause"
                };
                if ui.button(pause_play_text).clicked() {
                    self.session.toggle_pause();
                }

                if ui.button("Reset").clicked() {
                    self.session.reset();
                }
            });
        });

        if let [selected_id] = self.session.selected.as_slice() {
            match self.session.get_device_mut(*selected_id) {
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
        let m_pos = vec2(mx, my);

        clear_background(self.draw_ctx.colors.bg_0);

        match self.cursor {
            CursorState::Idle
            | CursorState::DraggingSelectedDevices(_)
            | CursorState::PanningViewport(_) => {}

            CursorState::DraggingLooseWire(from_id, wire_type) => {
                let from_dev = self.session.get_device(from_id).unwrap();
                draw_wire_from_device(
                    &self.draw_ctx,
                    from_dev.as_ref(),
                    m_pos,
                    wire_type,
                    self.draw_ctx.colors.fg_2,
                );
            }
            CursorState::DraggingConnectedWire(from_id, to_id, wire_type) => {
                let from_dev = self.session.get_device(from_id).unwrap();
                let to_dev = self.session.get_device(to_id).unwrap();
                draw_wire_between_devices(
                    &self.draw_ctx,
                    from_dev.as_ref(),
                    to_dev.as_ref(),
                    wire_type,
                    self.draw_ctx.colors.fg_0,
                );
            }
            CursorState::DraggingInvalidWire(from_id, wire_type) => {
                let from_dev = self.session.get_device(from_id).unwrap();
                draw_wire_from_device(
                    &self.draw_ctx,
                    from_dev.as_ref(),
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
        }

        self.session.draw(&self.draw_ctx);
    }
}
