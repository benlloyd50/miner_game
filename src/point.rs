#[derive(Debug, Copy, Clone)]
pub struct UPoint {
    pub x: usize,
    pub y: usize,
}

impl UPoint {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub(crate) fn as_idx(&self, width: usize) -> usize {
        xy_to_idx(self.x, self.y, width)
    }
}

pub fn idx_to_xy(idx: usize, width: usize) -> UPoint {
    UPoint { x: idx / width, y: idx % width }
}

pub fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}

pub struct _IPoint {
    pub x: i32,
    pub y: i32,
}
