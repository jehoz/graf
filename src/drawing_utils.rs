use egui::Color32;
use macroquad::{
    color::Color,
    math::vec2,
    shapes::{draw_line, draw_poly},
};

use crate::{app::DrawContext, dag::WireType, math::V2};

pub const DEVICE_WIDTH: f32 = 24.0;

pub const SNAP_GRID_SIZE: f32 = 16.0;

pub struct ColorPalette {
    pub fg_0: Color,
    pub fg_1: Color,
    pub fg_2: Color,
    pub fg_3: Color,
    pub bg_0: Color,
    pub bg_1: Color,
    pub bg_2: Color,
    pub bg_3: Color,
    pub selected: Color,
    pub error: Color,
}

pub fn color_to_color32(c: Color) -> Color32 {
    let [r, g, b, _a] = c.into();
    Color32::from_rgb(r, g, b)
}

pub fn draw_line_v(from: V2, to: V2, thickness: f32, color: Color) {
    draw_line(from.x, from.y, to.x, to.y, thickness, color)
}

pub fn draw_arrow(
    from: V2,
    to: V2,
    thickness: f32,
    head_size: f32,
    fill: Color,
    outline: Option<Color>,
) {
    let rotation = vec2(1.0, 0.0)
        .angle_between((to - from).into())
        .to_degrees();
    let arrow_pos = to - (to - from).normalize() * head_size;

    if let Some(color) = outline {
        draw_line_v(from, arrow_pos, thickness + 1.5, color);
        draw_poly(
            arrow_pos.x,
            arrow_pos.y,
            3,
            head_size + 2.0,
            rotation,
            color,
        );
    }

    draw_line_v(from, arrow_pos, thickness, fill);
    draw_poly(arrow_pos.x, arrow_pos.y, 3, head_size, rotation, fill);
}

pub fn draw_wire(from: V2, to: V2, wire_type: WireType, color: Color) {
    match wire_type {
        WireType::Normal => draw_arrow(from, to, 1.5, 6.0, color, None),
        WireType::Negated => draw_arrow(from, to, 1.5, 5.0, macroquad::color::BLACK, Some(color)),
    }
}

// computes the point on a given circle's perimeter that is closest to another point
pub fn closest_point_on_circle(center: V2, radius: f32, point: V2) -> V2 {
    let delta = point - center;
    center + delta.normalize() * radius
}

pub fn draw_wire_from_device(
    draw_ctx: &DrawContext,
    device_position: V2,
    to: V2,
    wire_type: WireType,
    color: Color,
) {
    let from_pos = closest_point_on_circle(
        device_position,
        DEVICE_WIDTH / 2.0 + 3.0,
        draw_ctx.viewport_to_world(to),
    );
    draw_wire(draw_ctx.world_to_viewport(from_pos), to, wire_type, color);
}

pub fn draw_wire_between_devices(
    draw_ctx: &DrawContext,
    from_device: V2,
    to_device: V2,
    wire_type: WireType,
    color: Color,
) {
    let from_pos = closest_point_on_circle(from_device, DEVICE_WIDTH / 2.0 + 3.0, to_device);
    let to_pos = closest_point_on_circle(to_device, DEVICE_WIDTH / 2.0 + 3.0, from_device);
    draw_wire(
        draw_ctx.world_to_viewport(from_pos),
        draw_ctx.world_to_viewport(to_pos),
        wire_type,
        color,
    );
}
