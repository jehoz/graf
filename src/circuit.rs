use core::clone::Clone;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    app::{DrawContext, UpdateContext},
    dag::{self, Dag, DeviceId, Wire, WireType},
    devices::{Arity, Device},
    drawing_utils::{draw_wire_between_devices, DEVICE_WIDTH, SNAP_GRID_SIZE},
    math::{Rect, V2},
};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DevicePosition {
    raw: V2,
    snapped: V2,
}

impl DevicePosition {
    pub fn new() -> Self {
        DevicePosition {
            raw: V2::ZERO,
            snapped: V2::ZERO,
        }
    }

    pub fn modify(&mut self, delta: V2) {
        self.raw += delta;
        self.snapped = (self.raw / SNAP_GRID_SIZE).round() * SNAP_GRID_SIZE;
    }

    pub fn snap(&mut self) {
        self.raw = self.snapped;
    }
}

impl From<V2> for DevicePosition {
    fn from(value: V2) -> Self {
        let mut p = DevicePosition::new();
        p.modify(value);
        p
    }
}

#[derive(Serialize, Deserialize)]
pub struct Circuit {
    pub devices: HashMap<DeviceId, (DevicePosition, Device)>,
    pub dag: Dag,
}

impl Circuit {
    pub fn new() -> Self {
        Circuit {
            devices: HashMap::new(),
            dag: Dag::new(),
        }
    }

    pub fn add_device(&mut self, device: Device, position: V2) -> DeviceId {
        let id = self.dag.add_device();
        let pos = DevicePosition::from(position);
        self.devices.insert(id, (pos, device));
        id
    }

    pub fn delete_device(&mut self, device_id: DeviceId) {
        self.dag.remove_device(device_id);
        self.devices.remove(&device_id);
    }

    pub fn move_device(&mut self, device_id: DeviceId, delta: V2) {
        self.devices
            .get_mut(&device_id)
            .map(|(p, _)| p.modify(delta));
    }

    pub fn snap_device(&mut self, device_id: DeviceId) {
        self.devices.get_mut(&device_id).map(|(p, _)| p.snap());
    }

    pub fn connect_devices(&mut self, from: DeviceId, to: DeviceId, wire_type: WireType) {
        // just silently ignore any errors for now
        if let Err(dag::IllegalWireError) = self.dag.add_wire(from, to, wire_type) {
            print!("Got IllegalEdgeError when trying to connected devices!!!");
        }
    }

    pub fn disconnect_devices(&mut self, from: DeviceId, to: DeviceId) {
        self.dag.remove_wire(from, to)
    }

    /// Copies the contents of another circuit into this one
    pub fn import_subcircuit(&mut self, subcircuit: &Circuit, position: V2) -> Vec<DeviceId> {
        let mut translation = HashMap::new();
        for (id, (pos, device)) in subcircuit.devices.iter() {
            let new_id = self.add_device(device.clone(), pos.raw + position);
            translation.insert(id, new_id);
        }

        for wire in subcircuit.dag.wires() {
            let from = translation.get(&wire.from).unwrap();
            let to = translation.get(&wire.to).unwrap();
            self.connect_devices(*from, *to, wire.wire_type);
        }

        translation.values().map(|id| *id).collect()
    }

    pub fn get_device(&self, device_id: DeviceId) -> Option<&Device> {
        self.devices.get(&device_id).map(|(_, dev)| dev)
    }

    pub fn get_device_mut(&mut self, device_id: DeviceId) -> Option<&mut Device> {
        self.devices.get_mut(&device_id).map(|(_, dev)| dev)
    }

    pub fn get_device_at(&self, position: V2) -> Option<DeviceId> {
        // if inside multiple devices' bounding boxes, get closest one
        let mut min_dist = f32::INFINITY;
        let mut closest = None;
        for (id, (pos, _)) in self.devices.iter() {
            let dx = (pos.snapped.x - position.x).abs();
            let dy = (pos.snapped.y - position.y).abs();
            if dx <= DEVICE_WIDTH && dy <= DEVICE_WIDTH {
                let d = position.distance(pos.snapped);
                if d < min_dist {
                    min_dist = d;
                    closest = Some(*id);
                }
            }
        }

        closest
    }

    pub fn get_devices_in_rect(&self, rect: Rect) -> Vec<DeviceId> {
        let mut v = Vec::new();
        for (id, (pos, _)) in self.devices.iter() {
            if rect.contains(pos.snapped) {
                v.push(*id)
            }
        }
        v
    }

