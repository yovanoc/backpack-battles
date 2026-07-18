/// Deterministic splitmix64 stream. The same seed always yields the same
/// sequence, which is what makes battles and generated bags replayable.
#[derive(Debug)]
pub(crate) struct Rng {
    state: u64,
}

impl Rng {
    pub(crate) const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    pub(crate) fn one_in(&mut self, denominator: u64) -> bool {
        self.next_u64().is_multiple_of(denominator)
    }

    // ponytail: modulo bias is negligible for balancing over a tiny domain.
    pub(crate) fn below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        self.next_u64() % bound
    }

    pub(crate) fn choice<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        let bound = u64::try_from(items.len()).unwrap_or(1).max(1);
        let index = usize::try_from(self.below(bound)).unwrap_or(0);
        &items[index]
    }
}
