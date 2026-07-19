use super::{
    BalanceConfig, ItemStat, MatchupStat, Tally, distinct_kinds, fresh_stats, generator, record,
    swap,
};
use crate::{
    Archetype, BattleConfig, FallTelemetry, Hero, MAX_TICKS, Outcome, rng::Rng, simulate,
    simulate_with_telemetry,
};

pub(super) struct ShardReport {
    pub(super) stats: Vec<ItemStat>,
    pub(super) left_wins: u64,
    pub(super) right_wins: u64,
    pub(super) draws: u64,
    pub(super) total_ticks: u64,
    pub(super) unclassified_matchups: u64,
    pub(super) matchups: [[MatchupStat; Archetype::COUNT]; Archetype::COUNT],
    pub(super) mirrored_left_wins: u64,
    pub(super) mirrored_right_wins: u64,
    pub(super) mirrored_draws: u64,
    pub(super) fall_telemetry: FallTelemetry,
    pub(super) duration_histogram: [u64; MAX_TICKS as usize + 1],
    pub(super) lead_changes: u64,
    pub(super) swing_battles: u64,
    pub(super) decided_tick_total: u64,
}

pub(super) fn run(config: &BalanceConfig, shard: u64, shards: u64) -> ShardReport {
    let mut report = ShardReport {
        stats: fresh_stats(),
        left_wins: 0,
        right_wins: 0,
        draws: 0,
        total_ticks: 0,
        unclassified_matchups: 0,
        matchups: [[MatchupStat::default(); Archetype::COUNT]; Archetype::COUNT],
        mirrored_left_wins: 0,
        mirrored_right_wins: 0,
        mirrored_draws: 0,
        fall_telemetry: FallTelemetry::default(),
        duration_histogram: [0; MAX_TICKS as usize + 1],
        lead_changes: 0,
        swing_battles: 0,
        decided_tick_total: 0,
    };
    let mut index = shard;
    while index < config.battles {
        run_battle(config, index, &mut report);
        index += shards;
    }
    report
}

fn run_battle(config: &BalanceConfig, index: u64, report: &mut ShardReport) {
    let mut rng = Rng::new(campaign_seed(config.seed, index));
    let (left_profile, right_profile) = match config.campaign_mode.profile_count() {
        Some(profiles) => (index % profiles, (index / profiles) % profiles),
        None => (index.wrapping_mul(2), index.wrapping_mul(2).wrapping_add(1)),
    };
    let left_bag = generator::generated_bag(
        &mut rng,
        config.allow_rotation,
        config.campaign_mode,
        left_profile,
    );
    let right_bag = generator::generated_bag(
        &mut rng,
        config.allow_rotation,
        config.campaign_mode,
        right_profile,
    );
    let left_kinds = distinct_kinds(&left_bag);
    let right_kinds = distinct_kinds(&right_bag);
    let left_archetype = super::dominant_archetype(&left_bag);
    let right_archetype = super::dominant_archetype(&right_bag);
    let battle_seed = rng.next_u64();
    let battle_config =
        BattleConfig::new(config.tick_limit, battle_seed).expect("tick limit already validated");
    let (result, falls, decisiveness) = simulate_with_telemetry(
        Hero::new("left", config.hero_health, left_bag.clone()),
        Hero::new("right", config.hero_health, right_bag.clone()),
        battle_config,
    );
    report.fall_telemetry.attempts += falls.attempts;
    report.fall_telemetry.valid_targets += falls.valid_targets;
    report.fall_telemetry.no_target += falls.no_target;
    report.fall_telemetry.chance_miss += falls.chance_miss;
    report.fall_telemetry.prevented += falls.prevented;
    report.fall_telemetry.succeeded += falls.succeeded;
    report.total_ticks += u64::from(result.ticks);
    report.duration_histogram[usize::from(result.ticks)] += 1;
    report.lead_changes += u64::from(decisiveness.lead_changes);
    if decisiveness.lead_changes > 0 {
        report.swing_battles += 1;
    }
    report.decided_tick_total += u64::from(decisiveness.decided_tick);
    let (left_tally, right_tally) = match result.outcome {
        Outcome::LeftWins => {
            report.left_wins += 1;
            (Tally::Win, Tally::Loss)
        }
        Outcome::RightWins => {
            report.right_wins += 1;
            (Tally::Loss, Tally::Win)
        }
        Outcome::Draw => {
            report.draws += 1;
            (Tally::Draw, Tally::Draw)
        }
    };
    match (left_archetype, right_archetype) {
        (Some(left), Some(right)) => {
            let matchup = &mut report.matchups[left as usize][right as usize];
            matchup.battles += 1;
            match result.outcome {
                Outcome::LeftWins => matchup.left_wins += 1,
                Outcome::RightWins => matchup.right_wins += 1,
                Outcome::Draw => matchup.draws += 1,
            }
        }
        _ => report.unclassified_matchups += 1,
    }
    record(&mut report.stats, &left_kinds, left_tally);
    record(&mut report.stats, &right_kinds, right_tally);
    if config.mirror_sides {
        match simulate(
            Hero::new("left", config.hero_health, right_bag.clone()),
            Hero::new("right", config.hero_health, left_bag.clone()),
            battle_config,
        )
        .outcome
        {
            Outcome::LeftWins => report.mirrored_left_wins += 1,
            Outcome::RightWins => report.mirrored_right_wins += 1,
            Outcome::Draw => report.mirrored_draws += 1,
        }
    }
    if battle_seed.is_multiple_of(4) {
        swap::run(
            config,
            &mut rng,
            &left_bag,
            &right_bag,
            battle_config,
            left_tally,
            &mut report.stats,
        );
    }
}

fn campaign_seed(seed: u64, index: u64) -> u64 {
    let mut mixer = Rng::new(seed.wrapping_add(index));
    mixer.next_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjacent_battles_do_not_use_shifted_rng_streams() {
        let mut first = Rng::new(campaign_seed(42, 0));
        first.next_u64();
        let mut second = Rng::new(campaign_seed(42, 1));

        assert_ne!(first.next_u64(), second.next_u64());
    }
}
