# Item Catalog

All 34 items with their exact battle data, grouped by archetype. Source of truth:
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

## Defense

| Item | Shape | Wt | Act | Armor | HP | Retal | Fall | Effect |
|---|---|---|---|---|---|---|---|---|
| Loose Buckler | 2×2 | 1 | — | 2 | 30 | — | 1-in-8 every 20 | +30 health, +2 armor |
| Armor | 2×2 | 10 | — | 2 | 40 | — | fixed | +40 health, +2 armor; cannot fall |
| Shield | 1×2 | 1 | 25/25 | — | — | — | — | +8 block, maximum 24 |
| Potted Cactus | 1×1 | 1 | — | — | — | 2 | — | retaliate for 2 damage when hit |
| Healing Potion | 1×1 | 1 | — | — | — | — | — | heal 25 below half health, then consume |
| Thornmail | L(3) | 1 | — | 1 | — | 3 | — | +1 armor; retaliate for 3 damage |
| Spiked Targe | 1×1 | 1 | 20/20 | — | — | 1 | — | +3 block, maximum 12; retaliate for 1 damage |
| Pavise | 3×1 | 8 | — | 1 | 24 | — | fixed | +24 health, +1 armor; cannot fall |

## Scaling

| Item | Shape | Wt | Act | Fall | Effect |
|---|---|---|---|---|---|
| Hourglass | 1×1 | 1 | 20/20 | — | adjacent items +400 speed bps |
| Leech | 1×1 | 1 | 10/10 | — | drain 1 health and heal 1 |
| Grimoire | 2×2 | 10 | 30/43 | fixed | 10 normal damage, +5 each activation; cannot fall |
| Poison Vial | 1×1 | 1 | 15/15 | — | enemy loses 2 health |
| Doom Candle | 1×1 | 1 | 45/20 | — | enemy loses 3 health |
| Blood Chalice | 1×2 | 1 | 35/25 | — | enemy loses 3 health and heal 1 |

## Control

| Item | Shape | Wt | Act | Effect |
|---|---|---|---|---|
| Grappling Hook | 1×2 | 1 | 60/60 | force lightest edge item to fall; 2 uses |
| Weighted Net | 1×1 | 1 | — | force lightest edge weapon to fall at battle start |
| Bomb | 2×1 | 1 | 30/30 | 10 piercing damage, then 1/2 drop lightest edge weapon; consumed |
| Caltrops | 3×1 | 1 | 30/30 | drop lightest edge item |
| Tripwire | 1×1 | 1 | — | 1/2 drop lightest edge item at battle start; consumed |
| Shrapnel Mine | 2×1 | 1 | 20/20 | 10 piercing damage, then 1/3 drop lightest edge weapon; consumed |
| Bear Trap | L(3) | 1 | 24/35 | 12 piercing damage, then drop lightest edge item; consumed |

## Support

| Item | Shape | Wt | Act | Adj | HP | Effect |
|---|---|---|---|---|---|---|
| Whetstone | 1×1 | 1 | — | 2 | — | adjacent weapons +2 damage |
| Strap | 1×1 | 1 | — | — | — | protect adjacent items from falling |
| War Banner | 3×1 | 1 | — | 7 | 10 | +10 health; adjacent weapons +7 damage |
| Signal Drum | 1×2 | 1 | 15/30 | — | — | adjacent items +800 speed bps |
| Field Kit | 1×1 | 1 | — | — | — | heal 15 below half health, then consume |
| Barricade Kit | 2×1 | 1 | — | — | — | +10 block, maximum 18 at battle start; consumed |
