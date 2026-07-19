use crate::{Archetype, Bag};

pub(super) fn dominant_archetype(bag: &Bag) -> Option<Archetype> {
    let mut cells = [0; Archetype::COUNT];
    for item in bag.items() {
        cells[item.kind().archetype() as usize] += item.shape().len();
    }
    let mut dominant = None;
    let mut maximum = 0;
    let mut tied = false;
    for archetype in Archetype::ALL {
        let score = cells[archetype as usize];
        if score > maximum {
            dominant = Some(archetype);
            maximum = score;
            tied = false;
        } else if score == maximum && score > 0 {
            tied = true;
        }
    }
    if tied { None } else { dominant }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cell, Item, ItemKind};

    #[test]
    fn tied_cell_investment_has_no_dominant_archetype() {
        let bag = Bag::new(vec![
            Item::new(ItemKind::Dagger, Cell::new(0, 0)),
            Item::new(ItemKind::PoisonVial, Cell::new(0, 1)),
        ])
        .expect("valid bag");

        assert_eq!(dominant_archetype(&bag), None);
    }

    #[test]
    fn greatest_cell_investment_is_dominant() {
        let bag = Bag::new(vec![
            Item::new(ItemKind::WoodenSword, Cell::new(0, 0)),
            Item::new(ItemKind::Dagger, Cell::new(0, 1)),
            Item::new(ItemKind::PoisonVial, Cell::new(0, 2)),
        ])
        .expect("valid bag");

        assert_eq!(dominant_archetype(&bag), Some(Archetype::Aggression));
    }
}
