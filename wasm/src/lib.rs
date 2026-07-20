//! WebAssembly bridge for the deterministic battle engine.
//!
//! The engine types are not `serde`-aware (kept dependency-free), so this crate
//! defines small serialisable DTOs and maps the engine's public API onto them.
//! `run_battle` drives one full deterministic battle and returns the initial
//! bags plus every tick's health/block/events, which the viewer plays back.

use backpack_battles::{
    Archetype, BAG_HEIGHT, BAG_WIDTH, BASE_HEALTH, Bag, Battle, BattleConfig, BattleUpdate, Cell,
    Hero, Item, ItemKind, ItemStats, MAX_TICKS, Outcome, Rotation, TickReport, demo_heroes,
};
use wasm_bindgen::prelude::*;

mod dto;
mod events;
use dto::*;
use events::event_view;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Run one deterministic battle between two seed-generated demo bags and return
/// the full replay (initial bags + per-tick state and events).
#[wasm_bindgen]
pub fn run_battle(seed: u64, hero_health: u16, tick_limit: u16) -> Result<Replay, JsValue> {
    let health = if hero_health == 0 {
        BASE_HEALTH
    } else {
        hero_health
    };
    // demo_heroes fixes both bags from the seed; rebuild the heroes so the caller
    // can override base health while keeping the same generated bags.
    let (base_left, base_right) = demo_heroes(seed);
    let left = Hero::new(base_left.name(), health, base_left.bag().clone());
    let right = Hero::new(base_right.name(), health, base_right.bag().clone());
    play_battle(left, right, seed, health, tick_limit)
}

/// Run a battle between two caller-built bags (the bag-editor path). Each side
/// is an array of `Placement` objects; `Bag::new` validates the layout.
#[wasm_bindgen]
pub fn run_battle_with_bags(
    left: Placements,
    right: Placements,
    seed: u64,
    hero_health: u16,
    tick_limit: u16,
) -> Result<Replay, JsValue> {
    let health = if hero_health == 0 {
        BASE_HEALTH
    } else {
        hero_health
    };
    let left = build_hero("Left", &left.0, health)?;
    let right = build_hero("Right", &right.0, health)?;
    play_battle(left, right, seed, health, tick_limit)
}

fn play_battle(
    left: Hero,
    right: Hero,
    seed: u64,
    health: u16,
    tick_limit: u16,
) -> Result<Replay, JsValue> {
    let ticks_limit = tick_limit.clamp(1, MAX_TICKS);
    let left_view = hero_view(&left);
    let right_view = hero_view(&right);
    let config = BattleConfig::new(ticks_limit, seed)
        .map_err(|error| JsValue::from_str(&format!("invalid config: {error}")))?;
    let mut battle = Battle::new(left, right, config);

    let mut ticks: Vec<TickView> = Vec::new();
    let result = loop {
        match battle.advance() {
            BattleUpdate::Tick(report) => ticks.push(tick_view(&report, &battle)),
            BattleUpdate::Finished(result) => break result,
        }
    };
    let outcome = match result.outcome {
        Outcome::LeftWins => OutcomeView::Left,
        Outcome::RightWins => OutcomeView::Right,
        Outcome::Draw => OutcomeView::Draw,
    };

    let replay = Replay {
        bag_width: BAG_WIDTH,
        bag_height: BAG_HEIGHT,
        seed,
        base_health: health,
        tick_limit: ticks_limit,
        left: left_view,
        right: right_view,
        ticks,
        outcome,
        result_ticks: result.ticks,
        left_final_health: result.left_health,
        right_final_health: result.right_health,
    };
    Ok(replay)
}

