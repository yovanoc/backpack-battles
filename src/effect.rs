use crate::{DamageMode, FallCause, ItemRef, Side};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Effect {
    pub source: ItemRef,
    pub kind: EffectKind,
}

impl Effect {
    pub const fn new(source: ItemRef, kind: EffectKind) -> Self {
        Self { source, kind }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EffectKind {
    Damage {
        target: Side,
        amount: u16,
        mode: DamageMode,
    },
    LoseHealth {
        target: Side,
        amount: u16,
    },
    Heal {
        target: Side,
        amount: u16,
    },
    GainBlock {
        target: Side,
        amount: u16,
        maximum: u16,
    },
    ModifySpeed {
        target: ItemTarget,
        basis_points: u16,
    },
    AttemptFall {
        target: ItemTarget,
        cause: FallCause,
        chance: Option<u64>,
    },
    Consume {
        target: ItemRef,
    },
    ApplyPoison {
        target: Side,
        stacks: u16,
    },
    CleansePoison {
        target: Side,
        amount: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ItemTarget {
    Adjacent(ItemRef),
    LightestEdge { side: Side, weapons_only: bool },
}
