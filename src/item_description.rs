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
            Self::LooseBuckler => "+20 health, +1 armor; 1/8 fall every 20 ticks",
            Self::Armor => "+28 health, +1 armor; cannot fall",
            Self::Shield => "+6 block, maximum 20; cleanse 3 poison",
            Self::Cactus => "retaliate for 2 damage when hit",
            Self::Leech => "drain 1 health and heal 1",
            Self::HealingPotion => "heal 16 below half health, then consume",
            Self::GrapplingHook => "force lightest edge item to fall; 2 uses",
            Self::Net => "force lightest edge weapon to fall at battle start",
            Self::Strap => "protect adjacent items from falling",
            Self::Windbreaker => "6 piercing damage",
            Self::Spear => "16 normal damage",
            Self::WarBanner => "+10 health; adjacent weapons +7 damage",
            Self::Bomb => "10 piercing damage, then 1/2 drop lightest edge weapon; consumed",
            Self::Caltrops => "2 normal damage; delay enemy's next activation by 6 ticks",
            Self::Thornmail => "+1 armor; retaliate for 3 damage",
            Self::Dagger => "4 normal damage",
            Self::Warhammer => "18 normal damage; cannot fall",
            Self::Grimoire => "10 normal damage, +5 each activation; cannot fall",
            Self::PoisonVial => "enemy loses 2 health",
            Self::Rapier => "6 piercing damage",
            Self::SpikedTarge => "+3 block, maximum 12; retaliate for 1 damage; cleanse 2 poison",
            Self::Pavise => "+16 health, +1 armor; cannot fall",
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
            Self::Katana => "9 normal damage",
            Self::ThrowingAxe => "11 piercing damage",
            Self::MorningStar => "14 normal damage",
            Self::Chainmail => "+14 health, +3 armor; cannot fall",
            Self::Bulwark => "+10 block, maximum 30; cleanse 4 poison",
            Self::VenomFang => "apply 2 poison",
            Self::Spellbook => "6 normal damage, +4 each activation; cannot fall",
            Self::VampiricBlade => "enemy loses 4 health and heal 2",
            Self::PlagueCenser => "apply 4 poison",
            Self::Bola => "force lightest edge item to fall at battle start; consumed",
            Self::Grenade => "8 piercing damage, then 1/3 drop lightest edge item; consumed",
            Self::HarpoonGun => "force lightest edge item to fall; 3 uses",
            Self::Grindstone => "adjacent weapons +4 damage",
            Self::Metronome => "adjacent items +600 speed bps",
            Self::MedicBag => "heal 30 below half health, then consume",
            Self::RallyingHorn => "+8 health; adjacent weapons +4 damage",
        }
    }
}
