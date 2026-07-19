use crate::{Effect, EffectKind, FallCause, Item, ItemRef, ItemState, ItemTarget};

mod activate;
pub(crate) use activate::activate;

pub(crate) fn battle_start(item: &mut Item, source: ItemRef) -> Vec<Effect> {
    match item.state_mut() {
        ItemState::Net { used } if !*used => {
            *used = true;
            vec![Effect::new(
                source,
                EffectKind::AttemptFall {
                    target: ItemTarget::LightestEdge {
                        side: source.side.opponent(),
                        weapons_only: true,
                    },
                    cause: FallCause::Forced { source },
                    chance: None,
                },
            )]
        }
        ItemState::Tripwire => vec![
            Effect::new(
                source,
                EffectKind::AttemptFall {
                    target: ItemTarget::LightestEdge {
                        side: source.side.opponent(),
                        weapons_only: false,
                    },
                    cause: FallCause::Forced { source },
                    chance: Some(2),
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::BarricadeKit => vec![
            Effect::new(
                source,
                EffectKind::GainBlock {
                    target: source.side,
                    amount: 10,
                    maximum: 18,
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::Bola => vec![
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
        ItemState::WoodenSword
        | ItemState::Crossbow
        | ItemState::Whetstone
        | ItemState::Hourglass
        | ItemState::LooseBuckler
        | ItemState::Armor
        | ItemState::Shield
        | ItemState::Cactus
        | ItemState::Leech
        | ItemState::HealingPotion
        | ItemState::GrapplingHook { .. }
        | ItemState::Net { .. }
        | ItemState::Strap
        | ItemState::Windbreaker
        | ItemState::Spear
        | ItemState::WarBanner
        | ItemState::Bomb
        | ItemState::Caltrops
        | ItemState::Thornmail
        | ItemState::Dagger
        | ItemState::Warhammer
        | ItemState::Grimoire { .. }
        | ItemState::PoisonVial
        | ItemState::Rapier
        | ItemState::SpikedTarge
        | ItemState::Pavise
        | ItemState::DoomCandle
        | ItemState::BloodChalice
        | ItemState::ShrapnelMine
        | ItemState::BearTrap
        | ItemState::SignalDrum
        | ItemState::FieldKit
        | ItemState::Katana
        | ItemState::ThrowingAxe
        | ItemState::MorningStar
        | ItemState::Chainmail
        | ItemState::Bulwark
        | ItemState::VenomFang
        | ItemState::Spellbook { .. }
        | ItemState::VampiricBlade
        | ItemState::PlagueCenser
        | ItemState::Grenade
        | ItemState::HarpoonGun { .. }
        | ItemState::Grindstone
        | ItemState::Metronome
        | ItemState::MedicBag
        | ItemState::RallyingHorn => Vec::new(),
    }
}

pub(crate) fn tick(item: &mut Item, source: ItemRef, health: u16, max_health: u16) -> Vec<Effect> {
    match item.state_mut() {
        // ponytail: health*2 saturates only above u16::MAX/2 HP, unreachable in
        // any real battle; the guard fires correctly across the live HP range.
        ItemState::HealingPotion if health.saturating_mul(2) <= max_health => vec![
            Effect::new(
                source,
                EffectKind::Heal {
                    target: source.side,
                    amount: 16,
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::FieldKit if health.saturating_mul(2) <= max_health => vec![
            Effect::new(
                source,
                EffectKind::Heal {
                    target: source.side,
                    amount: 15,
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::MedicBag if health.saturating_mul(2) <= max_health => vec![
            Effect::new(
                source,
                EffectKind::Heal {
                    target: source.side,
                    amount: 30,
                },
            ),
            Effect::new(source, EffectKind::Consume { target: source }),
        ],
        ItemState::WoodenSword
        | ItemState::Crossbow
        | ItemState::Whetstone
        | ItemState::Hourglass
        | ItemState::LooseBuckler
        | ItemState::Armor
        | ItemState::Shield
        | ItemState::Cactus
        | ItemState::Leech
        | ItemState::HealingPotion
        | ItemState::GrapplingHook { .. }
        | ItemState::Net { .. }
        | ItemState::Strap
        | ItemState::Windbreaker
        | ItemState::Spear
        | ItemState::WarBanner
        | ItemState::Bomb
        | ItemState::Caltrops
        | ItemState::Thornmail
        | ItemState::Dagger
        | ItemState::Warhammer
        | ItemState::Grimoire { .. }
        | ItemState::PoisonVial
        | ItemState::Rapier
        | ItemState::SpikedTarge
        | ItemState::Pavise
        | ItemState::DoomCandle
        | ItemState::BloodChalice
        | ItemState::Tripwire
        | ItemState::ShrapnelMine
        | ItemState::BearTrap
        | ItemState::SignalDrum
        | ItemState::FieldKit
        | ItemState::BarricadeKit
        | ItemState::Katana
        | ItemState::ThrowingAxe
        | ItemState::MorningStar
        | ItemState::Chainmail
        | ItemState::Bulwark
        | ItemState::VenomFang
        | ItemState::Spellbook { .. }
        | ItemState::VampiricBlade
        | ItemState::PlagueCenser
        | ItemState::Bola
        | ItemState::Grenade
        | ItemState::HarpoonGun { .. }
        | ItemState::Grindstone
        | ItemState::Metronome
        | ItemState::RallyingHorn
        | ItemState::MedicBag => Vec::new(),
    }
}
