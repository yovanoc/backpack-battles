use crate::{Cell, ItemKind, Offset, Rotation, Shape, geometry};

mod state;
pub(crate) use state::ItemState;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ItemId(Cell);

impl ItemId {
    pub const fn cell(self) -> Cell {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemRef {
    pub side: crate::Side,
    pub id: ItemId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    id: ItemId,
    position: Cell,
    state: ItemState,
    shape: Shape,
    charge_remaining: Option<u32>,
    charge_period: Option<u32>,
    speed_basis_points: u16,
}

impl Item {
    pub fn new(kind: ItemKind, position: Cell) -> Self {
        Self::with_rotation(kind, position, Rotation::Deg0)
    }

    pub fn with_rotation(kind: ItemKind, position: Cell, rotation: Rotation) -> Self {
        let shape = geometry::rotated(kind.shape(), rotation);
        let charge_remaining = kind
            .activation()
            .map(|timing| u32::from(timing.initial) * 10_000);
        Self {
            id: ItemId(anchor_cell(position, shape.as_slice())),
            position,
            state: ItemState::new(kind),
            shape,
            charge_remaining,
            charge_period: charge_remaining,
            speed_basis_points: 0,
        }
    }

    pub const fn id(&self) -> ItemId {
        self.id
    }

    pub const fn kind(&self) -> ItemKind {
        self.state.kind()
    }

    pub const fn position(&self) -> Cell {
        self.position
    }

    pub fn shape(&self) -> &[Offset] {
        self.shape.as_slice()
    }

    pub const fn speed_basis_points(&self) -> u16 {
        self.speed_basis_points
    }

    pub(crate) const fn is_ready(&self) -> bool {
        matches!(self.charge_remaining, Some(0))
    }

    /// Progress toward the next activation, 0.0 (just fired) to 1.0 (ready).
    /// `None` for passive items that never activate.
    pub fn charge_progress(&self) -> Option<f64> {
        let remaining = self.charge_remaining?;
        let period = self.charge_period?;
        if period == 0 {
            return Some(1.0);
        }
        Some(1.0 - f64::from(remaining) / f64::from(period))
    }

    pub(crate) fn advance_charge(&mut self) {
        let Some(charge) = &mut self.charge_remaining else {
            return;
        };
        let speed = 10_000_u32 + u32::from(self.speed_basis_points);
        *charge = charge.saturating_sub(speed);
    }

    pub(crate) fn schedule_next(&mut self) {
        let Some(timing) = self.kind().activation() else {
            self.charge_remaining = None;
            return;
        };
        if !self.state.can_activate_again() {
            self.charge_remaining = None;
            return;
        }
        self.charge_remaining = Some(u32::from(timing.recurring) * 10_000);
        self.charge_period = self.charge_remaining;
    }

    pub(crate) fn add_speed(&mut self, basis_points: u16) -> u16 {
        // ponytail: speed caps at u16::MAX bps; no bag stacks near that ceiling.
        self.speed_basis_points = self.speed_basis_points.saturating_add(basis_points);
        self.speed_basis_points
    }

    pub(crate) fn state_mut(&mut self) -> &mut ItemState {
        &mut self.state
    }
}

/// An item's stable identity is its minimum occupied cell. That cell is always
/// occupied, so two non-overlapping items can never share it - which keeps IDs
/// unique and stable under any rotation.
fn anchor_cell(position: Cell, shape: &[Offset]) -> Cell {
    shape
        .iter()
        .map(|offset| {
            Cell::new(
                position.x.saturating_add(offset.x),
                position.y.saturating_add(offset.y),
            )
        })
        .min()
        .unwrap_or(position)
}
