use backpack_battles::*;

#[test]
fn loose_buckler_adds_forty_two_max_health() {
    // Given / When
    let hero = hero("left", 100, vec![item(ItemKind::LooseBuckler, 0, 0)]);

    // Then
    assert_eq!(hero.health(), 142);
    assert_eq!(hero.max_health(), 142);
}

#[test]
fn battle_replays_exactly_when_seed_is_the_same() {
    // Given
    let config = BattleConfig::new(600, 42).expect("valid battle config");
    let (left, right) = demo_heroes(42);
    let mut first = Battle::new(left.clone(), right.clone(), config);
    let mut second = Battle::new(left, right, config);

    // When
    let first_replay = replay(&mut first);
    let second_replay = replay(&mut second);

    // Then
    assert_eq!(first_replay, second_replay);
}

#[test]
fn buckler_effect_is_removed_on_the_tick_it_falls() {
    // Given
    let left = hero("left", 30, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let right = hero("right", 30, vec![item(ItemKind::LooseBuckler, 0, 0)]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(20, 6).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 20);

    // Then: within tick 20, the left attacker damage lands before the right
    // buckler falls on its own rank.
    let damage_index = report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::DamageDealt {
                target: Side::Right,
                mode: DamageMode::Normal,
                ..
            }
        )
    });
    let fell_index = report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::ItemFell {
                kind: ItemKind::LooseBuckler,
                cause: FallCause::Natural,
                ..
            }
        )
    });
    assert!(damage_index.is_some() && fell_index.is_some());
    assert!(damage_index.unwrap() < fell_index.unwrap());
    // Right opens 72/72 (30 base + 42 buckler) with DEFENDER_GUARD; the hit is
    assert_eq!(report.right_health, 30);
}

#[test]
fn whetstone_boosts_an_adjacent_weapon() {
    // Given
    // Sword + Whetstone on the RIGHT (defender), so the adjacency bonus is
    // measured against an unguarded left hero, independent of DEFENDER_GUARD.
    let left = hero("left", 30, vec![]);
    let right = hero(
        "right",
        30,
        vec![
            item(ItemKind::WoodenSword, 0, 0),
            item(ItemKind::Whetstone, 1, 0),
        ],
    );
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(20, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 20);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            amount: 10,
            ..
        }
    )));
}

#[test]
fn bag_rejects_overlapping_item_shapes() {
    // Given / When
    let bag = Bag::new(vec![
        item(ItemKind::WoodenSword, 0, 0),
        item(ItemKind::Whetstone, 0, 1),
    ]);

    // Then
    assert_eq!(
        bag,
        Err(BagError::Overlap {
            item: ItemKind::Whetstone,
            at: Cell::new(0, 1),
        })
    );
}

#[test]
fn battle_config_rejects_tick_limits_outside_the_domain() {
    // Given / When / Then
    assert_eq!(
        BattleConfig::new(0, 1),
        Err(ConfigError::InvalidTickLimit(0))
    );
    assert_eq!(
        BattleConfig::new(2001, 1),
        Err(ConfigError::InvalidTickLimit(2001))
    );
}

#[test]
fn dead_hero_cannot_attack() {
    // Given
    let left = hero("left", 0, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let right = hero("right", 30, vec![]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(20, 1).expect("valid battle config"),
    );

    // When
    let update = battle.advance();

    // Then
    assert_eq!(
        update,
        BattleUpdate::Finished(BattleResult {
            outcome: Outcome::RightWins,
            ticks: 0,
            left_health: 0,
            right_health: 30,
        })
    );
}

#[test]
fn bag_input_order_does_not_change_seeded_battle() {
    // Given
    let first_items = vec![
        item(ItemKind::LooseBuckler, 0, 0),
        item(ItemKind::LooseBuckler, 3, 0),
    ];
    let mut reversed_items = first_items.clone();
    reversed_items.reverse();
    let right = hero("right", 100, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let config = BattleConfig::new(60, 6).expect("valid battle config");
    let mut first = Battle::new(hero("left", 100, first_items), right.clone(), config);
    let mut second = Battle::new(hero("left", 100, reversed_items), right, config);

    // When
    let first_replay = replay(&mut first);
    let second_replay = replay(&mut second);

    // Then
    assert_eq!(first_replay, second_replay);
}

#[test]
fn per_side_natural_fall_rolls_ignore_bag_input_order() {
    // Given: seed 127 rolls pass, pass, miss. Left has one fall candidate;
    // right has two, so the rolls must be consumed left-rank then right-rank.
    let left = hero(
        "left",
        500,
        vec![
            item(ItemKind::WoodenSword, 0, 0),
            item(ItemKind::LooseBuckler, 2, 0),
        ],
    );
    let first_right_items = vec![
        item(ItemKind::LooseBuckler, 0, 0),
        item(ItemKind::LooseBuckler, 3, 0),
    ];
    let mut reversed_right_items = first_right_items.clone();
    reversed_right_items.reverse();
    let config = BattleConfig::new(20, 127).expect("valid battle config");
    let mut first = Battle::new(left.clone(), hero("right", 500, first_right_items), config);
    let mut second = Battle::new(left, hero("right", 500, reversed_right_items), config);

    // When
    let first_report = report_at(&mut first, 20);
    let second_report = report_at(&mut second, 20);

    // Then
    let natural_fall_sides: Vec<Side> = first_report
        .events
        .iter()
        .filter_map(|event| match event {
            BattleEvent::ItemFell {
                item,
                cause: FallCause::Natural,
                ..
            } => Some(item.side),
            _ => None,
        })
        .collect();
    let left_fall = first_report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::ItemFell {
                item: ItemRef {
                    side: Side::Left,
                    ..
                },
                cause: FallCause::Natural,
                ..
            }
        )
    });
    let damage = first_report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::DamageDealt {
                target: Side::Right,
                mode: DamageMode::Normal,
                ..
            }
        )
    });
    let right_fall = first_report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::ItemFell {
                item: ItemRef {
                    side: Side::Right,
                    ..
                },
                cause: FallCause::Natural,
                ..
            }
        )
    });
    assert!(left_fall.is_some() && damage.is_some() && right_fall.is_some());
    assert_eq!(natural_fall_sides, vec![Side::Left, Side::Right]);
    assert!(left_fall.unwrap() < damage.unwrap());
    assert!(damage.unwrap() < right_fall.unwrap());
    assert_eq!(first_report, second_report);
}

fn item(kind: ItemKind, x: u8, y: u8) -> Item {
    Item::new(kind, Cell::new(x, y))
}

fn hero(name: &str, health: u16, items: Vec<Item>) -> Hero {
    Hero::new(name, health, Bag::new(items).expect("valid bag"))
}

fn report_at(battle: &mut Battle, tick: u16) -> TickReport {
    loop {
        match battle.advance() {
            BattleUpdate::Tick(report) if report.tick == tick => return report,
            BattleUpdate::Tick(_) => {}
            BattleUpdate::Finished(result) => panic!("battle finished too early: {result:?}"),
        }
    }
}

fn replay(battle: &mut Battle) -> Vec<BattleUpdate> {
    let mut updates = Vec::new();
    loop {
        let update = battle.advance();
        let finished = matches!(update, BattleUpdate::Finished(_));
        updates.push(update);
        if finished {
            return updates;
        }
    }
}
