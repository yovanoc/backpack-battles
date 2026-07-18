#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Archetype {
    Aggression,
    Defense,
    Scaling,
    Control,
    Support,
}

impl Archetype {
    pub const ALL: [Self; 5] = [
        Self::Aggression,
        Self::Defense,
        Self::Scaling,
        Self::Control,
        Self::Support,
    ];
    pub const COUNT: usize = Self::ALL.len();

    pub const fn name(self) -> &'static str {
        match self {
            Self::Aggression => "aggression",
            Self::Defense => "defense",
            Self::Scaling => "scaling",
            Self::Control => "control",
            Self::Support => "support",
        }
    }
}
