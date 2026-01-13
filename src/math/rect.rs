use crate::math::V2;

/// 2D rectangle defined by top-right corner, width, and height
pub struct Rect {
    pub x: f32,
    pub y: f32,

    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn size(&self) -> V2 {
        V2::new(self.w, self.h)
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    pub fn top_left(&self) -> V2 {
        V2::new(self.x, self.y)
    }

    pub fn top_right(&self) -> V2 {
        V2::new(self.right(), self.y)
    }

    pub fn bottom_left(&self) -> V2 {
        V2::new(self.x, self.bottom())
    }

    /// Checks if a given point is inside of the `Rect`
    pub fn contains(&self, point: V2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y >= self.top()
            && point.y < self.bottom()
    }

    pub fn offset(self, offset: V2) -> Self {
        Rect {
            x: self.x + offset.x,
            y: self.y + offset.y,
            w: self.w,
            h: self.h,
        }
    }
}
