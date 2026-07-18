use backpack_battles::{
    BalanceConfig, CampaignMode, Cell, Item, ItemKind, Offset, Rotation, run_balance,
};

#[test]
fn balance_report_is_deterministic_for_a_seed() {
    // Given
    let config = BalanceConfig {
        battles: 200,
        seed: 7,
        tick_limit: 200,
        hero_health: 100,
        allow_rotation: true,
        mirror_sides: false,
        campaign_mode: CampaignMode::Random,
    };

    // When
    let first = run_balance(&config).expect("valid balance config");
    let second = run_balance(&config).expect("valid balance config");

    // Then
    assert_eq!(first, second);
}

#[test]
fn per_item_tallies_sum_to_bag_appearances() {
    // Given
    let config = BalanceConfig {
        battles: 500,
        seed: 99,
        tick_limit: 300,
        hero_health: 100,
        allow_rotation: false,
        mirror_sides: false,
        campaign_mode: CampaignMode::Random,
    };

    // When
    let report = run_balance(&config).expect("valid balance config");

    // Then
    assert_eq!(report.battles, 500);
    assert!(report.draws <= report.battles);
    for stat in &report.stats {
        assert_eq!(
            stat.wins + stat.losses + stat.draws,
            stat.bags,
            "{:?} tallies must equal its bag appearances",
            stat.kind
        );
    }
}

#[test]
fn rotating_a_vertical_item_makes_it_horizontal() {
    // Given / When
    let upright = Item::new(ItemKind::WoodenSword, Cell::new(0, 0));
    let rotated = Item::with_rotation(ItemKind::WoodenSword, Cell::new(0, 0), Rotation::Deg90);

    // Then
    assert_eq!(sorted_cells(&upright), vec![cell(0, 0), cell(0, 1)]);
    assert_eq!(sorted_cells(&rotated), vec![cell(0, 0), cell(1, 0)]);
}

fn sorted_cells(item: &Item) -> Vec<Offset> {
    let mut cells = item.shape().to_vec();
    cells.sort_by_key(|offset| (offset.x, offset.y));
    cells
}

const fn cell(x: u8, y: u8) -> Offset {
    Offset { x, y }
}

#[test]
fn swap_experiments_are_bounded_and_footprint_matched() {
    // Given: no rotation, so footprint groups are exactly the shape constants.
    let config = BalanceConfig {
        battles: 300,
        seed: 11,
        tick_limit: 300,
        hero_health: 100,
        allow_rotation: false,
        mirror_sides: false,
        campaign_mode: CampaignMode::Random,
    };

    // When
    let report = run_balance(&config).expect("valid balance config");
    let stat = |kind: ItemKind| {
        report
            .stats
            .iter()
            .find(|stat| stat.kind == kind)
            .expect("every kind has a stat")
    };

    // Then: a kind with a unique footprint (Spear's L-tetromino) never gets
    // swap data. Crossbow's horizontal-two now has partners (Bomb, Shrapnel
    // Mine, Barricade Kit), so it can be swapped.
    assert_eq!(
        stat(ItemKind::Spear).swaps(),
        0,
        "L-tetromino has no partner"
    );

    let swaps: u64 = report.stats.iter().map(|stat| stat.swaps()).sum();
    assert!(swaps > 0, "swaps must occur in 300 battles");
    assert_eq!(swaps % 2, 0, "each experiment records a matched pair");
    assert!(swaps <= report.battles * 2, "at most one pair per battle");
}

#[test]
fn confidence_ranking_penalizes_small_samples() {
    // Given: a perfect but tiny sample and a strong well-measured sample.
    let stat = |wins, losses| backpack_battles::ItemStat {
        kind: ItemKind::Leech,
        bags: wins + losses,
        wins: 0,
        losses: 0,
        draws: 0,
        swap_wins: wins,
        swap_losses: losses,
        swap_draws: 0,
    };

    // When
    let tiny = stat(1, 0).rank_score();
    let measured = stat(80, 20).rank_score();

    // Then
    assert!(measured > tiny);
}

