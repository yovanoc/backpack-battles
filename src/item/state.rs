use crate::ItemKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ItemState {
    WoodenSword,
    Crossbow,
    Whetstone,
    Hourglass,
    LooseBuckler,
    Armor,
    Shield,
    Cactus,
    Leech,
    HealingPotion,
    GrapplingHook { uses_left: u8 },
    Net { used: bool },
    Strap,
    Windbreaker,
    Spear,
    WarBanner,
    Bomb,
    Caltrops,
    Thornmail,
    Dagger,
    Warhammer,
    Grimoire { damage: u16 },
    PoisonVial,
    Rapier,
    SpikedTarge,
    Pavise,
    DoomCandle,
    BloodChalice,
    Tripwire,
    ShrapnelMine,
    BearTrap,
    SignalDrum,
    FieldKit,
    BarricadeKit,
}

impl ItemState {
    pub(super) const fn new(kind: ItemKind) -> Self {
        match kind {
            ItemKind::WoodenSword => Self::WoodenSword,
            ItemKind::Crossbow => Self::Crossbow,
            ItemKind::Whetstone => Self::Whetstone,
            ItemKind::Hourglass => Self::Hourglass,
            ItemKind::LooseBuckler => Self::LooseBuckler,
            ItemKind::Armor => Self::Armor,
            ItemKind::Shield => Self::Shield,
            ItemKind::Cactus => Self::Cactus,
            ItemKind::Leech => Self::Leech,
            ItemKind::HealingPotion => Self::HealingPotion,
            ItemKind::GrapplingHook => Self::GrapplingHook { uses_left: 2 },
            ItemKind::Net => Self::Net { used: false },
            ItemKind::Strap => Self::Strap,
            ItemKind::Windbreaker => Self::Windbreaker,
            ItemKind::Spear => Self::Spear,
            ItemKind::WarBanner => Self::WarBanner,
            ItemKind::Bomb => Self::Bomb,
            ItemKind::Caltrops => Self::Caltrops,
            ItemKind::Thornmail => Self::Thornmail,
            ItemKind::Dagger => Self::Dagger,
            ItemKind::Warhammer => Self::Warhammer,
            ItemKind::Grimoire => Self::Grimoire { damage: 10 },
            ItemKind::PoisonVial => Self::PoisonVial,
            ItemKind::Rapier => Self::Rapier,
            ItemKind::SpikedTarge => Self::SpikedTarge,
            ItemKind::Pavise => Self::Pavise,
            ItemKind::DoomCandle => Self::DoomCandle,
            ItemKind::BloodChalice => Self::BloodChalice,
            ItemKind::Tripwire => Self::Tripwire,
            ItemKind::ShrapnelMine => Self::ShrapnelMine,
            ItemKind::BearTrap => Self::BearTrap,
            ItemKind::SignalDrum => Self::SignalDrum,
            ItemKind::FieldKit => Self::FieldKit,
            ItemKind::BarricadeKit => Self::BarricadeKit,
        }
    }

    pub(crate) const fn kind(&self) -> ItemKind {
        match self {
            Self::WoodenSword => ItemKind::WoodenSword,
            Self::Crossbow => ItemKind::Crossbow,
            Self::Whetstone => ItemKind::Whetstone,
            Self::Hourglass => ItemKind::Hourglass,
            Self::LooseBuckler => ItemKind::LooseBuckler,
            Self::Armor => ItemKind::Armor,
            Self::Shield => ItemKind::Shield,
            Self::Cactus => ItemKind::Cactus,
            Self::Leech => ItemKind::Leech,
            Self::HealingPotion => ItemKind::HealingPotion,
            Self::GrapplingHook { .. } => ItemKind::GrapplingHook,
            Self::Net { .. } => ItemKind::Net,
            Self::Strap => ItemKind::Strap,
            Self::Windbreaker => ItemKind::Windbreaker,
            Self::Spear => ItemKind::Spear,
            Self::WarBanner => ItemKind::WarBanner,
            Self::Bomb => ItemKind::Bomb,
            Self::Caltrops => ItemKind::Caltrops,
            Self::Thornmail => ItemKind::Thornmail,
            Self::Dagger => ItemKind::Dagger,
            Self::Warhammer => ItemKind::Warhammer,
            Self::Grimoire { .. } => ItemKind::Grimoire,
            Self::PoisonVial => ItemKind::PoisonVial,
            Self::Rapier => ItemKind::Rapier,
            Self::SpikedTarge => ItemKind::SpikedTarge,
            Self::Pavise => ItemKind::Pavise,
            Self::DoomCandle => ItemKind::DoomCandle,
            Self::BloodChalice => ItemKind::BloodChalice,
            Self::Tripwire => ItemKind::Tripwire,
            Self::ShrapnelMine => ItemKind::ShrapnelMine,
            Self::BearTrap => ItemKind::BearTrap,
            Self::SignalDrum => ItemKind::SignalDrum,
            Self::FieldKit => ItemKind::FieldKit,
            Self::BarricadeKit => ItemKind::BarricadeKit,
        }
    }

    pub(super) const fn can_activate_again(&self) -> bool {
        match self {
            Self::GrapplingHook { uses_left } => *uses_left > 0,
            Self::WoodenSword
            | Self::Crossbow
            | Self::Hourglass
            | Self::Shield
            | Self::Leech
            | Self::Windbreaker
            | Self::Spear
            | Self::Bomb
            | Self::Caltrops
            | Self::Dagger
            | Self::Warhammer
            | Self::Grimoire { .. }
            | Self::PoisonVial
            | Self::Rapier
            | Self::SpikedTarge
            | Self::DoomCandle
            | Self::BloodChalice
            | Self::ShrapnelMine
            | Self::BearTrap
            | Self::SignalDrum => true,
            Self::Whetstone
            | Self::LooseBuckler
            | Self::Armor
            | Self::Cactus
            | Self::HealingPotion
            | Self::Net { .. }
            | Self::Strap
            | Self::WarBanner
            | Self::Thornmail
            | Self::Pavise
            | Self::Tripwire
            | Self::FieldKit
            | Self::BarricadeKit => false,
        }
    }
}
