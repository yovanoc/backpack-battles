use crate::Offset;

pub(crate) const ONE: [Offset; 1] = [Offset { x: 0, y: 0 }];
pub(crate) const VERTICAL_TWO: [Offset; 2] = [Offset { x: 0, y: 0 }, Offset { x: 0, y: 1 }];
pub(crate) const HORIZONTAL_TWO: [Offset; 2] = [Offset { x: 0, y: 0 }, Offset { x: 1, y: 0 }];
pub(crate) const SQUARE: [Offset; 4] = [
    Offset { x: 0, y: 0 },
    Offset { x: 1, y: 0 },
    Offset { x: 0, y: 1 },
    Offset { x: 1, y: 1 },
];
pub(crate) const LINE_THREE: [Offset; 3] = [
    Offset { x: 0, y: 0 },
    Offset { x: 1, y: 0 },
    Offset { x: 2, y: 0 },
];
pub(crate) const L_TROMINO: [Offset; 3] = [
    Offset { x: 0, y: 0 },
    Offset { x: 0, y: 1 },
    Offset { x: 1, y: 1 },
];
pub(crate) const L_TETROMINO: [Offset; 4] = [
    Offset { x: 0, y: 0 },
    Offset { x: 0, y: 1 },
    Offset { x: 0, y: 2 },
    Offset { x: 1, y: 2 },
];
