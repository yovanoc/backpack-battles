use crate::{DamageMode, Effect, EffectKind, FallCause, Item, ItemRef, ItemState, ItemTarget};

pub(crate) fn activate(item: &mut Item, source: ItemRef) -> Vec<Effect> {
    let effects = match item.state_mut() {
        ItemState::WoodenSword => vec![damage(source, 8, DamageMode::Normal)],
        ItemState::Crossbow => vec![damage(source, 10, DamageMode::Normal)],
        ItemState::Hourglass => vec![Effect::new(
            source,
            EffectKind::ModifySpeed {
                target: ItemTarget::Adjacent(source),
                basis_points: 400,
            },
        )],
        ItemState::Shield => vec![Effect::new(
            source,
            EffectKind::GainBlock {
                target: source.side,
                amount: 8,
                maximum: 24,
            },
        )],
        ItemState::Leech => vec![
            Effect::new(
                source,
                EffectKind::LoseHealth {
                    target: source.side.opponent(),
                    amount: 1,
                },
            ),
            Effect::new(
                source,
                EffectKind::Heal {
                    target: source.side,
                    amount: 1,
                },
            ),
        ],
        ItemState::PoisonVial => vec![Effect::new(
            source,
            EffectKind::LoseHealth {
                target: source.side.opponent(),
                amount: 2,
            },
        )],
        ItemState::GrapplingHook { uses_left } if *uses_left > 0 => {
            *uses_left -= 1;
            vec![Effect::new(
                source,
                EffectKind::AttemptFall {
                    target: ItemTarget::LightestEdge {
                        side: source.side.opponent(),
                        weapons_only: false,
                    },
                    cause: FallCause::Forced { source },
                    chance: None,
                },
            )]
        }
        ItemState::Windbreaker => vec![damage(source, 6, DamageMode::Piercing)],
        ItemState::Spear => vec![damage(source, 16, DamageMode::Normal)],
        ItemState::Dagger => vec![damage(source, 4, DamageMode::Normal)],
        ItemState::Warhammer => vec![damage(source, 18, DamageMode::Normal)],
        ItemState::Rapier => vec![damage(source, 6, DamageMode::Piercing)],
        ItemState::SpikedTarge => vec![Effect::new(
            source,
            EffectKind::GainBlock {
                target: source.side,
                amount: 3,
                maximum: 12,
            },
        )],
        ItemState::DoomCandle => vec![Effect::new(
            source,
            EffectKind::LoseHealth {
                target: source.side.opponent(),
                amount: 3,
            },
        )],
        ItemState::BloodChalice => vec![
            Effect::new(
                source,
                EffectKind::LoseHealth {
                    target: source.side.opponent(),
                    amount: 3,
                },
            ),
            Effect::new(
                source,
                EffectKind::Heal {
                    target: source.side,
                    amount: 1,
                },
            ),
        ],
        ItemState::SignalDrum => vec![Effect::new(
            source,
            EffectKind::ModifySpeed {
                target: ItemTarget::Adjacent(source),
                basis_points: 800,
            },
        )],
        ItemState::ShrapnelMine => vec![
            damage(source, 10, DamageMode::Piercing),
            Effect::new(
                source,
                EffectKind::AttemptFall {
                    target: ItemTarget::LightestEdge {
                        side: source.side.opponent(),
                        weapons_only: true,
                    },
                    cause: FallCause::Forced { source },
                    chance: Some(3),
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::BearTrap => vec![
            damage(source, 12, DamageMode::Piercing),
            Effect::new(
                source,
                EffectKind::AttemptFall {
                    target: ItemTarget::LightestEdge {
                        side: source.side.opponent(),
                        weapons_only: false,
                    },
                    cause: FallCause::Forced { source },
                    chance: None,
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::Grimoire {
            damage: current_damage,
        } => {
            let effect = damage(source, *current_damage, DamageMode::Normal);
            *current_damage = current_damage.saturating_add(5);
            vec![effect]
        }
        ItemState::Bomb => vec![
            damage(source, 10, DamageMode::Piercing),
            Effect::new(
                source,
                EffectKind::AttemptFall {
                    target: ItemTarget::LightestEdge {
                        side: source.side.opponent(),
                        weapons_only: true,
                    },
                    cause: FallCause::Forced { source },
                    chance: Some(2),
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::Caltrops => vec![Effect::new(
            source,
            EffectKind::AttemptFall {
                target: ItemTarget::LightestEdge {
                    side: source.side.opponent(),
                    weapons_only: false,
                },
                cause: FallCause::Forced { source },
                chance: None,
            },
        )],
        ItemState::Whetstone
        | ItemState::LooseBuckler
        | ItemState::Armor
        | ItemState::Cactus
        | ItemState::HealingPotion
        | ItemState::GrapplingHook { .. }
        | ItemState::Net { .. }
        | ItemState::Strap
        | ItemState::WarBanner
        | ItemState::Thornmail
        | ItemState::Pavise
        | ItemState::Tripwire
        | ItemState::FieldKit
        | ItemState::BarricadeKit => Vec::new(),
    };
    item.schedule_next();
    effects
}

const fn damage(source: ItemRef, amount: u16, mode: DamageMode) -> Effect {
    Effect::new(
        source,
        EffectKind::Damage {
            target: source.side.opponent(),
            amount,
            mode,
        },
    )
}
