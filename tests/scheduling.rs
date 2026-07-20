use backpack_battles::*;

#[test]
fn crossbow_uses_initial_charge_then_recurring_charge() {
    // Given
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![Item::new(ItemKind::Crossbow, Cell::new(0, 0))])
            .expect("valid crossbow placement"),
    );
    let right = Hero::new("right", 100, Bag::new(vec![]).expect("empty bag is valid"));
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(50, 1).expect("valid battle config"),
    );

    // When
    let activation_ticks = activation_ticks(&mut battle, ItemKind::Crossbow);

    // Then
    assert_eq!(activation_ticks, vec![15, 45]);
}

#[test]
fn hourglass_stacks_speed_on_adjacent_items() {
    // Given
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![
            Item::new(ItemKind::Hourglass, Cell::new(0, 0)),
            Item::new(ItemKind::WoodenSword, Cell::new(1, 0)),
        ])
        .expect("valid adjacent placement"),
    );
    let right = Hero::new("right", 500, Bag::new(vec![]).expect("empty bag is valid"));
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(45, 1).expect("valid battle config"),
    );

    // When
    let mut speed_changes = Vec::new();
    while let BattleUpdate::Tick(report) = battle.advance() {
        for event in report.events {
            if let BattleEvent::ItemSpeedChanged { item, basis_points } = event
                && item.id.cell() == Cell::new(1, 0)
            {
                speed_changes.push((report.tick, basis_points));
            }
        }
    }

    // Then
    assert_eq!(speed_changes, vec![(20, 400), (40, 800)]);
}

#[test]
fn accumulated_speed_shortens_an_active_charge() {
    // Given
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![
            Item::new(ItemKind::Hourglass, Cell::new(1, 0)),
            Item::new(ItemKind::Hourglass, Cell::new(2, 0)),
            Item::new(ItemKind::Crossbow, Cell::new(1, 1)),
        ])
        .expect("valid charging layout"),
    );
    let right = Hero::new("right", 500, Bag::new(vec![]).expect("empty bag is valid"));
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(45, 1).expect("valid battle config"),
    );

    // When
    let activation_ticks = activation_ticks(&mut battle, ItemKind::Crossbow);

    // Then
    assert_eq!(activation_ticks, vec![15, 43]);
}

#[test]
fn healing_potion_triggers_once_then_is_consumed() {
    // Given
    let left = Hero::new(
        "left",
        10,
        Bag::new(vec![Item::new(ItemKind::HealingPotion, Cell::new(0, 0))])
            .expect("valid potion placement"),
    );
    let right = Hero::new(
        "right",
        100,
        Bag::new(vec![Item::new(ItemKind::WoodenSword, Cell::new(0, 0))])
            .expect("valid sword placement"),
    );
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(21, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 21);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::Healed {
            target: Side::Left,
            amount: 8,
            ..
        }
    )));
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::ItemConsumed {
            kind: ItemKind::HealingPotion,
            ..
        }
    )));
    assert_eq!(report.left_health, 10);
}

fn activation_ticks(battle: &mut Battle, kind: ItemKind) -> Vec<u16> {
    let mut ticks = Vec::new();
    loop {
        match battle.advance() {
            BattleUpdate::Tick(report) => {
                if report.events.iter().any(|event| {
                    matches!(
                        event,
                        BattleEvent::ItemActivated {
                            kind: activated_kind,
                            ..
                        } if *activated_kind == kind
                    )
                }) {
                    ticks.push(report.tick);
                }
            }
            BattleUpdate::Finished(_) => return ticks,
        }
    }
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