#[test]
fn draws_count_as_half_an_outcome() {
    // Given
    let stat = backpack_battles::ItemStat {
        kind: ItemKind::Leech,
        bags: 10,
        wins: 0,
        losses: 0,
        draws: 10,
        swap_wins: 0,
        swap_losses: 0,
        swap_draws: 10,
    };

    // When / Then
    assert_eq!(stat.score_rate(), 0.5);
    assert_eq!(stat.swap_score_rate(), 0.5);
    assert!(stat.rank_score() < 0.5);
}

#[test]
fn balance_report_accounts_for_sides_matchups_and_duration() {
    // Given
    let config = BalanceConfig {
        battles: 200,
        seed: 31,
        tick_limit: 300,
        hero_health: 100,
        allow_rotation: true,
        mirror_sides: false,
        campaign_mode: CampaignMode::Random,
    };

    // When
    let report = run_balance(&config).expect("valid balance config");
    let matchup_battles: u64 = report
        .matchups
        .iter()
        .flatten()
        .map(|matchup| matchup.battles)
        .sum();

    // Then
    assert_eq!(report.left_wins + report.right_wins + report.draws, 200);
    assert_eq!(matchup_battles + report.unclassified_matchups, 200);
    assert!((200..=60_000).contains(&report.total_ticks));
    assert!((1..=report.duration_p90()).contains(&report.duration_p50()));
    assert!(report.duration_p90() <= report.duration_p99());
    assert!(report.fall_telemetry.attempts > 0);
    assert!(report.fall_telemetry.succeeded <= report.fall_telemetry.valid_targets);
    for kind in ItemKind::ALL {
        let _ = kind.archetype();
    }
}

#[test]
fn interleaved_ranks_never_share_a_rank_or_double_kill_without_cascade() {
    // The 50ms interleaving puts left on even ranks and right on odd ranks, so
    // opposing primaries never share a rank and a double death can only come
    // from a retaliation cascade. Both tripwire counters must stay zero across
    // a large mixed campaign.
    let config = BalanceConfig {
        battles: 5_000,
        seed: 31,
        tick_limit: 600,
        hero_health: 100,
        allow_rotation: true,
        mirror_sides: true,
        campaign_mode: CampaignMode::Hybrid,
    };

    let report = run_balance(&config).expect("valid balance config");

    assert_eq!(report.fall_telemetry.shared_activation_ranks, 0);
    assert_eq!(report.fall_telemetry.shared_lethal_ranks, 0);
}

#[test]
fn mirrored_campaign_preserves_item_stats_and_pairs_sides() {
    // Given
    let base = BalanceConfig {
        battles: 200,
        seed: 17,
        tick_limit: 300,
        hero_health: 100,
        allow_rotation: true,
        mirror_sides: false,
        campaign_mode: CampaignMode::Random,
    };
    let mirrored = BalanceConfig {
        mirror_sides: true,
        ..base
    };

    // When
    let base_report = run_balance(&base).expect("valid base campaign");
    let mirrored_report = run_balance(&mirrored).expect("valid mirrored campaign");

    // Then
    assert_eq!(mirrored_report.stats, base_report.stats);
    assert_eq!(mirrored_report.mirrored_battles, 200);
    assert_eq!(
        mirrored_report.mirrored_left_wins
            + mirrored_report.mirrored_right_wins
            + mirrored_report.mirrored_draws,
        200
    );
}

#[test]
fn pure_campaigns_produce_classified_matchups() {
    // Given
    let config = BalanceConfig {
        battles: 100,
        seed: 23,
        tick_limit: 200,
        hero_health: 100,
        allow_rotation: true,
        mirror_sides: false,
        campaign_mode: CampaignMode::Pure,
    };

    // When
    let report = run_balance(&config).expect("valid pure campaign");

    // Then
    assert_eq!(report.unclassified_matchups, 0);
    assert!(
        report
            .matchups
            .iter()
            .flatten()
            .all(|matchup| matchup.battles > 0)
    );
}
