use std::{error::Error, fmt};

use crate::{BAG_HEIGHT, BAG_WIDTH, Cell, Item, ItemId, ItemKind, geometry::BAG_CELLS};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bag {
    items: Vec<Item>,
}

impl Bag {
    pub fn new(mut items: Vec<Item>) -> Result<Self, BagError> {
        let mut occupied = [false; BAG_CELLS];
        let mut copies = [0_u8; ItemKind::COUNT];
        for item in &items {
            copies[item.kind() as usize] += 1;
            if copies[item.kind() as usize] > 2 {
                return Err(BagError::TooManyCopies { item: item.kind() });
            }
            for offset in item.shape() {
                let position = item.position();
                let Some(x) = position.x.checked_add(offset.x) else {
                    return Err(BagError::OutOfBounds { item: item.kind() });
                };
                let Some(y) = position.y.checked_add(offset.y) else {
                    return Err(BagError::OutOfBounds { item: item.kind() });
                };
                if x >= BAG_WIDTH || y >= BAG_HEIGHT {
                    return Err(BagError::OutOfBounds { item: item.kind() });
                }
                let at = Cell::new(x, y);
                let index = usize::from(y) * usize::from(BAG_WIDTH) + usize::from(x);
                if occupied[index] {
                    return Err(BagError::Overlap {
                        item: item.kind(),
                        at,
                    });
                }
                occupied[index] = true;
            }
        }
        items.sort_by_key(|item| item.id());
        Ok(Self { items })
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub(crate) fn items_mut(&mut self) -> &mut [Item] {
        &mut self.items
    }

    pub(crate) fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.iter().find(|item| item.id() == id)
    }

    pub(crate) fn item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        self.items.iter_mut().find(|item| item.id() == id)
    }

    pub(crate) fn remove(&mut self, id: ItemId) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id() == id)?;
        Some(self.items.remove(index))
    }

    pub(crate) fn adjacent_ids(&self, source: ItemId) -> impl Iterator<Item = ItemId> + '_ {
        self.items
            .iter()
            .filter(move |item| item.id() != source && self.items_touch(source, item.id()))
            .map(Item::id)
    }

    pub(crate) fn lightest_edge(&self, weapons_only: bool) -> Option<ItemId> {
        self.items
            .iter()
            .filter(|item| item.kind().can_fall())
            .filter(|item| !weapons_only || item.kind().is_weapon())
            .filter(|item| self.is_on_edge(item.id()))
            .min_by_key(|item| (item.kind().weight(), item.id()))
            .map(Item::id)
    }

    pub(crate) fn armor(&self) -> u16 {
        // ponytail: saturating fold over ≤12 small-value cells; overflow is
        // theoretical, the cap is a harmless ceiling not a real bound.
        self.items
            .iter()
            .fold(0, |total, item| total.saturating_add(item.kind().armor()))
    }

    pub(crate) fn max_health_bonus(&self) -> u16 {
        self.items.iter().fold(0, |total, item| {
            total.saturating_add(item.kind().max_health())
        })
    }

    pub(crate) fn adjacent_damage_bonus(&self, id: ItemId) -> u16 {
        self.adjacent_ids(id)
            .filter_map(|adjacent_id| self.item(adjacent_id))
            .fold(0, |total, item| {
                total.saturating_add(item.kind().adjacent_damage())
            })
    }

    pub(crate) fn retaliators(&self) -> impl Iterator<Item = (ItemId, u16)> + '_ {
        self.items.iter().filter_map(|item| {
            let damage = item.kind().retaliation();
            (damage > 0).then_some((item.id(), damage))
        })
    }

    pub(crate) fn protector(&self, target: ItemId) -> Option<ItemId> {
        self.items
            .iter()
            .filter(|item| item.kind() == ItemKind::Strap)
            .filter(|item| self.items_touch(target, item.id()))
            .map(Item::id)
            .next()
    }

    fn is_on_edge(&self, id: ItemId) -> bool {
        let Some(item) = self.item(id) else {
            return false;
        };
        item.shape().iter().any(|offset| {
            let x = item.position().x + offset.x;
            let y = item.position().y + offset.y;
            x == 0 || y == 0 || x + 1 == BAG_WIDTH || y + 1 == BAG_HEIGHT
        })
    }

    fn items_touch(&self, first_id: ItemId, second_id: ItemId) -> bool {
        let (Some(first), Some(second)) = (self.item(first_id), self.item(second_id)) else {
            return false;
        };
        first.shape().iter().any(|first_cell| {
            second.shape().iter().any(|second_cell| {
                let first_x = first.position().x + first_cell.x;
                let first_y = first.position().y + first_cell.y;
                let second_x = second.position().x + second_cell.x;
                let second_y = second.position().y + second_cell.y;
                first_x.abs_diff(second_x) + first_y.abs_diff(second_y) == 1
            })
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BagError {
    OutOfBounds { item: ItemKind },
    Overlap { item: ItemKind, at: Cell },
    TooManyCopies { item: ItemKind },
}

impl fmt::Display for BagError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { item } => {
                write!(formatter, "{} does not fit in the bag", item.name())
            }
            Self::Overlap { item, at } => write!(
                formatter,
                "{} overlaps another item at ({}, {})",
                item.name(),
                at.x,
                at.y
            ),
            Self::TooManyCopies { item } => {
                write!(
                    formatter,
                    "a bag can hold at most two {} items",
                    item.name()
                )
            }
        }
    }
}

impl Error for BagError {}

impl fmt::Display for Bag {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut grid = ['·'; BAG_CELLS];
        for (index, item) in self.items.iter().enumerate() {
            let symbol = item_symbol(index);
            for offset in item.shape() {
                let x = item.position().x + offset.x;
                let y = item.position().y + offset.y;
                let cell = usize::from(y) * usize::from(BAG_WIDTH) + usize::from(x);
                if let Some(slot) = grid.get_mut(cell) {
                    *slot = symbol;
                }
            }
        }
        for y in 0..BAG_HEIGHT {
            for x in 0..BAG_WIDTH {
                let cell = usize::from(y) * usize::from(BAG_WIDTH) + usize::from(x);
                write!(formatter, "{} ", grid[cell])?;
            }
            writeln!(formatter)?;
        }
        for (index, item) in self.items.iter().enumerate() {
            writeln!(formatter, "  {} {}", item_symbol(index), item.kind().name())?;
        }
        Ok(())
    }
}

fn item_symbol(index: usize) -> char {
    char::from_digit(u32::try_from(index).unwrap_or(35) + 1, 36).unwrap_or('?')
}
