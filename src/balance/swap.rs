use super::{BalanceConfig, ItemStat, Tally, distinct_kinds};
use crate::{
    Bag, BattleConfig, Hero, Item, ItemKind, Offset, Outcome, Rotation, rng::Rng, simulate,
};

pub(super) fn run(
    config: &BalanceConfig,
    rng: &mut Rng,
    left_bag: &Bag,
    right_bag: &Bag,
    battle_config: BattleConfig,
    original_tally: Tally,
    stats: &mut [ItemStat],
) {
    let items = left_bag.items();
    if items.is_empty() {
        return;
    }
    let kinds = distinct_kinds(left_bag);
    let selected_kind = *rng.choice(&kinds);
    let mut slot = 0;
    let mut copies = 0;
    for (index, item) in items.iter().enumerate() {
        if item.kind() == selected_kind {
            copies += 1;
            if rng.below(copies) == 0 {
                slot = index;
            }
        }
    }
    let original = &items[slot];
    let footprint = shape_mask(original.shape());

    let rotations: &[Rotation] = if config.allow_rotation {
        &Rotation::ALL
    } else {
        &[Rotation::Deg0]
    };
    let mut selected = None;
    let mut candidate_count = 0;
    for kind in ItemKind::ALL {
        if kind == original.kind() || items.iter().filter(|item| item.kind() == kind).count() >= 2 {
            continue;
        }
        for rotation in rotations {
            let candidate = Item::with_rotation(kind, original.position(), *rotation);
            if shape_mask(candidate.shape()) == footprint {
                candidate_count += 1;
                if rng.below(candidate_count) == 0 {
                    selected = Some(candidate);
                }
                break;
            }
        }
    }
    let Some(candidate) = selected else {
        return;
    };

    record(&mut stats[original.kind() as usize], original_tally);
    let kind = candidate.kind();
    let mut swapped = items.to_vec();
    swapped[slot] = candidate;
    let bag = Bag::new(swapped).expect("same-footprint swap keeps the bag valid");
    let left = Hero::new("left", config.hero_health, bag);
    let right = Hero::new("right", config.hero_health, right_bag.clone());
    let tally = match simulate(left, right, battle_config).outcome {
        Outcome::LeftWins => Tally::Win,
        Outcome::RightWins => Tally::Loss,
        Outcome::Draw => Tally::Draw,
    };
    record(&mut stats[kind as usize], tally);
}

fn shape_mask(shape: &[Offset]) -> u32 {
    shape.iter().fold(0, |mask, offset| {
        mask | 1 << (u32::from(offset.y) * 4 + u32::from(offset.x))
    })
}

fn record(stat: &mut ItemStat, tally: Tally) {
    match tally {
        Tally::Win => stat.swap_wins += 1,
        Tally::Loss => stat.swap_losses += 1,
        Tally::Draw => stat.swap_draws += 1,
    }
}
