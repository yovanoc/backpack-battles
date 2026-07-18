use crate::ItemKind;

impl ItemKind {
    pub const fn first_activation(self) -> Option<u16> {
        match self.activation() {
            Some(timing) => Some(timing.initial),
            None => None,
        }
    }

    pub const fn cadence(self) -> Option<u16> {
        match self.activation() {
            Some(timing) => Some(timing.recurring),
            None => None,
        }
    }

    pub const fn effect_description(self) -> &'static str {
        match self {
            Self::WoodenSword => "8 normal damage",
            Self::Crossbow => "10 normal damage",
            Self::Whetstone => "adjacent weapons +2 damage",
            Self::Hourglass => "adjacent items +400 speed bps",
            Self::LooseBuckler => "+30 health, +2 armor; 1/8 fall every 20 ticks",
            Self::Armor => "+40 health, +2 armor; cannot fall",
            Self::Shield => "+8 block, maximum 24",
            Self::Cactus => "retaliate for 2 damage when hit",
            Self::Leech => "drain 1 health and heal 1",
            Self::HealingPotion => "heal 25 below half health, then consume",
            Self::GrapplingHook => "force lightest edge item to fall; 2 uses",
            Self::Net => "force lightest edge weapon to fall at battle start",
            Self::Strap => "protect adjacent items from falling",
            Self::Windbreaker => "6 piercing damage",
            Self::Spear => "16 normal damage",
            Self::WarBanner => "+10 health; adjacent weapons +7 damage",
            Self::Bomb => "10 piercing damage, then 1/2 drop lightest edge weapon; consumed",
            Self::Caltrops => "drop lightest edge item",
            Self::Thornmail => "+1 armor; retaliate for 3 damage",
            Self::Dagger => "4 normal damage",
            Self::Warhammer => "18 normal damage; cannot fall",
            Self::Grimoire => "10 normal damage, +5 each activation; cannot fall",
            Self::PoisonVial => "enemy loses 2 health",
            Self::Rapier => "6 piercing damage",
            Self::SpikedTarge => "+3 block, maximum 12; retaliate for 1 damage",
            Self::Pavise => "+24 health, +1 armor; cannot fall",
            Self::DoomCandle => "enemy loses 3 health",
            Self::BloodChalice => "enemy loses 3 health and heal 1",
            Self::Tripwire => "1/2 drop lightest edge item at battle start; consumed",
            Self::ShrapnelMine => {
                "10 piercing damage, then 1/3 drop lightest edge weapon; consumed"
            }
            Self::BearTrap => "12 piercing damage, then drop lightest edge item; consumed",
            Self::SignalDrum => "adjacent items +800 speed bps",
            Self::FieldKit => "heal 15 below half health, then consume",
            Self::BarricadeKit => "+10 block, maximum 18 at battle start; consumed",
        }
    }
}
