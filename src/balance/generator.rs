use super::CampaignMode;
use crate::{
    Archetype, Bag, Cell, Item, ItemKind, Offset, Rotation,
    geometry::{BAG_CELLS, BAG_HEIGHT, BAG_WIDTH},
    rng::Rng,
};

const PLACEMENT_ATTEMPTS: usize = 40;
const ELITE_SAMPLES: usize = 16;

pub(crate) fn random_bag(rng: &mut Rng, allow_rotation: bool) -> Bag {
    generated_bag(rng, allow_rotation, CampaignMode::Random, 0)
}

pub(super) fn generated_bag(
    rng: &mut Rng,
    allow_rotation: bool,
    mode: CampaignMode,
    bag_index: u64,
) -> Bag {
    if matches!(mode, CampaignMode::Elite) {
        // ponytail: best-of-16 sampler, not a placement optimizer. Surfaces the
        // well-arranged (elite) case for adjacency items; swap for a real search
        // only if 16 samples miss the ceiling.
        let mut best = sample_bag(rng, allow_rotation, mode, bag_index);
        let mut best_score = adjacency_score(&best);
        for _ in 1..ELITE_SAMPLES {
            let candidate = sample_bag(rng, allow_rotation, mode, bag_index);
            let score = adjacency_score(&candidate);
            if score > best_score {
                best = candidate;
                best_score = score;
            }
        }
        return best;
    }
    sample_bag(rng, allow_rotation, mode, bag_index)
}

fn sample_bag(rng: &mut Rng, allow_rotation: bool, mode: CampaignMode, bag_index: u64) -> Bag {
    let mut occupied = [false; BAG_CELLS];
    let mut copies = [0_u8; ItemKind::COUNT];
    let mut items = Vec::new();
    for _ in 0..PLACEMENT_ATTEMPTS {
        if !ItemKind::ALL
            .iter()
            .any(|kind| copies[*kind as usize] < 2 && allowed(kind.archetype(), mode, bag_index))
        {
            break;
        }
        let kind = loop {
            let kind = *rng.choice(&ItemKind::ALL);
            if copies[kind as usize] < 2 && allowed(kind.archetype(), mode, bag_index) {
                break kind;
            }
        };
        let rotation = if allow_rotation {
            *rng.choice(&Rotation::ALL)
        } else {
            Rotation::Deg0
        };
        let anchor = Cell::new(
            u8::try_from(rng.below(u64::from(BAG_WIDTH))).unwrap_or(0),
            u8::try_from(rng.below(u64::from(BAG_HEIGHT))).unwrap_or(0),
        );
        let candidate = Item::with_rotation(kind, anchor, rotation);
        if place(&candidate, &mut occupied) {
            copies[kind as usize] += 1;
            items.push(candidate);
        }
    }
    Bag::new(items).expect("generator only places valid, non-overlapping items")
}

/// Realized adjacency payoff: for each buff item, how many adjacent items it
/// helps, weighted by the strength of the help. Best-of-N picks the arrangement
/// that lands the most total synergy weight.
///
///   Whetstone / War Banner -> adjacent weapons (weight = its damage buff: 2, 7)
///   Hourglass / Signal Drum -> adjacent activating items (speed buff)
///   Strap                   -> adjacent fall-prone items (fall protection)
fn adjacency_score(bag: &Bag) -> u16 {
    bag.items()
        .iter()
        .map(|item| realized_synergies(bag, item))
        .sum()
}

fn realized_synergies(bag: &Bag, item: &Item) -> u16 {
    let kind = item.kind();
    let qualifies: fn(ItemKind) -> bool = match kind {
        ItemKind::Whetstone
        | ItemKind::WarBanner
        | ItemKind::Grindstone
        | ItemKind::RallyingHorn => ItemKind::is_weapon,
        ItemKind::Hourglass | ItemKind::SignalDrum | ItemKind::Metronome => {
            |neighbor| neighbor.activation().is_some()
        }
        ItemKind::Strap => ItemKind::can_fall,
        _ => return 0,
    };
    let targets = bag
        .adjacent_ids(item.id())
        .filter_map(|id| bag.item(id))
        .filter(|neighbor| qualifies(neighbor.kind()))
        .count();
    let count = u16::try_from(targets).unwrap_or(u16::MAX);
    count.saturating_mul(synergy_weight(kind))
}

// ponytail: hand-tuned points on a shared scale, not real value. Damage buffs
// use their own numbers (2, 7); speed uses bps/100; Strap is a flat situational
// 3. Retune if a synergy lands too often or too rarely in elite bags.
fn synergy_weight(kind: ItemKind) -> u16 {
    match kind {
        ItemKind::Whetstone
        | ItemKind::WarBanner
        | ItemKind::Grindstone
        | ItemKind::RallyingHorn => kind.adjacent_damage(),
        ItemKind::Hourglass => 4,
        ItemKind::SignalDrum => 8,
        ItemKind::Metronome => 6,
        ItemKind::Strap => 3,
        _ => 0,
    }
}

fn allowed(archetype: Archetype, mode: CampaignMode, bag_index: u64) -> bool {
    let primary = usize::try_from(bag_index % Archetype::COUNT as u64).unwrap_or(0);
    match mode {
        CampaignMode::Random | CampaignMode::Elite => true,
        CampaignMode::Pure => archetype as usize == primary,
        CampaignMode::Hybrid => {
            let offset =
                usize::try_from((bag_index / Archetype::COUNT as u64) % 4).unwrap_or(0) + 1;
            let secondary = (primary + offset) % Archetype::COUNT;
            archetype as usize == primary || archetype as usize == secondary
        }
    }
}

fn place(item: &Item, occupied: &mut [bool; BAG_CELLS]) -> bool {
    for offset in item.shape() {
        match cell_index(item.position(), *offset) {
            Some(index) if !occupied[index] => {}
            _ => return false,
        }
    }
    for offset in item.shape() {
        if let Some(index) = cell_index(item.position(), *offset) {
            occupied[index] = true;
        }
    }
    true
}

fn cell_index(position: Cell, offset: Offset) -> Option<usize> {
    let x = position.x.checked_add(offset.x)?;
    let y = position.y.checked_add(offset.y)?;
    if x >= BAG_WIDTH || y >= BAG_HEIGHT {
        return None;
    }
    Some(usize::from(y) * usize::from(BAG_WIDTH) + usize::from(x))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_bags_have_at_most_two_copies_per_kind() {
        for seed in 0..100 {
            let bag = random_bag(&mut Rng::new(seed), true);
            let mut copies = [0_u8; ItemKind::COUNT];
            for item in bag.items() {
                copies[item.kind() as usize] += 1;
            }
            assert!(copies.into_iter().all(|count| count <= 2));
        }
    }

    #[test]
    fn elite_bags_beat_random_on_adjacency() {
        let mut elite_total = 0_u32;
        let mut random_total = 0_u32;
        for seed in 0..64 {
            let elite = generated_bag(&mut Rng::new(seed), true, CampaignMode::Elite, seed);
            let random = generated_bag(&mut Rng::new(seed), true, CampaignMode::Random, seed);
            elite_total += u32::from(adjacency_score(&elite));
            random_total += u32::from(adjacency_score(&random));
        }
        assert!(
            elite_total > random_total,
            "elite arrangement should realize more adjacency than random: {elite_total} vs {random_total}"
        );
    }
}
