use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Tsify, Serialize)]
#[tsify(into_wasm_abi, large_number_types_as_bigints)]
pub struct Replay {
    pub bag_width: u8,
    pub bag_height: u8,
    pub seed: u64,
    pub base_health: u16,
    pub tick_limit: u16,
    pub left: HeroView,
    pub right: HeroView,
    pub ticks: Vec<TickView>,
    pub outcome: OutcomeView,
    pub result_ticks: u16,
    pub left_final_health: u16,
    pub right_final_health: u16,
}

#[derive(Tsify, Serialize)]
pub struct HeroView {
    pub name: String,
    pub max_health: u16,
    pub items: Vec<ItemView>,
}

#[derive(Tsify, Serialize)]
pub struct ItemView {
    pub id: [u8; 2],
    pub kind: String,
    pub archetype: ArchetypeView,
    pub effect: String,
    pub cells: Vec<[u8; 2]>,
    pub stats: StatsView,
}

#[derive(Tsify, Serialize)]
pub struct TickView {
    pub tick: u16,
    pub left_health: u16,
    pub right_health: u16,
    pub left_block: u16,
    pub right_block: u16,
    pub left_poison: u16,
    pub right_poison: u16,
    pub left_items: Vec<ItemRuntimeView>,
    pub right_items: Vec<ItemRuntimeView>,
    pub events: Vec<EventView>,
}

#[derive(Tsify, Serialize)]
pub struct ItemRuntimeView {
    pub id: [u8; 2],
    pub charge_progress: Option<f64>,
    pub speed_basis_points: u16,
}

#[derive(Tsify, Serialize, Default)]
pub struct EventView {
    pub kind: EventKind,
    pub side: Option<SideView>,
    pub item: Option<[u8; 2]>,
    pub by: Option<[u8; 2]>,
    pub item_kind: Option<String>,
    pub amount: Option<u16>,
    pub mode: Option<DamageModeView>,
    pub cause: Option<FallCauseView>,
}

#[derive(Tsify, Serialize)]
#[tsify(large_number_types_as_bigints)]
pub struct StatsView {
    pub weapon: bool,
    pub weight: u16,
    pub can_fall: bool,
    pub armor: u16,
    pub max_health: u16,
    pub adjacent_damage: u16,
    pub retaliation: u16,
    pub vengeful: bool,
    pub first_activation: Option<u16>,
    pub cadence: Option<u16>,
    pub natural_fall_every: Option<u16>,
    pub natural_fall_one_in: Option<u64>,
}

#[derive(Tsify, Deserialize)]
pub struct Placement {
    pub kind: u16,
    pub x: u8,
    pub y: u8,
    #[serde(default)]
    pub rotation: u8,
}

#[derive(Tsify, Deserialize)]
#[serde(transparent)]
#[tsify(from_wasm_abi)]
pub struct Placements(pub Vec<Placement>);

#[derive(Tsify, Serialize)]
pub struct ItemInfo {
    pub index: u16,
    pub kind: String,
    pub archetype: ArchetypeView,
    pub effect: String,
    pub shape: Vec<[u8; 2]>,
    pub stats: StatsView,
}

#[derive(Tsify, Serialize)]
#[serde(transparent)]
#[tsify(into_wasm_abi, large_number_types_as_bigints)]
pub struct ItemCatalog(pub Vec<ItemInfo>);

#[derive(Tsify, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchetypeView {
    Aggression,
    Defense,
    Scaling,
    Control,
    Support,
}

#[derive(Tsify, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeView {
    Left,
    Right,
    Draw,
}

#[derive(Tsify, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SideView {
    Left,
    Right,
}

#[derive(Tsify, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    #[default]
    Activated,
    Damage,
    HealthLost,
    Healed,
    Block,
    Speed,
    Fell,
    FallPrevented,
    Consumed,
    Poisoned,
    PoisonDamage,
    PoisonCleansed,
}

#[derive(Tsify, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageModeView {
    Normal,
    Piercing,
    Retaliation,
}

#[derive(Tsify, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FallCauseView {
    Natural,
    Forced,
}
