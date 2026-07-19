use std::{error::Error, fmt, time::Duration};

use crate::{Bag, ItemKind, ItemRef};

pub const MAX_TICKS: u16 = 600;
pub const TICK_DURATION: Duration = Duration::from_millis(100);

/// Opening block reserve the defending (right) hero starts each battle with,
/// calibrated to offset the attacker's even-rank timing edge (design §13).
/// Guard 2 minimizes the worst-case paired side bias across Random, Pure, and
/// Hybrid mirrored campaigns after retaliation was made raw (which strengthened
/// defenders and lowered the guard the attacker edge needs to offset).
pub const DEFENDER_GUARD: u16 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hero {
    name: String,
    base_max_health: u16,
    pub(crate) health: u16,
    pub(crate) block: u16,
    pub(crate) bag: Bag,
    pub(crate) poison: u16,
}

impl Hero {
    pub fn new(name: impl Into<String>, base_max_health: u16, bag: Bag) -> Self {
        let health = base_max_health.saturating_add(bag.max_health_bonus());
        Self {
            name: name.into(),
            base_max_health,
            health,
            block: 0,
            bag,
            poison: 0,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn health(&self) -> u16 {
        self.health
    }

    pub const fn block(&self) -> u16 {
        self.block
    }

    pub fn max_health(&self) -> u16 {
        self.base_max_health
            .saturating_add(self.bag.max_health_bonus())
    }

    pub fn armor(&self) -> u16 {
        self.bag.armor()
    }

    pub const fn bag(&self) -> &Bag {
        &self.bag
    }

    pub(crate) fn clamp_health_to_max(&mut self) {
        self.health = self.health.min(self.max_health());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BattleConfig {
    tick_limit: u16,
    seed: u64,
}

impl BattleConfig {
    pub fn new(tick_limit: u16, seed: u64) -> Result<Self, ConfigError> {
        if !(1..=MAX_TICKS).contains(&tick_limit) {
            return Err(ConfigError::InvalidTickLimit(tick_limit));
        }
        Ok(Self { tick_limit, seed })
    }

    pub const fn tick_limit(self) -> u16 {
        self.tick_limit
    }

    pub const fn seed(self) -> u64 {
        self.seed
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigError {
    InvalidTickLimit(u16),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTickLimit(limit) => {
                write!(
                    formatter,
                    "tick limit must be between 1 and {MAX_TICKS}, got {limit}"
                )
            }
        }
    }
}

impl Error for ConfigError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Left,
    Right,
}

impl Side {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    pub const fn opponent(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageMode {
    Normal,
    Piercing,
    Retaliation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallCause {
    Natural,
    Forced { source: ItemRef },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FallTelemetry {
    pub attempts: u64,
    pub valid_targets: u64,
    pub no_target: u64,
    pub chance_miss: u64,
    pub prevented: u64,
    pub succeeded: u64,
    /// Ticks where opposing primary activations landed on the same 50ms rank.
    /// Zero by construction (left=even rank, right=odd rank); a nonzero value
    /// means the interleaving invariant regressed.
    pub shared_activation_ranks: u64,
    /// Ticks where both heroes died on one rank without a retaliation cascade.
    /// Zero by construction; Cactus lethal-cascade draws are counted elsewhere.
    pub shared_lethal_ranks: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BattleEvent {
    ItemActivated {
        item: ItemRef,
        kind: ItemKind,
    },
    DamageDealt {
        source: ItemRef,
        target: Side,
        mode: DamageMode,
        amount: u16,
    },
    HealthLost {
        source: ItemRef,
        target: Side,
        amount: u16,
    },
    Healed {
        source: ItemRef,
        target: Side,
        amount: u16,
    },
    BlockChanged {
        hero: Side,
        block: u16,
    },
    ItemSpeedChanged {
        item: ItemRef,
        basis_points: u16,
    },
    ItemFell {
        item: ItemRef,
        kind: ItemKind,
        cause: FallCause,
    },
    FallPrevented {
        item: ItemRef,
        by: ItemRef,
    },
    ItemConsumed {
        item: ItemRef,
        kind: ItemKind,
    },
    Poisoned {
        target: Side,
        stacks: u16,
    },
    PoisonDamage {
        target: Side,
        amount: u16,
    },
    PoisonCleansed {
        target: Side,
        remaining: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickReport {
    pub tick: u16,
    pub left_health: u16,
    pub right_health: u16,
    pub left_block: u16,
    pub right_block: u16,
    pub events: Vec<BattleEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    LeftWins,
    RightWins,
    Draw,
}

impl Outcome {
    pub const fn name(self) -> &'static str {
        match self {
            Self::LeftWins => "left wins",
            Self::RightWins => "right wins",
            Self::Draw => "draw",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BattleResult {
    pub outcome: Outcome,
    pub ticks: u16,
    pub left_health: u16,
    pub right_health: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BattleUpdate {
    Tick(TickReport),
    Finished(BattleResult),
}
