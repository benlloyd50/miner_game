#[derive(Debug, Copy, Clone)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

pub fn idx_to_xy(idx: usize, width: usize) -> Point {
    Point { x: idx / width, y: idx % width }
}

pub fn xy_to_idx(x: usize, y: usize, width: usize) -> usize {
    x + y * width
}
