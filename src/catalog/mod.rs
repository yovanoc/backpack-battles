use crate::{Archetype, Offset, item_shapes::*};

mod definition;
use definition::{ActivationTiming, Definition, NaturalFall};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemKind {
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
    GrapplingHook,
    Net,
    Strap,
    Windbreaker,
    Spear,
    WarBanner,
    Bomb,
    Caltrops,
    Thornmail,
    Dagger,
    Warhammer,
    Grimoire,
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

impl ItemKind {
    pub const fn name(self) -> &'static str {
        self.definition().name
    }

    pub const fn shape(self) -> &'static [Offset] {
        self.definition().shape
    }

    pub const fn archetype(self) -> Archetype {
        match self {
            Self::WoodenSword
            | Self::Crossbow
            | Self::Windbreaker
            | Self::Spear
            | Self::Dagger
            | Self::Rapier
            | Self::Warhammer => Archetype::Aggression,
            Self::LooseBuckler
            | Self::Armor
            | Self::Shield
            | Self::Cactus
            | Self::HealingPotion
            | Self::Thornmail
            | Self::SpikedTarge
            | Self::Pavise => Archetype::Defense,
            Self::Hourglass
            | Self::Leech
            | Self::Grimoire
            | Self::PoisonVial
            | Self::DoomCandle
            | Self::BloodChalice => Archetype::Scaling,
            Self::GrapplingHook
            | Self::Net
            | Self::Bomb
            | Self::Caltrops
            | Self::Tripwire
            | Self::ShrapnelMine
            | Self::BearTrap => Archetype::Control,
            Self::Whetstone
            | Self::Strap
            | Self::WarBanner
            | Self::SignalDrum
            | Self::FieldKit
            | Self::BarricadeKit => Archetype::Support,
        }
    }

    pub(crate) const fn activation(self) -> Option<ActivationTiming> {
        self.definition().activation
    }

    pub(crate) const fn is_weapon(self) -> bool {
        self.definition().weapon
    }

    pub(crate) const fn weight(self) -> u16 {
        self.definition().weight
    }

    pub(crate) const fn can_fall(self) -> bool {
        self.definition().can_fall
    }

    pub(crate) const fn armor(self) -> u16 {
        self.definition().armor
    }

    pub(crate) const fn max_health(self) -> u16 {
        self.definition().max_health
    }

    pub(crate) const fn adjacent_damage(self) -> u16 {
        self.definition().adjacent_damage
    }

    pub(crate) const fn retaliation(self) -> u16 {
        self.definition().retaliation
    }

    pub(crate) const fn natural_fall(self) -> Option<NaturalFall> {
        self.definition().natural_fall
    }

    const fn definition(self) -> Definition {
        match self {
            Self::WoodenSword => Definition::new("Wooden Sword", &VERTICAL_TWO)
                .activation(20, 20)
                .weapon(3),
            Self::Crossbow => Definition::new("Crossbow", &HORIZONTAL_TWO)
                .activation(15, 30)
                .weapon(4),
            Self::Whetstone => Definition::new("Whetstone", &ONE).adjacent_damage(2),
            Self::Hourglass => Definition::new("Hourglass", &ONE).activation(20, 20),
            Self::LooseBuckler => Definition::new("Loose Buckler", &SQUARE)
                .armor(2)
                .max_health(30)
                .natural_fall(20, 8),
            Self::Armor => Definition::new("Armor", &SQUARE)
                .armor(2)
                .max_health(40)
                .weight(10)
                .fixed(),
            Self::Shield => Definition::new("Shield", &VERTICAL_TWO).activation(25, 25),
            Self::Cactus => Definition::new("Potted Cactus", &ONE).retaliation(2),
            Self::Leech => Definition::new("Leech", &ONE).activation(10, 10),
            Self::HealingPotion => Definition::new("Healing Potion", &ONE),
            Self::GrapplingHook => {
                Definition::new("Grappling Hook", &VERTICAL_TWO).activation(60, 60)
            }
            Self::Net => Definition::new("Weighted Net", &ONE),
            Self::Strap => Definition::new("Strap", &ONE),
            Self::Windbreaker => Definition::new("Windbreaker", &ONE)
                .activation(25, 25)
                .weapon(2),
            Self::Spear => Definition::new("Spear", &L_TETROMINO)
                .activation(30, 30)
                .weapon(5),
            Self::WarBanner => Definition::new("War Banner", &LINE_THREE)
                .adjacent_damage(7)
                .max_health(10),
            Self::Bomb => Definition::new("Bomb", &HORIZONTAL_TWO).activation(30, 30),
            Self::Caltrops => Definition::new("Caltrops", &LINE_THREE).activation(30, 30),
            Self::Thornmail => Definition::new("Thornmail", &L_TROMINO)
                .armor(1)
                .retaliation(3),
            Self::Dagger => Definition::new("Dagger", &ONE).activation(5, 13).weapon(1),
            Self::Warhammer => Definition::new("Warhammer", &SQUARE)
                .activation(37, 37)
                .weapon(10)
                .fixed(),
            Self::Grimoire => Definition::new("Grimoire", &SQUARE)
                .activation(30, 43)
                .weight(10)
                .fixed(),
            Self::PoisonVial => Definition::new("Poison Vial", &ONE).activation(15, 15),
            Self::Rapier => Definition::new("Rapier", &VERTICAL_TWO)
                .activation(26, 26)
                .weapon(3),
            Self::SpikedTarge => Definition::new("Spiked Targe", &ONE)
                .activation(20, 20)
                .retaliation(1),
            Self::Pavise => Definition::new("Pavise", &LINE_THREE)
                .armor(1)
                .max_health(24)
                .weight(8)
                .fixed(),
            Self::DoomCandle => Definition::new("Doom Candle", &ONE).activation(45, 20),
            Self::BloodChalice => {
                Definition::new("Blood Chalice", &VERTICAL_TWO).activation(35, 25)
            }
            Self::Tripwire => Definition::new("Tripwire", &ONE),
            Self::ShrapnelMine => {
                Definition::new("Shrapnel Mine", &HORIZONTAL_TWO).activation(20, 20)
            }
            Self::BearTrap => Definition::new("Bear Trap", &L_TROMINO).activation(24, 35),
            Self::SignalDrum => Definition::new("Signal Drum", &VERTICAL_TWO).activation(15, 30),
            Self::FieldKit => Definition::new("Field Kit", &ONE),
            Self::BarricadeKit => Definition::new("Barricade Kit", &HORIZONTAL_TWO),
        }
    }
}

impl ItemKind {
    pub const ALL: [Self; 34] = [
        Self::WoodenSword,
        Self::Crossbow,
        Self::Whetstone,
        Self::Hourglass,
        Self::LooseBuckler,
        Self::Armor,
        Self::Shield,
        Self::Cactus,
        Self::Leech,
        Self::HealingPotion,
        Self::GrapplingHook,
        Self::Net,
        Self::Strap,
        Self::Windbreaker,
        Self::Spear,
        Self::WarBanner,
        Self::Bomb,
        Self::Caltrops,
        Self::Thornmail,
        Self::Dagger,
        Self::Warhammer,
        Self::Grimoire,
        Self::PoisonVial,
        Self::Rapier,
        Self::SpikedTarge,
        Self::Pavise,
        Self::DoomCandle,
        Self::BloodChalice,
        Self::Tripwire,
        Self::ShrapnelMine,
        Self::BearTrap,
        Self::SignalDrum,
        Self::FieldKit,
        Self::BarricadeKit,
    ];
    pub const COUNT: usize = Self::ALL.len();
}