/// Every item kind with its archetype, effect text and footprint - for a legend.
#[wasm_bindgen]
pub fn list_items() -> ItemCatalog {
    let items: Vec<ItemInfo> = ItemKind::ALL
        .iter()
        .zip(0_u16..)
        .map(|(kind, index)| ItemInfo {
            index,
            kind: kind.name().to_string(),
            archetype: archetype_view(kind.archetype()),
            effect: kind.effect_description().to_string(),
            shape: kind
                .shape()
                .iter()
                .map(|offset| [offset.x, offset.y])
                .collect(),
            stats: stats_view(kind.stats()),
        })
        .collect();
    ItemCatalog(items)
}

/// The base health used when the caller passes 0.
#[wasm_bindgen]
pub fn default_health() -> u16 {
    BASE_HEALTH
}

// ---------------------------------------------------------------------------
// Mapping helpers
// ---------------------------------------------------------------------------

fn hero_view(hero: &Hero) -> HeroView {
    HeroView {
        name: hero.name().to_string(),
        max_health: hero.max_health(),
        items: hero.bag().items().iter().map(item_view).collect(),
    }
}

fn item_view(item: &Item) -> ItemView {
    let kind = item.kind();
    let anchor = item.id().cell();
    let position = item.position();
    let cells = item
        .shape()
        .iter()
        .map(|offset| [position.x + offset.x, position.y + offset.y])
        .collect();
    ItemView {
        id: [anchor.x, anchor.y],
        kind: kind.name().to_string(),
        archetype: archetype_view(kind.archetype()),
        effect: kind.effect_description().to_string(),
        cells,
        stats: stats_view(kind.stats()),
    }
}

fn tick_view(report: &TickReport, battle: &Battle) -> TickView {
    TickView {
        tick: report.tick,
        left_health: report.left_health,
        right_health: report.right_health,
        left_block: report.left_block,
        right_block: report.right_block,
        left_poison: battle.left_hero().poison(),
        right_poison: battle.right_hero().poison(),
        left_items: runtime_items(battle.left_hero()),
        right_items: runtime_items(battle.right_hero()),
        events: report.events.iter().map(event_view).collect(),
    }
}

fn runtime_items(hero: &Hero) -> Vec<ItemRuntimeView> {
    hero.bag()
        .items()
        .iter()
        .map(|item| {
            let cell = item.id().cell();
            ItemRuntimeView {
                id: [cell.x, cell.y],
                charge_progress: item.charge_progress(),
                speed_basis_points: item.speed_basis_points(),
            }
        })
        .collect()
}

fn archetype_view(archetype: Archetype) -> ArchetypeView {
    match archetype {
        Archetype::Aggression => ArchetypeView::Aggression,
        Archetype::Defense => ArchetypeView::Defense,
        Archetype::Scaling => ArchetypeView::Scaling,
        Archetype::Control => ArchetypeView::Control,
        Archetype::Support => ArchetypeView::Support,
    }
}

fn stats_view(stats: ItemStats) -> StatsView {
    StatsView {
        weapon: stats.weapon,
        weight: stats.weight,
        can_fall: stats.can_fall,
        armor: stats.armor,
        max_health: stats.max_health,
        adjacent_damage: stats.adjacent_damage,
        retaliation: stats.retaliation,
        vengeful: stats.vengeful,
        first_activation: stats.first_activation,
        cadence: stats.cadence,
        natural_fall_every: stats.natural_fall_every,
        natural_fall_one_in: stats.natural_fall_one_in,
    }
}

fn build_hero(name: &str, placements: &[Placement], health: u16) -> Result<Hero, JsValue> {
    let mut items = Vec::with_capacity(placements.len());
    for placement in placements {
        let kind = *ItemKind::ALL
            .get(usize::from(placement.kind))
            .ok_or_else(|| {
                JsValue::from_str(&format!("item index {} out of range", placement.kind))
            })?;
        let rotation = Rotation::ALL[usize::from(placement.rotation % 4)];
        items.push(Item::with_rotation(
            kind,
            Cell::new(placement.x, placement.y),
            rotation,
        ));
    }
    let bag =
        Bag::new(items).map_err(|error| JsValue::from_str(&format!("invalid bag: {error}")))?;
    Ok(Hero::new(name, health, bag))
}
