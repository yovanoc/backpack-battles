use crate::Offset;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivationTiming {
    pub initial: u16,
    pub recurring: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NaturalFall {
    pub every: u16,
    pub one_in: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct Definition {
    pub(crate) name: &'static str,
    pub(crate) shape: &'static [Offset],
    pub(crate) activation: Option<ActivationTiming>,
    pub(crate) weapon: bool,
    pub(crate) weight: u16,
    pub(crate) can_fall: bool,
    pub(crate) armor: u16,
    pub(crate) max_health: u16,
    pub(crate) adjacent_damage: u16,
    pub(crate) retaliation: u16,
    pub(crate) natural_fall: Option<NaturalFall>,
}

impl Definition {
    pub(crate) const fn new(name: &'static str, shape: &'static [Offset]) -> Self {
        Self {
            name,
            shape,
            activation: None,
            weapon: false,
            weight: 1,
            can_fall: true,
            armor: 0,
            max_health: 0,
            adjacent_damage: 0,
            retaliation: 0,
            natural_fall: None,
        }
    }

    pub(crate) const fn activation(mut self, initial: u16, recurring: u16) -> Self {
        self.activation = Some(ActivationTiming { initial, recurring });
        self
    }

    pub(crate) const fn weapon(mut self, weight: u16) -> Self {
        self.weapon = true;
        self.weight = weight;
        self
    }

    pub(crate) const fn weight(mut self, weight: u16) -> Self {
        self.weight = weight;
        self
    }

    pub(crate) const fn fixed(mut self) -> Self {
        self.can_fall = false;
        self
    }

    pub(crate) const fn armor(mut self, armor: u16) -> Self {
        self.armor = armor;
        self
    }

    pub(crate) const fn max_health(mut self, max_health: u16) -> Self {
        self.max_health = max_health;
        self
    }

    pub(crate) const fn adjacent_damage(mut self, adjacent_damage: u16) -> Self {
        self.adjacent_damage = adjacent_damage;
        self
    }

    pub(crate) const fn retaliation(mut self, retaliation: u16) -> Self {
        self.retaliation = retaliation;
        self
    }

    pub(crate) const fn natural_fall(mut self, every: u16, one_in: u64) -> Self {
        self.natural_fall = Some(NaturalFall { every, one_in });
        self
    }
}
