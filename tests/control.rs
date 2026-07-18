use backpack_battles::*;

#[test]
fn strap_intercepts_targeted_item_fall() {
    // Given
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![Item::new(ItemKind::GrapplingHook, Cell::new(0, 0))])
            .expect("valid grappling hook placement"),
    );
    let right = Hero::new(
        "right",
        100,
        Bag::new(vec![
            Item::new(ItemKind::Whetstone, Cell::new(0, 1)),
            Item::new(ItemKind::Strap, Cell::new(1, 1)),
        ])
        .expect("valid protected placement"),
    );
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(60, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 60);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::FallPrevented { item, by }
            if item.id.cell() == Cell::new(0, 1)
                && by.id.cell() == Cell::new(1, 1)
    )));
    assert!(
        !report
            .events
            .iter()
            .any(|event| matches!(event, BattleEvent::ItemFell { .. }))
    );
}

#[test]
fn net_forces_edge_weapon_to_fall_before_first_activation() {
    // Given
    let left = Hero::new(
        "left",
        100,
        Bag::new(vec![Item::new(ItemKind::Net, Cell::new(0, 0))]).expect("valid net placement"),
    );
    let right = Hero::new(
        "right",
        100,
        Bag::new(vec![Item::new(ItemKind::Crossbow, Cell::new(0, 0))])
            .expect("valid crossbow placement"),
    );
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(20, 1).expect("valid battle config"),
    );

    // When
    let report = report_at(&mut battle, 1);

    // Then
    assert!(report.events.iter().any(|event| matches!(
        event,
        BattleEvent::ItemFell {
            kind: ItemKind::Crossbow,
            cause: FallCause::Forced { .. },
            ..
        }
    )));
    assert!(!report.events.iter().any(|event| matches!(
        event,
        BattleEvent::ItemActivated {
            kind: ItemKind::Crossbow,
            ..
        }
    )));
}

#[test]
fn bomb_forces_an_opponent_weapon_to_fall_on_its_single_attempt() {
    // Given
    let left = Hero::new(
        "left",
        1000,
        Bag::new(vec![Item::new(ItemKind::Bomb, Cell::new(0, 0))]).expect("valid bomb placement"),
    );
    let right = Hero::new(
        "right",
        1000,
        Bag::new(vec![Item::new(ItemKind::Crossbow, Cell::new(0, 0))])
            .expect("valid crossbow placement"),
    );
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(600, 2).expect("valid battle config"),
    );

    // When
    let mut crossbow_fell = false;
    while let BattleUpdate::Tick(report) = battle.advance() {
        if report.events.iter().any(|event| {
            matches!(
                event,
                BattleEvent::ItemFell {
                    kind: ItemKind::Crossbow,
                    cause: FallCause::Forced { .. },
                    ..
                }
            )
        }) {
            crossbow_fell = true;
            break;
        }
    }

    // Then
    assert!(
        crossbow_fell,
        "bomb should force the crossbow to fall within the battle"
    );
}

#[test]
fn rotated_l_item_is_placeable_and_ids_by_its_minimum_cell() {
    // Given / When
    let rotated = Item::with_rotation(ItemKind::Thornmail, Cell::new(0, 0), Rotation::Deg270);

    // Then: the anchor cell is the minimum occupied cell, not a fixed (0, 0).
    assert_eq!(rotated.id().cell(), Cell::new(0, 1));
    assert!(Bag::new(vec![rotated]).is_ok());
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
