use backpack_battles::*;

// The engine runs two 50ms ranks per authored 100ms tick: the left hero
// (attacker) resolves on the even rank, the right hero (defender) on the odd
// rank. Opposing items therefore never resolve in the same instant.

#[test]
fn attacker_kill_cancels_the_defender_activation_on_the_same_tick() {
    // Both Poison Vials fire on tick 15. Left resolves first (even rank) and
    // its block-bypassing LoseHealth kills the right hero before the right
    // rank runs, so the right vial never activates.
    let left = hero("left", 2, vec![item(ItemKind::PoisonVial, 0, 0)]);
    let right = hero("right", 2, vec![item(ItemKind::PoisonVial, 0, 0)]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(20, 1).expect("valid battle config"),
    );

    let result = run_to_end(&mut battle);

    assert_eq!(result.outcome, Outcome::LeftWins);
    assert_eq!(result.right_health, 0);
    assert!(
        result.left_health > 0,
        "left must survive; right never acts"
    );
}

#[test]
fn same_cadence_weapons_resolve_left_before_right_within_a_tick() {
    // High HP so neither dies: both Wooden Swords fire on tick 20. The left
    // (attacker, even rank) event must precede the right (defender, odd rank).
    let left = hero("left", 500, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let right = hero("right", 500, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(25, 1).expect("valid battle config"),
    );

    let report = report_at(&mut battle, 20);
    let order: Vec<Side> = report
        .events
        .iter()
        .filter_map(|event| match event {
            BattleEvent::DamageDealt { target, .. } => Some(*target),
            _ => None,
        })
        .collect();

    // Left attacks the right first, then the right attacks the left.
    assert_eq!(order, vec![Side::Right, Side::Left]);
}

#[test]
fn lethal_hit_still_triggers_cactus_retaliation_in_the_same_cascade() {
    // Left Spear (16 normal) kills a low-HP right Cactus hero; the Cactus
    // retaliation (design layer-3 "cascade aux points") must resolve before
    // the battle is declared finished.
    let left = hero("left", 3, vec![item(ItemKind::Spear, 0, 0)]);
    let right = hero("right", 4, vec![item(ItemKind::Cactus, 0, 0)]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(35, 1).expect("valid battle config"),
    );

    let report = report_at(&mut battle, 30);
    let lethal = report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::DamageDealt {
                target: Side::Right,
                mode: DamageMode::Normal,
                ..
            }
        )
    });
    let retaliation = report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::DamageDealt {
                target: Side::Left,
                mode: DamageMode::Retaliation,
                ..
            }
        )
    });

    assert!(lethal.is_some(), "left spear must land the lethal hit");
    assert!(
        retaliation.is_some(),
        "cactus retaliation must resolve on the lethal blow"
    );
    assert!(lethal.unwrap() < retaliation.unwrap());
}

#[test]
fn defender_opens_with_guard_block_spent_after_armor() {
    // The right (defender) hero starts each battle with DEFENDER_GUARD block.
    // A left Wooden Sword (8) hits a right Armor hero: armor reduces by 2, the
    // opening guard (2) absorbs 2 of the remaining 6, and 4 leaks to health.
    let left = hero("left", 500, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let right = hero("right", 500, vec![item(ItemKind::Armor, 0, 0)]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(25, 1).expect("valid battle config"),
    );

    // Opening block is visible before any activation.
    let opening = report_at(&mut battle, 1);
    assert_eq!(opening.right_block, DEFENDER_GUARD);

    // Guard is 2, so 6 damage spends the whole guard and 4 leaks to health.
    let right_max = 500 + 40; // Armor +40 max health
    let report = report_at(&mut battle, 20);
    assert_eq!(
        report.right_block, 0,
        "guard fully spent by the 6 that lands"
    );
    assert_eq!(report.right_health, right_max - 4);
}

#[test]
fn right_natural_fall_happens_after_the_same_tick_left_rank() {
    // A right Loose Buckler due to fall on tick 20 stays active during the
    // left rank (the sword hits the guarded, armored right), then the buckler
    // falls on the right rank, dropping max health and clamping current health.
    let left = hero("left", 30, vec![item(ItemKind::WoodenSword, 0, 0)]);
    let right = hero("right", 30, vec![item(ItemKind::LooseBuckler, 0, 0)]);
    let mut battle = Battle::new(
        left,
        right,
        BattleConfig::new(20, 6).expect("valid battle config"),
    );

    let report = report_at(&mut battle, 20);
    let damage = report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::DamageDealt {
                target: Side::Right,
                mode: DamageMode::Normal,
                ..
            }
        )
    });
    let fell = report.events.iter().position(|event| {
        matches!(
            event,
            BattleEvent::ItemFell {
                kind: ItemKind::LooseBuckler,
                ..
            }
        )
    });

    assert!(damage.is_some() && fell.is_some());
    assert!(
        damage.unwrap() < fell.unwrap(),
        "left damage precedes right fall"
    );
    // Right opens 60/60 (30 base + 30 buckler) with DEFENDER_GUARD. The sword
    // partly leaks, then the buckler falls → max 30, clamping current health.
    assert_eq!(report.right_health, 30);
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

fn run_to_end(battle: &mut Battle) -> BattleResult {
    loop {
        if let BattleUpdate::Finished(result) = battle.advance() {
            return result;
        }
    }
}
