use crate::Offset;

pub(crate) const ONE: [Offset; 1] = [Offset { x: 0, y: 0 }];
// A domino: rotation (Rotation::ALL) covers both orientations, so there is no
// separate vertical/horizontal 2-cell shape.
pub(crate) const DOMINO: [Offset; 2] = [Offset { x: 0, y: 0 }, Offset { x: 1, y: 0 }];
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
pub(crate) const T_TETROMINO: [Offset; 4] = [
    Offset { x: 0, y: 0 },
    Offset { x: 1, y: 0 },
    Offset { x: 2, y: 0 },
    Offset { x: 1, y: 1 },
];
