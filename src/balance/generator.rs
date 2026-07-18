use super::CampaignMode;
use crate::{
    Archetype, Bag, Cell, Item, ItemKind, Offset, Rotation,
    geometry::{BAG_CELLS, BAG_HEIGHT, BAG_WIDTH},
    rng::Rng,
};

const PLACEMENT_ATTEMPTS: usize = 40;

pub(crate) fn random_bag(rng: &mut Rng, allow_rotation: bool) -> Bag {
    generated_bag(rng, allow_rotation, CampaignMode::Random, 0)
}

pub(super) fn generated_bag(
    rng: &mut Rng,
    allow_rotation: bool,
    mode: CampaignMode,
    bag_index: u64,
) -> Bag {
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

fn allowed(archetype: Archetype, mode: CampaignMode, bag_index: u64) -> bool {
    let primary = usize::try_from(bag_index % Archetype::COUNT as u64).unwrap_or(0);
    match mode {
        CampaignMode::Random => true,
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
}