    pub fn get_wire_at(&self, position: V2) -> Option<Wire> {
        const WIRE_CLICKABLE_DISTANCE: f32 = 5.0;

        for wire in self.dag.wires() {
            let u = self.device_position(wire.from).unwrap();
            let v = self.device_position(wire.to).unwrap();

            let len2 = u.distance_squared(v);

            if len2 == 0.0 {
                return None;
            }

            let t = ((position - u).dot(v - u) / len2).clamp(0.0, 1.0);
            let point_on_line = u + t * (v - u);

            if position.distance(point_on_line) < WIRE_CLICKABLE_DISTANCE {
                return Some(*wire);
            }
        }

        None
    }

    /// Given a list of `DeviceId`s, creates a new `Circuit` with copies of those devices,
    /// maintaining their relative positions and any wires between them.
    pub fn clone_subcircuit(&self, devices: &Vec<DeviceId>) -> Circuit {
        let mut subcircuit = Circuit::new();
        // translation table from old device id to new device id
        let mut translation = HashMap::new();
        let mut top_left = V2::new(f32::INFINITY, f32::INFINITY);
        for dev_id in devices.iter() {
            if let Some((pos, device)) = self.devices.get(&dev_id) {
                if pos.snapped.x < top_left.x {
                    top_left.x = pos.snapped.x;
                }
                if pos.snapped.y < top_left.y {
                    top_left.y = pos.snapped.y;
                }
                let new_id = subcircuit.add_device(device.clone(), pos.snapped);
                translation.insert(dev_id, new_id);
            }
        }

        // set device positions to be relative to bounding box top-left corner
        for dev_id in translation.values() {
            subcircuit.move_device(*dev_id, -top_left);
            subcircuit.snap_device(*dev_id);
        }

        for wire in self.dag.wires() {
            if devices.contains(&wire.from) && devices.contains(&wire.to) {
                let new_from = translation.get(&wire.from).unwrap();
                let new_to = translation.get(&wire.to).unwrap();

                subcircuit.connect_devices(*new_from, *new_to, wire.wire_type);
            }
        }

        subcircuit
    }

    pub fn can_connect(&self, from: DeviceId, to: DeviceId) -> bool {
        if !self.devices.contains_key(&from) || !self.devices.contains_key(&to) || from == to {
            return false;
        }

        let (_, to_dev) = self.devices.get(&to).unwrap();
        if to_dev.input_arity() == Arity::Nullary {
            return false;
        } else if to_dev.input_arity() == Arity::Unary && self.dag.incoming(to).count() > 0 {
            return false;
        }

        !self.dag.is_reachable(to, from)
    }

    pub fn device_position(&self, id: DeviceId) -> Option<V2> {
        self.devices.get(&id).map(|(p, _)| p.snapped)
    }

    pub fn reset_devices(&mut self) {
        for (_, dev) in self.devices.values_mut() {
            dev.reset();
        }
    }

    pub fn update(&mut self, ctx: &mut UpdateContext) {
        let mut device_outputs: HashMap<DeviceId, bool> = HashMap::new();
        for dev_id in self.dag.devices() {
            let inputs: Vec<bool> = self
                .dag
                .incoming(*dev_id)
                .filter_map(|wire| match wire.wire_type {
                    WireType::Normal => device_outputs.get(&wire.from).copied(),
                    WireType::Negated => device_outputs.get(&wire.from).copied().map(|x| !x),
                })
                .collect();

            let (_, dev) = self.devices.get_mut(dev_id).unwrap();
            if let Some(output) = dev.update(ctx, inputs) {
                device_outputs.insert(*dev_id, output);
            }
        }
    }

    pub fn draw(&self, draw_ctx: &DrawContext, selected: &Vec<DeviceId>) {
        for wire in self.dag.wires() {
            let (from_pos, _) = self.devices.get(&wire.from).unwrap();
            let (to_pos, _) = self.devices.get(&wire.to).unwrap();

            let color = if selected.contains(&wire.from) && selected.contains(&wire.to) {
                draw_ctx.colors.selected
            } else {
                draw_ctx.colors.fg_1
            };

            draw_wire_between_devices(
                draw_ctx,
                from_pos.snapped,
                to_pos.snapped,
                wire.wire_type,
                color,
            );
        }

        for (id, (pos, device)) in &self.devices {
            let color = if selected.contains(id) {
                draw_ctx.colors.selected
            } else {
                draw_ctx.colors.fg_1
            };

            device.draw(
                draw_ctx,
                draw_ctx.world_to_viewport(pos.snapped),
                24.0,
                color,
            );
        }
    }
}
