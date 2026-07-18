# Backpack Battles

A deterministic backpack autobattler engine and balance workbench, written in Rust.

Two heroes each own a grid **Bag** of non-overlapping **Items**. A battle is a pure
function of `(left bag, right bag, seed)`: the same inputs always produce the same
fight, so replays are exact and balance can be measured by mass simulation.

![Watching a deterministic battle in the terminal UI](docs/watch.gif)

*The `watch` TUI: HP gauges, archetype-colored bags, per-item charge bars, and a live fight journal.*

## Origins

This project is an implementation of a game design imagined by **[s17n](https://www.twitch.tv/s17n)** on stream.

The founding design document — *« L'Arène » (titre provisoire) — v2* — lives here:
<https://docs.blackprism.org/s/18b4c51c-ce22-4963-a9ad-505e6fa865a9#h-document-fondateur-%E2%80%94-%C2%AB-larene-%C2%BB-titre-provisoire-%E2%80%94-v2>

## Core ideas

- **Deterministic.** No wall clock, no global randomness. Every chance-based outcome is fixed by the battle seed.
- **Interleaved ranks.** Each 100 ms tick resolves as two 50 ms ranks: the left hero (attacker) on the even rank, the right hero (defender) on the odd rank. Opposing items never resolve in the same instant, so there is no "who goes first" tiebreak bias. A calibrated **defender guard** offsets the attacker's opening-timing edge.
- **Effects, not mutations.** An item never touches another item or hero directly. It emits typed effects; the resolver applies them in a fixed order through the damage pipeline (armor → block → health, with piercing and raw retaliation as named exceptions).
- **Archetypes.** Aggression, Defense, Scaling, Control, Support — a triangle where each branch beats its prey, plus Control as a cross-cutting disruption axis.

## Usage

```sh
cargo build --release

# Show every item's exact rules and the opening activation timeline.
./target/release/backpack-battles catalog

# Run one deterministic demo battle.
./target/release/backpack-battles battle

# Watch a battle in a full-screen terminal UI.
./target/release/backpack-battles watch

# Simulate many battles to surface over/underpowered items and side bias.
./target/release/backpack-battles balance --battles 100000 --mirror --campaign hybrid
```

### Balance campaigns

`balance` generates random bags and tallies, per item, how often the bag holding it
won — controlling for size via matched same-footprint swaps and ranking by a
Hoeffding 95% lower bound.

| Flag | Meaning |
|---|---|
| `--battles <N>` | Number of battles to simulate (default 100000). |
| `--seed <N>` | Fixes the whole campaign (default 42). |
| `--mirror` | Replays every matchup with bags swapped to isolate engine side bias. |
| `--campaign <random\|pure\|hybrid>` | Bag generation: random mixes, pure single-archetype, hybrid archetype pairs. |
| `--health <N>` | Base hero health (default 100). |
| `--ticks <N>` | Tick limit (default 600). |
| `--no-rotate` | Place every item upright instead of using random rotations. |

The report also prints side-bias and shared-rank telemetry (`shared_activation` /
`shared_lethal`), which stay at zero by construction and act as regression tripwires
for the interleaving invariant.

## Documentation

- [`CONTEXT.md`](CONTEXT.md) — glossary / ubiquitous language for the battle domain.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) — engine architecture, tick order, and how to add an item.
- [`DESIGN.md`](DESIGN.md) — the terminal UI design system.

## Development

```sh
cargo test                                              # unit + integration tests
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```
