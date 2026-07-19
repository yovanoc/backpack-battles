use backpack_battles::*;

#[test]
fn normal_damage_resolves_armor_then_block_then_retaliation() {
    // Given
    // Attacker on the LEFT (unguarded) so the Cactus retaliation lands on an
    // unguarded target; defensive rig on the right carries DEFENDER_GUARD.
    let left = attacking_hero(ItemKind::WoodenSword);
    let right = defensive_hero(true);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(40, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 40);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Right,
            mode: DamageMode::Normal,
            amount: 1,
            ..
        }
    )));
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Retaliation,
            amount: 2,
            ..
        }
    )));
    // Right opens with DEFENDER_GUARD (2); Shield adds 6 (max 20). WoodenSword
    // (8) minus Armor (1) leaves 7; block absorbs 6, so 1 leaks and block ends 0.
    assert_eq!(report.right_block, 0);
}

#[test]
fn piercing_damage_bypasses_armor_and_retaliation_but_spends_block() {
    // Given
    let left = defensive_hero(true);
    let right = attacking_hero(ItemKind::Windbreaker);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(50, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 50);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Piercing,
            amount: 0,
            ..
        }
    )));
    assert!(!report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            mode: DamageMode::Retaliation,
            ..
        }
    )));
    assert_eq!(report.left_block, 0);
}

#[test]
fn direct_health_loss_bypasses_armor_and_block() {
    // Given
    let left = defensive_hero(false);
    let right = attacking_hero(ItemKind::Leech);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(28, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 28);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::HealthLost {
            target: Side::Left,
            amount: 1,
            ..
        }
    )));
    assert_eq!(report.left_block, 6);
}

#[test]
fn war_banner_adds_seven_damage_to_an_adjacent_weapon() {
    // Given
    // Attacking rig on the RIGHT so the adjacency bonus is measured against an
    // unguarded left hero, independent of DEFENDER_GUARD.
    let left = Hero::new("left", 100, Bag::new(Vec::new()).expect("valid empty bag"));
    let right = Hero::new(
        "right",
        100,
        Bag::new(vec![
            Item::new(ItemKind::WoodenSword, Cell::new(0, 0)),
            Item::new(ItemKind::WarBanner, Cell::new(0, 1)),
        ])
        .expect("valid offensive placement"),
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
            mode: DamageMode::Normal,
            amount: 15,
            ..
        }
    )));
}

#[test]
fn dagger_opens_at_tick_five_for_four_damage() {
    // Given
    let left = Hero::new("left", 100, Bag::new(Vec::new()).expect("valid empty bag"));
    let right = attacking_hero(ItemKind::Dagger);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(5, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 5);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Normal,
            amount: 4,
            ..
        }
    )));
}

#[test]
fn rapier_pierces_armor_for_six_damage_at_tick_twentysix() {
    // Given: an Armor-only hero on the LEFT (unguarded, no block) so the
    // Rapier's piercing damage is measured against armor it must ignore.
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![Item::new(ItemKind::Armor, Cell::new(0, 0))]).expect("valid bag"),
    );
    let right = attacking_hero(ItemKind::Rapier);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(26, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 26);

    // Then: 6 piercing damage ignores the left Armor's 2 armor entirely.
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Piercing,
            amount: 6,
            ..
        }
    )));
}

#[test]
fn warhammer_hits_for_eighteen_damage_at_tick_thirty_seven() {
    // Given
    let left = Hero::new("left", 100, Bag::new(Vec::new()).expect("valid empty bag"));
    let right = attacking_hero(ItemKind::Warhammer);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(37, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 37);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Normal,
            amount: 18,
            ..
        }
    )));
}

#[test]
fn grimoire_opens_at_tick_thirty_for_ten_damage() {
    // Given
    let left = Hero::new("left", 100, Bag::new(Vec::new()).expect("valid empty bag"));
    let right = attacking_hero(ItemKind::Grimoire);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(30, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 30);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Normal,
            amount: 10,
            ..
        }
    )));
}

#[test]
fn grimoire_gains_five_damage_after_each_activation() {
    // Given
    let left = Hero::new("left", 100, Bag::new(Vec::new()).expect("valid empty bag"));
    let right = attacking_hero(ItemKind::Grimoire);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(73, 1).expect("valid battle config"),
    );
    let _ = report_at(&mut battle, 30);

    // When
    let report = report_at(&mut battle, 73);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::DamageDealt {
            target: Side::Left,
            mode: DamageMode::Normal,
            amount: 15,
            ..
        }
    )));
}

#[test]
fn poison_vial_bypasses_defenses_without_healing_its_owner() {
    // Given
    let left = defensive_hero(false);
    let right = attacking_hero(ItemKind::PoisonVial);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(30, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 30);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::HealthLost {
            target: Side::Left,
            amount: 2,
            ..
        }
    )));
    assert!(!report.events.iter().any(|event| matches!(
        event,
        BattleEvent::Healed {
            target: Side::Right,
            ..
        }
    )));
    assert_eq!(report.left_block, 6);
}

#[test]
fn venom_fang_stacks_poison_that_ticks_down_ignoring_defenses() {
    // Given: an Armor-only hero on the LEFT (no Shield, so no cleanse) so poison
    // (2 stacks, raw) is measured against armor it must ignore.
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![Item::new(ItemKind::Armor, Cell::new(0, 0))]).expect("valid bag"),
    );
    let right = attacking_hero(ItemKind::VenomFang);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(14, 1).expect("valid battle config"),
    );

    // When: Venom Fang applies 2 poison on its tick-12 activation; the left
    // hero then takes 2 poison at tick 13 and 1 at tick 14 as the stack decays.
    let applied = report_at(&mut battle, 12);
    let first_tick = report_at(&mut battle, 13);
    let second_tick = report_at(&mut battle, 14);

    // Then
    assert!(applied.events.iter().any(|event| matches!(
        event,
        BattleEvent::Poisoned {
            target: Side::Left,
            stacks: 2,
        }
    )));
    assert!(first_tick.events.iter().any(|event| matches!(
        event,
        BattleEvent::PoisonDamage {
            target: Side::Left,
            amount: 2,
        }
    )));
    assert!(second_tick.events.iter().any(|event| matches!(
        event,
        BattleEvent::PoisonDamage {
            target: Side::Left,
            amount: 1,
        }
    )));
}

fn attacking_hero(kind: ItemKind) -> Hero {
    Hero::new(
        "right",
        100,
        Bag::new(vec![Item::new(kind, Cell::new(0, 0))]).expect("valid item placement"),
    )
}

fn defensive_hero(with_cactus: bool) -> Hero {
    let mut items = vec![
        Item::new(ItemKind::Armor, Cell::new(0, 0)),
        Item::new(ItemKind::Shield, Cell::new(3, 0)),
    ];
    if with_cactus {
        items.push(Item::new(ItemKind::Cactus, Cell::new(0, 2)));
    }
    Hero::new(
        "right",
        100,
        Bag::new(items).expect("valid defensive placement"),
    )
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
