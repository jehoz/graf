use macroquad::math::Vec2;
use serde::Serialize;

// custom 2D vector type that is serializable and implicitly convertible to and from Vec2
#[derive(PartialEq, Copy, Clone, Serialize)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2> for V2 {
    fn from(vec2: Vec2) -> V2 {
        V2 {
            x: vec2.x,
            y: vec2.y,
        }
    }
}

impl Into<Vec2> for V2 {
    fn into(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}
