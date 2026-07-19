use crate::{BattleEvent, FallCause, FallTelemetry, Hero, Item, ItemRef, Side, behavior, rng::Rng};

#[derive(Debug)]
pub(crate) struct Combat {
    pub(crate) left: Hero,
    pub(crate) right: Hero,
    pub(crate) random: Rng,
    pub(crate) fall_telemetry: FallTelemetry,
}

impl Combat {
    pub(crate) const fn new(left: Hero, right: Hero, seed: u64) -> Self {
        Self {
            left,
            right,
            random: Rng::new(seed),
            fall_telemetry: FallTelemetry {
                attempts: 0,
                valid_targets: 0,
                no_target: 0,
                chance_miss: 0,
                prevented: 0,
                succeeded: 0,
                shared_activation_ranks: 0,
                shared_lethal_ranks: 0,
            },
        }
    }

    pub(crate) const fn hero(&self, side: Side) -> &Hero {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }

    pub(crate) const fn hero_mut(&mut self, side: Side) -> &mut Hero {
        match side {
            Side::Left => &mut self.left,
            Side::Right => &mut self.right,
        }
    }

    pub(crate) fn item(&self, item_ref: ItemRef) -> Option<&Item> {
        self.hero(item_ref.side).bag.item(item_ref.id)
    }

    pub(crate) fn item_mut(&mut self, item_ref: ItemRef) -> Option<&mut Item> {
        self.hero_mut(item_ref.side).bag.item_mut(item_ref.id)
    }

    pub(crate) fn fill_item_refs(&self, side: Side, out: &mut Vec<ItemRef>) {
        out.clear();
        for item in self.hero(side).bag.items() {
            out.push(ItemRef {
                side,
                id: item.id(),
            });
        }
    }

    pub(crate) fn fill_ready_refs(&self, side: Side, out: &mut Vec<ItemRef>) {
        out.clear();
        if self.hero(side).health == 0 {
            return;
        }
        for item in self.hero(side).bag.items() {
            if item.is_ready() {
                out.push(ItemRef {
                    side,
                    id: item.id(),
                });
            }
        }
    }

    pub(crate) fn advance_charges(&mut self, side: Side) {
        for item in self.hero_mut(side).bag.items_mut() {
            item.advance_charge();
        }
    }

    pub(crate) fn resolve_battle_start(&mut self, side: Side, events: &mut Vec<BattleEvent>) {
        let refs: Vec<ItemRef> = self
            .hero(side)
            .bag
            .items()
            .iter()
            .map(|item| ItemRef {
                side,
                id: item.id(),
            })
            .collect();
        for source in refs {
            let Some(item) = self.item_mut(source) else {
                continue;
            };
            let effects = behavior::battle_start(item, source);
            self.resolve_effects(effects, events);
            if self.is_finished() {
                return;
            }
        }
    }

    pub(crate) fn resolve_tick(&mut self, source: ItemRef, events: &mut Vec<BattleEvent>) {
        let hero = self.hero(source.side);
        let (health, max_health) = (hero.health, hero.max_health());
        let Some(item) = self.item_mut(source) else {
            return;
        };
        let effects = behavior::tick(item, source, health, max_health);
        self.resolve_effects(effects, events);
    }

    pub(crate) fn resolve_activation(&mut self, source: ItemRef, events: &mut Vec<BattleEvent>) {
        // A hero killed earlier in this rank cannot still activate.
        if self.is_finished() {
            return;
        }
        let Some(kind) = self.item(source).map(Item::kind) else {
            return;
        };
        let Some(item) = self.item_mut(source) else {
            return;
        };
        let effects = behavior::activate(item, source);
        events.push(BattleEvent::ItemActivated { item: source, kind });
        self.resolve_effects(effects, events);
    }

    pub(crate) fn resolve_natural_falls(
        &mut self,
        side: Side,
        tick: u16,
        events: &mut Vec<BattleEvent>,
        scratch: &mut Vec<ItemRef>,
    ) {
        scratch.clear();
        for item in self.hero(side).bag.items() {
            if let Some(fall) = item.kind().natural_fall()
                && tick.is_multiple_of(fall.every)
            {
                scratch.push(ItemRef {
                    side,
                    id: item.id(),
                });
            }
        }
        for &item_ref in scratch.iter() {
            let Some(fall) = self
                .item(item_ref)
                .and_then(|item| item.kind().natural_fall())
            else {
                continue;
            };
            if self.random.one_in(fall.one_in) {
                self.attempt_fall(item_ref, FallCause::Natural, events);
            }
        }
    }

    /// Apply `side`'s standing poison at the top of its rank: raw self-damage
    /// equal to the stack count (ignores armor and block), then one stack
    /// decays. Returns whether it ended the battle.
    pub(crate) fn resolve_poison(&mut self, side: Side, events: &mut Vec<BattleEvent>) -> bool {
        let hero = self.hero_mut(side);
        if hero.poison == 0 {
            return false;
        }
        let amount = hero.health.min(hero.poison);
        hero.health -= amount;
        hero.poison -= 1;
        events.push(BattleEvent::PoisonDamage {
            target: side,
            amount,
        });
        self.is_finished()
    }

    /// Resolve one 50ms rank for `side` at authored `tick`: that side's due
    /// natural falls, per-tick behavior, one charge advance, then its ready
    /// activations in anchored ItemId order. Stops the moment either hero dies
    /// so a lethal blow cancels any later same-rank activation. Returns whether
    /// this side landed a primary activation and whether the rank ended the
    /// battle, so the caller can guard the interleaving invariants.
    pub(crate) fn resolve_rank(
        &mut self,
        side: Side,
        tick: u16,
        events: &mut Vec<BattleEvent>,
        scratch: &mut Vec<ItemRef>,
    ) -> RankOutcome {
        if self.resolve_poison(side, events) {
            return RankOutcome {
                activated: false,
                ended: true,
            };
        }
        self.resolve_natural_falls(side, tick, events, scratch);
        if self.is_finished() {
            return RankOutcome {
                activated: false,
                ended: true,
            };
        }
        self.fill_item_refs(side, scratch);
        let refs: Vec<ItemRef> = scratch.clone();
        for source in refs {
            self.resolve_tick(source, events);
            if self.is_finished() {
                return RankOutcome {
                    activated: false,
                    ended: true,
                };
            }
        }
        self.advance_charges(side);
        self.fill_ready_refs(side, scratch);
        let ready: Vec<ItemRef> = scratch.clone();
        let mut activated = false;
        for source in ready {
            let before = events.len();
            self.resolve_activation(source, events);
            activated |= events[before..]
                .iter()
                .any(|event| matches!(event, BattleEvent::ItemActivated { .. }));
            if self.is_finished() {
                return RankOutcome {
                    activated,
                    ended: true,
                };
            }
        }
        RankOutcome {
            activated,
            ended: false,
        }
    }

    pub(crate) const fn is_finished(&self) -> bool {
        self.left.health == 0 || self.right.health == 0
    }
}

/// What one rank produced, so the caller can guard the interleaving invariant.
#[derive(Clone, Copy, Default)]
pub(crate) struct RankOutcome {
    pub(crate) activated: bool,
    pub(crate) ended: bool,
}
