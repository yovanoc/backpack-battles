# Battle Engine Architecture

## Adding an item

1. Add its `ItemKind` variant, archetype arm, `definition()` entry, and `ALL` array slot in `src/catalog/mod.rs`. Tune its balance values with the `Definition` builder from `src/catalog/definition.rs`; add a shape to `src/item_shapes.rs` only if none fits.
2. Add its runtime state variant in `src/item/state.rs` (the `ItemState` enum plus the `new`, `kind`, and `can_activate_again` arms).
3. Return typed effects from the matching phase: `battle_start` or `tick` in `src/behavior/mod.rs`, or the activation phase in `src/behavior/activate.rs`.
4. Reuse existing effects. Add an `EffectKind` and resolver branch only for a genuinely new game primitive.
5. Add its catalog text in `src/item_description.rs`.
6. Test observable events through `Battle::advance`.

An item never mutates another item or Hero directly. It produces effects; the resolver applies them in order.

## Tick order

Each 100ms tick resolves as two 50ms ranks: the left Hero (attacker) on the even rank, then the right Hero (defender) on the odd rank, so opposing Items never share an instant. Within each rank:

1. Battle-start behavior on tick 1 (left rank, then right rank).
2. That side's natural fall attempts.
3. That side's per-tick conditions and accumulation.
4. Advance that side's charges, then snapshot its ready Item instances.
5. Resolve that side's activations by anchored Bag cell, stopping the moment either Hero dies.
6. Finish on zero health or the configured tick limit.

The order of phases, Item IDs, effects, and random draws is part of deterministic replay behavior.

## Modules

- `catalog/mod.rs`: item kinds, archetypes, definitions, and the full roster (`ALL`).
- `catalog/definition.rs`: the immutable `Definition` builder and its balance-value fields.
- `item/mod.rs`: stable Item identity, charge, speed, uses, and shape.
- `item/state.rs`: the per-kind `ItemState` enum and its kind mapping.
- `bag.rs`: shape validation, adjacency, targeting queries, and derived passive stats.
- `behavior/mod.rs`: battle-start and per-tick Item behavior.
- `behavior/activate.rs`: exhaustive activation behavior by Item kind.
- `effect.rs`: closed vocabulary of requested state changes.
- `resolver.rs`: damage, block, healing, modification, targeting, and fall pipelines.
- `combat.rs`: mutable Heroes, item lookup, phase execution, and seeded randomness.
- `battle.rs`: the public deterministic tick loop.

## Invariants

- An `ItemRef` is stable for the Battle because it uses side plus anchored Bag cell.
- Public events are resolved facts; they never trigger behavior.
- Effects from a removed source are discarded.
- Derived stats are scanned from active Items, so falling Items stop contributing immediately.
- Item targeting uses stable Item ID order.
- No global randomness, wall clock, ECS, scripting, or dynamic behavior registry.
