use crate::{Archetype, Bag, BattleConfig, ConfigError, FallTelemetry, ItemKind, MAX_TICKS};

mod classification;
mod generator;
mod meta;
mod shard;
mod stat;
mod swap;

use classification::dominant_archetype;
pub(crate) use generator::random_bag;
pub use meta::{
    CONTESTED_MAX_COUNTER, CYCLE_EDGE_WINRATE, CYCLE_MIN_SCC, ItemPresence, MetaConfig, MetaReport,
    NO_WELL_MIN_COUNTER, ROSTER_MAX_PRESENCE, SUBSTANTIAL_MIN_DISTANCE, VerdictReport, run_meta,
    run_verdict,
};
pub use stat::{BalanceReport, ItemStat, MatchupStat};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignMode {
    Random,
    Pure,
    Hybrid,
    Elite,
}

impl CampaignMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Pure => "pure",
            Self::Hybrid => "hybrid",
            Self::Elite => "elite",
        }
    }

    const fn profile_count(self) -> Option<u64> {
        match self {
            Self::Random => None,
            Self::Pure => Some(Archetype::COUNT as u64),
            Self::Hybrid => Some((Archetype::COUNT * (Archetype::COUNT - 1)) as u64),
            Self::Elite => None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BalanceConfig {
    pub battles: u64,
    pub seed: u64,
    pub tick_limit: u16,
    pub hero_health: u16,
    pub allow_rotation: bool,
    pub mirror_sides: bool,
    pub campaign_mode: CampaignMode,
}

/// Run many battles between randomly generated bags and tally, per item kind,
/// how often the bag holding it won.
///
/// Roughly every fourth battle also tries one matched swap: a random left-bag
/// item is replaced by a random kind fitting the same cells, then the identical
/// battle is rerun. This controls for size, position, enemy, and bag-mates.
///
/// Each battle is seeded independently from `(seed, battle_index)`, so the work
/// splits across CPU cores and the merged report is identical no matter how many
/// threads run it. Same `seed` and `battles` always produce the same report.
pub fn run_balance(config: &BalanceConfig) -> Result<BalanceReport, ConfigError> {
    // Fail fast on a bad tick limit instead of once per battle. Every battle
    // reuses this same limit, so inside the shards `new` can never fail.
    BattleConfig::new(config.tick_limit, config.seed)?;

    let shards = shard_count(config.battles);
    let mut stats = fresh_stats();
    let mut left_wins = 0;
    let mut right_wins = 0;
    let mut draws = 0;
    let mut total_ticks = 0;
    let mut unclassified_matchups = 0;
    let mut matchups = [[MatchupStat::default(); Archetype::COUNT]; Archetype::COUNT];
    let mut mirrored_left_wins = 0;
    let mut mirrored_right_wins = 0;
    let mut mirrored_draws = 0;
    let mut fall_telemetry = FallTelemetry::default();
    let mut duration_histogram = [0; MAX_TICKS as usize + 1];
    let mut lead_changes = 0;
    let mut swing_battles = 0;
    let mut decided_tick_total = 0;

    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..shards)
            .map(|shard| scope.spawn(move || shard::run(config, shard, shards)))
            .collect();
        for handle in handles {
            let local = handle.join().expect("balance shard panicked");
            left_wins += local.left_wins;
            right_wins += local.right_wins;
            draws += local.draws;
            total_ticks += local.total_ticks;
            lead_changes += local.lead_changes;
            swing_battles += local.swing_battles;
            decided_tick_total += local.decided_tick_total;
            unclassified_matchups += local.unclassified_matchups;
            mirrored_left_wins += local.mirrored_left_wins;
            mirrored_right_wins += local.mirrored_right_wins;
            mirrored_draws += local.mirrored_draws;
            fall_telemetry.attempts += local.fall_telemetry.attempts;
            fall_telemetry.valid_targets += local.fall_telemetry.valid_targets;
            fall_telemetry.no_target += local.fall_telemetry.no_target;
            fall_telemetry.chance_miss += local.fall_telemetry.chance_miss;
            fall_telemetry.prevented += local.fall_telemetry.prevented;
            fall_telemetry.succeeded += local.fall_telemetry.succeeded;
            fall_telemetry.shared_activation_ranks += local.fall_telemetry.shared_activation_ranks;
            fall_telemetry.shared_lethal_ranks += local.fall_telemetry.shared_lethal_ranks;
            for (total, local) in duration_histogram.iter_mut().zip(local.duration_histogram) {
                *total += local;
            }
            for (total, local) in stats.iter_mut().zip(local.stats) {
                total.bags += local.bags;
                total.wins += local.wins;
                total.losses += local.losses;
                total.draws += local.draws;
                total.swap_wins += local.swap_wins;
                total.swap_losses += local.swap_losses;
                total.swap_draws += local.swap_draws;
            }
            for (total_row, local_row) in matchups.iter_mut().zip(local.matchups) {
                for (total, local) in total_row.iter_mut().zip(local_row) {
                    total.battles += local.battles;
                    total.left_wins += local.left_wins;
                    total.right_wins += local.right_wins;
                    total.draws += local.draws;
                }
            }
        }
    });

    Ok(BalanceReport {
        battles: config.battles,
        left_wins,
        right_wins,
        draws,
        total_ticks,
        unclassified_matchups,
        matchups,
        mirrored_battles: if config.mirror_sides {
            config.battles
        } else {
            0
        },
        mirrored_left_wins,
        mirrored_right_wins,
        mirrored_draws,
        fall_telemetry,
        duration_histogram,
        lead_changes,
        swing_battles,
        decided_tick_total,
        stats,
    })
}

// ponytail: stride sharding across all cores; swap for a work-stealing pool only
// if uneven battle lengths ever make cores idle.
fn shard_count(battles: u64) -> u64 {
    let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    u64::try_from(cores).unwrap_or(1).min(battles).max(1)
}

fn fresh_stats() -> Vec<ItemStat> {
    ItemKind::ALL
        .iter()
        .map(|kind| ItemStat {
            kind: *kind,
            bags: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            swap_wins: 0,
            swap_losses: 0,
            swap_draws: 0,
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Tally {
    Win,
    Loss,
    Draw,
}

fn record(stats: &mut [ItemStat], kinds: &[ItemKind], tally: Tally) {
    for kind in kinds {
        // stats is built in ItemKind::ALL (declaration) order, so the enum
        // discriminant is a direct index - no linear search.
        let stat = &mut stats[*kind as usize];
        debug_assert_eq!(stat.kind, *kind, "stats must be indexed by ItemKind order");
        stat.bags += 1;
        match tally {
            Tally::Win => stat.wins += 1,
            Tally::Loss => stat.losses += 1,
            Tally::Draw => stat.draws += 1,
        }
    }
}

fn distinct_kinds(bag: &Bag) -> Vec<ItemKind> {
    let mut kinds: Vec<ItemKind> = Vec::new();
    for item in bag.items() {
        if !kinds.contains(&item.kind()) {
            kinds.push(item.kind());
        }
    }
    kinds
}
