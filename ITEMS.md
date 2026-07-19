# Item Catalog

All 50 items with their exact battle data, grouped by archetype. Source of truth:
[`src/catalog/mod.rs`](src/catalog/mod.rs), [`src/item_description.rs`](src/item_description.rs).

## Column meaning

- **Shape** — cells occupied: `1×1`, `1×2` (vertical), `2×1` (horizontal), `2×2`, `3×1` (line), `L(3)`/`L(4)` (L-tromino/tetromino).
- **Wt** — weight (drop-priority tiebreak; lighter falls first). Default `1`.
- **Act** — activation timing as `initial/recurring` in ticks (1 tick = 100 ms). `—` = passive, never self-activates.
- **Wpn** — weapon flag (gets weapon damage buffs from Whetstone / War Banner).
- **Armor** — flat damage reduction added to the hero.
- **HP** — max-health bonus added to the hero.
- **Adj** — adjacent-weapon damage bonus granted to neighbors.
- **Retal** — retaliation damage dealt back when the hero is hit.
- **Fall** — natural fall chance as `1-in-N every M ticks`; `fixed` = cannot fall; `—` = can fall but no natural fall.
- **Poison** — a stacking damage-over-time reserve on the hero. At the start of each of that hero's ranks it takes raw damage equal to its stack count (ignores armor and block), then one stack decays. Reapply faster than it decays to ramp it.
- **Cleanse** — removes stacks from the hero's own poison reserve (Defense's answer to Scaling). Does not affect raw "loses N health" hits.

## Aggression

| Item | Shape | Wt | Act | Wpn | Effect |
|---|---|---|---|---|---|
| Wooden Sword | 1×2 | 3 | 20/20 | ✔ | 8 normal damage |
| Crossbow | 2×1 | 4 | 15/30 | ✔ | 10 normal damage |
| Windbreaker | 1×1 | 2 | 25/25 | ✔ | 6 piercing damage |
| Spear | L(4) | 5 | 30/30 | ✔ | 16 normal damage |
| Dagger | 1×1 | 1 | 5/13 | ✔ | 4 normal damage |
| Rapier | 1×2 | 3 | 26/26 | ✔ | 6 piercing damage |
| Warhammer | 2×2 | 10 | 37/37 | ✔ | 18 normal damage; cannot fall (`fixed`) |
| Katana | 1×2 | 4 | 22/22 | ✔ | 9 normal damage |
| Throwing Axe | 2×1 | 3 | 18/36 | ✔ | 11 piercing damage |
| Morning Star | L(3) | 6 | 34/34 | ✔ | 14 normal damage |

## Defense

| Item | Shape | Wt | Act | Armor | HP | Retal | Fall | Effect |
|---|---|---|---|---|---|---|---|---|
| Loose Buckler | 2×2 | 1 | — | 1 | 20 | — | 1-in-8 every 20 | +20 health, +1 armor |
| Armor | 2×2 | 10 | — | 1 | 28 | — | fixed | +28 health, +1 armor; cannot fall |
| Shield | 1×2 | 1 | 25/25 | — | — | — | — | +6 block, maximum 20; cleanse 3 poison |
| Potted Cactus | 1×1 | 1 | — | — | — | 2 | — | retaliate for 2 damage when hit |
| Healing Potion | 1×1 | 1 | — | — | — | — | — | heal 16 below half health, then consume |
| Thornmail | L(3) | 1 | — | 1 | — | 3 | — | +1 armor; retaliate for 3 damage |
| Spiked Targe | 1×1 | 1 | 20/20 | — | — | 1 | — | +3 block, maximum 12; retaliate for 1 damage; cleanse 2 poison |
| Pavise | 3×1 | 8 | — | 1 | 16 | — | fixed | +16 health, +1 armor; cannot fall |
| Chainmail | 2×2 | 9 | — | 3 | 14 | — | fixed | +14 health, +3 armor; cannot fall |
| Bulwark | 1×2 | 1 | 30/30 | — | — | — | — | +10 block, maximum 30; cleanse 4 poison |

## Scaling

| Item | Shape | Wt | Act | Fall | Effect |
|---|---|---|---|---|---|
| Hourglass | 1×1 | 1 | 20/20 | — | adjacent items +400 speed bps |
| Leech | 1×1 | 1 | 10/10 | — | drain 1 health and heal 1 |
| Grimoire | 2×2 | 10 | 30/43 | fixed | 10 normal damage, +5 each activation; cannot fall |
| Poison Vial | 1×1 | 1 | 15/15 | — | enemy loses 2 health |
| Doom Candle | 1×1 | 1 | 45/20 | — | enemy loses 3 health |
| Blood Chalice | 1×2 | 1 | 35/25 | — | enemy loses 3 health and heal 1 |
| Venom Fang | 1×1 | 1 | 12/12 | — | apply 2 poison |
| Spellbook | 1×2 | 1 | 28/40 | fixed | 6 normal damage, +4 each activation; cannot fall |
| Vampiric Blade | 1×2 | 1 | 22/22 | — | enemy loses 4 health and heal 2 |
| Plague Censer | 2×1 | 1 | 24/24 | — | apply 4 poison |

## Control

| Item | Shape | Wt | Act | Effect |
|---|---|---|---|---|
| Grappling Hook | 1×2 | 1 | 60/60 | force lightest edge item to fall; 2 uses |
| Weighted Net | 1×1 | 1 | — | force lightest edge weapon to fall at battle start |
| Bomb | 2×1 | 1 | 30/30 | 10 piercing damage, then 1/2 drop lightest edge weapon; consumed |
| Caltrops | 3×1 | 1 | 30/30 | 2 normal damage; delay enemy's next activation by 6 ticks |
| Tripwire | 1×1 | 1 | — | 1/2 drop lightest edge item at battle start; consumed |
| Shrapnel Mine | 2×1 | 1 | 20/20 | 10 piercing damage, then 1/3 drop lightest edge weapon; consumed |
| Bear Trap | L(3) | 1 | 24/35 | 12 piercing damage, then drop lightest edge item; consumed |
| Bola | 1×1 | 1 | — | force lightest edge item to fall at battle start; consumed |
| Grenade | 2×1 | 1 | 25/25 | 8 piercing damage, then 1/3 drop lightest edge item; consumed |
| Harpoon Gun | 1×2 | 1 | 45/45 | force lightest edge item to fall; 3 uses |

## Support

| Item | Shape | Wt | Act | Adj | HP | Effect |
|---|---|---|---|---|---|---|
| Whetstone | 1×1 | 1 | — | 2 | — | adjacent weapons +2 damage |
| Strap | 1×1 | 1 | — | — | — | protect adjacent items from falling |
| War Banner | 3×1 | 1 | — | 7 | 10 | +10 health; adjacent weapons +7 damage |
| Signal Drum | 1×2 | 1 | 15/30 | — | — | adjacent items +800 speed bps |
| Field Kit | 1×1 | 1 | — | — | — | heal 15 below half health, then consume |
| Barricade Kit | 2×1 | 1 | — | — | — | +10 block, maximum 18 at battle start; consumed |
| Grindstone | 1×1 | 1 | — | 4 | — | adjacent weapons +4 damage |
| Metronome | 1×1 | 1 | 20/20 | — | — | adjacent items +600 speed bps |
| Medic Bag | 2×1 | 1 | — | — | — | heal 30 below half health, then consume |
| Rallying Horn | 1×2 | 1 | — | 4 | 8 | +8 health; adjacent weapons +4 damage |
