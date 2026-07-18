pub const BAG_WIDTH: u8 = 5;
pub const BAG_HEIGHT: u8 = 4;
pub(crate) const BAG_CELLS: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cell {
    pub x: u8,
    pub y: u8,
}

impl Cell {
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offset {
    pub x: u8,
    pub y: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

impl Rotation {
    pub const ALL: [Self; 4] = [Self::Deg0, Self::Deg90, Self::Deg180, Self::Deg270];
}

/// A placed item's occupied cells, stored inline (max 4) so building an item
/// never touches the heap - which matters across millions of generated bags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    cells: [Offset; 4],
    len: u8,
}

impl Shape {
    pub fn as_slice(&self) -> &[Offset] {
        &self.cells[..self.len as usize]
    }
}

/// Rotate a shape around the origin and normalize it into the non-negative
/// quadrant, so the top-left of its bounding box sits at (0, 0).
pub fn rotated(base: &[Offset], rotation: Rotation) -> Shape {
    let len = base.len().min(4);
    let mut points = [(0i16, 0i16); 4];
    for (slot, offset) in points[..len].iter_mut().zip(base) {
        let (x, y) = (i16::from(offset.x), i16::from(offset.y));
        *slot = match rotation {
            Rotation::Deg0 => (x, y),
            Rotation::Deg90 => (-y, x),
            Rotation::Deg180 => (-x, -y),
            Rotation::Deg270 => (y, -x),
        };
    }
    let min_x = points[..len].iter().map(|point| point.0).min().unwrap_or(0);
    let min_y = points[..len].iter().map(|point| point.1).min().unwrap_or(0);
    let mut cells = [Offset { x: 0, y: 0 }; 4];
    for (cell, point) in cells[..len].iter_mut().zip(&points[..len]) {
        *cell = Offset {
            x: u8::try_from(point.0 - min_x).unwrap_or(0),
            y: u8::try_from(point.1 - min_y).unwrap_or(0),
        };
    }
    Shape {
        cells,
        len: u8::try_from(len).unwrap_or(0),
    }
}
