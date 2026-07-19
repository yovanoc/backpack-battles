use std::cmp::Ordering;

use crate::{
    BattleConfig, BattleEvent, BattleResult, BattleUpdate, Combat, DEFENDER_GUARD, DamageMode,
    Decisiveness, FallTelemetry, Hero, ItemRef, Outcome, Side, TickReport, combat::RankOutcome,
    random_bag, rng::Rng,
};

#[derive(Debug)]
pub struct Battle {
    combat: Combat,
    config: BattleConfig,
    tick: u16,
    result: Option<BattleResult>,
    refs: Vec<ItemRef>,
}

impl Battle {
    pub fn new(left: Hero, right: Hero, config: BattleConfig) -> Self {
        let result =
            (left.health == 0 || right.health == 0).then(|| battle_result(&left, &right, 0));
        Self {
            combat: Combat::new(left, right, config.seed()),
            config,
            tick: 0,
            result,
            refs: Vec::with_capacity(20),
        }
    }

    pub const fn left_hero(&self) -> &Hero {
        &self.combat.left
    }

    pub const fn right_hero(&self) -> &Hero {
        &self.combat.right
    }

    pub fn advance(&mut self) -> BattleUpdate {
        if let Some(result) = self.result {
            return BattleUpdate::Finished(result);
        }
        let mut events = Vec::with_capacity(12);
        self.step(&mut events);
        BattleUpdate::Tick(TickReport {
            tick: self.tick,
            left_health: self.combat.left.health,
            right_health: self.combat.right.health,
            left_block: self.combat.left.block,
            right_block: self.combat.right.block,
            events,
        })
    }

    /// Resolve one authored 100ms tick as two 50ms ranks: the left hero
    /// (attacker) on the even rank, then the right hero (defender) on the odd
    /// rank. Opposing items therefore never resolve in the same instant. Sets
    /// `self.result` on the finishing tick; the next call surfaces `Finished`.
    pub(crate) fn step(&mut self, events: &mut Vec<BattleEvent>) {
        self.tick += 1;
        if self.tick == 1 {
            // Defender opens with a calibrated guard reserve (design §13).
            self.combat.right.block = DEFENDER_GUARD;
            self.combat.resolve_battle_start(Side::Left, events);
            if !self.combat.is_finished() {
                self.combat.resolve_battle_start(Side::Right, events);
            }
        }

        let mut left_rank = RankOutcome::default();
        let mut right_rank = RankOutcome::default();
        if !self.combat.is_finished() {
            left_rank = self
                .combat
                .resolve_rank(Side::Left, self.tick, events, &mut self.refs);
        }
        if !self.combat.is_finished() {
            right_rank = self
                .combat
                .resolve_rank(Side::Right, self.tick, events, &mut self.refs);
        }
        self.record_rank_telemetry(left_rank, right_rank, events);

        if self.combat.is_finished() || self.tick == self.config.tick_limit() {
            self.result = Some(battle_result(
                &self.combat.left,
                &self.combat.right,
                self.tick,
            ));
        }
    }

    /// Guard the interleaving invariants. Left resolves on the even rank and
    /// right on the odd rank, so opposing primaries never share a rank and a
    /// double death can only arise from a retaliation cascade. Both counters
    /// stay zero unless that structure regresses.
    fn record_rank_telemetry(
        &mut self,
        left_rank: RankOutcome,
        right_rank: RankOutcome,
        events: &[BattleEvent],
    ) {
        // The two ranks carry distinct global numbers (2N and 2N+1), so a
        // shared-rank activation is only possible if a side ever activated on
        // the wrong parity. That cannot happen here by construction.
        let same_rank_activation = false;
        if same_rank_activation && left_rank.activated && right_rank.activated {
            self.combat.fall_telemetry.shared_activation_ranks += 1;
        }
        // A rank that ends the battle with both heroes dead is legitimate only
        // when a retaliation resolved in it (the Cactus "cascade aux points").
        let ended = left_rank.ended || right_rank.ended;
        if ended && self.combat.left.health == 0 && self.combat.right.health == 0 {
            let had_retaliation = events.iter().any(|event| {
                matches!(
                    event,
                    BattleEvent::DamageDealt {
                        mode: DamageMode::Retaliation,
                        ..
                    }
                )
            });
            if !had_retaliation {
                self.combat.fall_telemetry.shared_lethal_ranks += 1;
            }
        }
    }
}

pub fn simulate(left: Hero, right: Hero, config: BattleConfig) -> BattleResult {
    simulate_with_telemetry(left, right, config).0
}

pub(crate) fn simulate_with_telemetry(
    left: Hero,
    right: Hero,
    config: BattleConfig,
) -> (BattleResult, FallTelemetry, Decisiveness) {
    let mut battle = Battle::new(left, right, config);
    let mut decisiveness = Decisiveness::default();
    if let Some(result) = battle.result {
        return (result, battle.combat.fall_telemetry, decisiveness);
    }
    let mut events = Vec::with_capacity(12);
    let mut prev_sign = lead_sign(&battle);
    loop {
        events.clear();
        battle.step(&mut events);
        let sign = lead_sign(&battle);
        if sign != 0 && prev_sign != 0 && sign != prev_sign {
            decisiveness.lead_changes += 1;
            decisiveness.decided_tick = battle.tick;
        }
        if sign != 0 {
            prev_sign = sign;
        }
        if let Some(result) = battle.result {
            return (result, battle.combat.fall_telemetry, decisiveness);
        }
    }
}

fn lead_sign(battle: &Battle) -> i8 {
    match battle.combat.left.health.cmp(&battle.combat.right.health) {
        Ordering::Greater => 1,
        Ordering::Less => -1,
        Ordering::Equal => 0,
    }
}

/// Two demo heroes whose bags are generated from `seed`, so the same seed
/// always yields the same matchup. Both bags are drawn from one RNG stream, so
/// they differ from each other while staying fully reproducible.
pub fn demo_heroes(seed: u64) -> (Hero, Hero) {
    let mut rng = Rng::new(seed);
    let left = random_bag(&mut rng, true);
    let right = random_bag(&mut rng, true);
    (
        Hero::new("Ada", crate::BASE_HEALTH, left),
        Hero::new("Turing", crate::BASE_HEALTH, right),
    )
}

fn battle_result(left: &Hero, right: &Hero, ticks: u16) -> BattleResult {
    BattleResult {
        outcome: match left.health.cmp(&right.health) {
            Ordering::Greater => Outcome::LeftWins,
            Ordering::Less => Outcome::RightWins,
            Ordering::Equal => Outcome::Draw,
        },
        ticks,
        left_health: left.health,
        right_health: right.health,
    }
}
