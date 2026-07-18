# Battle Context

## Glossary

- **Battle**: One deterministic fight between a left Hero and a right Hero.
- **Battle Seed**: The value that fixes every chance-based outcome in a Battle. The same seed, Heroes, and rules produce the same Battle.
- **Hero**: A combatant with health and exactly one Bag.
- **Bag**: The Hero-owned grid of non-overlapping Items whose effects are currently active.
- **Item**: A placed thing with a Shape and its own battle data. It may affect its Hero, the enemy Hero, or nearby Items.
- **Item Instance**: One Item in one Battle, identified by its anchored Bag cell and carrying Battle-only state such as charge, stacks, or uses.
- **Shape**: The Bag cells occupied by an Item relative to its placement.
- **Nearby Items**: Items whose occupied cells share an edge.
- **Activation**: An Item becoming ready and producing its Battle effects.
- **Effect**: One ordered consequence of an Item, such as damage, healing, blocking, modifying another Item, or attempting a fall.
- **Archetype**: A strategic role: Aggression, Defense, Scaling, Control, or Support.
- **Dominant Archetype**: The unique Archetype occupying the most Bag cells. A tied Bag has no Dominant Archetype.
- **Block**: A Hero-held reserve spent to absorb damage after armor has reduced it.
- **Fall Attempt**: A request to remove an Item that protective Items may prevent.
- **Fallen Item**: An Item removed from its Bag. Its effects stop before the rest of that Tick resolves.
- **Tick**: One ordered Battle step. A live Tick represents 100 ms and resolves as two 50 ms Ranks; a simulation can resolve it without waiting.
- **Rank**: One 50 ms half-step. Each Tick runs the attacker (left Hero) on its even Rank, then the defender (right Hero) on its odd Rank, so opposing Items never resolve in the same instant. Item cadences stay authored in 100 ms Ticks (1 Tick = 2 Ranks).
- **Defender Guard**: The opening Block reserve the defending (right) Hero starts each Battle with, calibrated so the attacker's even-Rank timing edge nets to an even Battle.
