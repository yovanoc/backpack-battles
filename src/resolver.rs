use std::collections::VecDeque;

use crate::{
    BattleEvent, Combat, DamageMode, Effect, EffectKind, FallCause, ItemRef, ItemTarget, Side,
};

/// A hero at or below this percentage of max health is in "last stand": it
/// takes reduced weapon damage, compressing blowouts into closer fights and
/// opening comeback windows. Raw poison and health-loss ignore it by design.
const RALLY_THRESHOLD_PCT: u32 = 30;
/// Fraction (1/N) of weapon damage shrugged off while in last stand.
const RALLY_MITIGATION_DIVISOR: u16 = 3;

impl Combat {
    pub(crate) fn resolve_effects(&mut self, effects: Vec<Effect>, events: &mut Vec<BattleEvent>) {
        let mut queue = VecDeque::from(effects);
        while let Some(effect) = queue.pop_front() {
            if self.item(effect.source).is_none() {
                continue;
            }
            queue.extend(self.apply_effect(effect, events));
        }
    }

    fn apply_effect(&mut self, effect: Effect, events: &mut Vec<BattleEvent>) -> Vec<Effect> {
        match effect.kind {
            EffectKind::Damage {
                target,
                amount,
                mode,
            } => self.apply_damage(
                effect.source,
                Damage {
                    target,
                    amount,
                    mode,
                },
                events,
            ),
            EffectKind::LoseHealth { target, amount } => {
                let hero = self.hero_mut(target);
                let lost = hero.health.min(amount);
                hero.health -= lost;
                events.push(BattleEvent::HealthLost {
                    source: effect.source,
                    target,
                    amount: lost,
                });
                Vec::new()
            }
            EffectKind::Heal { target, amount } => {
                let hero = self.hero_mut(target);
                let healed = hero.max_health().saturating_sub(hero.health).min(amount);
                hero.health = hero.health.saturating_add(healed);
                events.push(BattleEvent::Healed {
                    source: effect.source,
                    target,
                    amount: healed,
                });
                Vec::new()
            }
            EffectKind::GainBlock {
                target,
                amount,
                maximum,
            } => {
                let hero = self.hero_mut(target);
                hero.block = hero.block.saturating_add(amount).min(maximum);
                events.push(BattleEvent::BlockChanged {
                    hero: target,
                    block: hero.block,
                });
                Vec::new()
            }
            EffectKind::ModifySpeed {
                target,
                basis_points,
            } => {
                for item_ref in self.targets(target) {
                    let Some(item) = self.item_mut(item_ref) else {
                        continue;
                    };
                    let total = item.add_speed(basis_points);
                    events.push(BattleEvent::ItemSpeedChanged {
                        item: item_ref,
                        basis_points: total,
                    });
                }
                Vec::new()
            }
            EffectKind::AttemptFall {
                target,
                cause,
                chance,
            } => {
                self.fall_telemetry.attempts += 1;
                let roll_passes = match chance {
                    Some(one_in) => self.random.one_in(one_in),
                    None => true,
                };
                let targets = self.targets(target);
                if targets.is_empty() {
                    self.fall_telemetry.no_target += 1;
                } else {
                    self.fall_telemetry.valid_targets += 1;
                }
                if roll_passes {
                    for item_ref in targets {
                        self.attempt_fall(item_ref, cause, events);
                    }
                } else {
                    self.fall_telemetry.chance_miss += 1;
                }
                Vec::new()
            }
            EffectKind::Consume { target } => {
                let Some(item) = self.hero_mut(target.side).bag.remove(target.id) else {
                    return Vec::new();
                };
                self.hero_mut(target.side).clamp_health_to_max();
                events.push(BattleEvent::ItemConsumed {
                    item: target,
                    kind: item.kind(),
                });
                Vec::new()
            }
            EffectKind::ApplyPoison { target, stacks } => {
                let hero = self.hero_mut(target);
                hero.poison = hero.poison.saturating_add(stacks);
                events.push(BattleEvent::Poisoned {
                    target,
                    stacks: hero.poison,
                });
                // Defense's charter: attrition, not survival. Retaliators punish
                // the poisoner once per application, giving Defense a real answer
                // to Scaling's armor-and-block-ignoring poison.
                self.hero(target)
                    .bag
                    .retaliators()
                    .map(|(id, amount)| {
                        Effect::new(
                            ItemRef { side: target, id },
                            EffectKind::Damage {
                                target: target.opponent(),
                                amount,
                                mode: DamageMode::Retaliation,
                            },
                        )
                    })
                    .collect()
            }
            EffectKind::CleansePoison { target, amount } => {
                let hero = self.hero_mut(target);
                hero.poison = hero.poison.saturating_sub(amount);
                events.push(BattleEvent::PoisonCleansed {
                    target,
                    remaining: hero.poison,
                });
                Vec::new()
            }
            EffectKind::ShiftCharge { target, ticks } => {
                for item_ref in self.targets(target) {
                    if let Some(item) = self.item_mut(item_ref) {
                        item.shift_charge(ticks);
                    }
                }
                Vec::new()
            }
        }
    }

    fn apply_damage(
        &mut self,
        source: ItemRef,
        damage: Damage,
        events: &mut Vec<BattleEvent>,
    ) -> Vec<Effect> {
        let source_kind = self.item(source).map(|item| item.kind());
        let adjacent_bonus = source_kind.filter(|kind| kind.is_weapon()).map_or(0, |_| {
            self.hero(source.side).bag.adjacent_damage_bonus(source.id)
        });
        // Retaliation is raw (design layer-5): it ignores armor and block.
        let armor = match damage.mode {
            DamageMode::Normal => self.hero(damage.target).armor(),
            DamageMode::Piercing | DamageMode::Retaliation => 0,
        };
        let after_armor = damage
            .amount
            .saturating_add(adjacent_bonus)
            .saturating_sub(armor);
        let max_health = self.hero(damage.target).max_health();
        let hero = self.hero_mut(damage.target);
        let blocked = match damage.mode {
            DamageMode::Retaliation => 0,
            DamageMode::Normal | DamageMode::Piercing => hero.block.min(after_armor),
        };
        if blocked > 0 {
            hero.block -= blocked;
            events.push(BattleEvent::BlockChanged {
                hero: damage.target,
                block: hero.block,
            });
        }
        let raw = after_armor - blocked;
        let incoming =
            if u32::from(hero.health) * 100 <= u32::from(max_health) * RALLY_THRESHOLD_PCT {
                raw.saturating_sub(raw / RALLY_MITIGATION_DIVISOR)
            } else {
                raw
            };
        let lost = hero.health.min(incoming);
        hero.health -= lost;
        events.push(BattleEvent::DamageDealt {
            source,
            target: damage.target,
            mode: damage.mode,
            amount: lost,
        });

        match damage.mode {
            DamageMode::Normal => self
                .hero(damage.target)
                .bag
                .retaliators()
                .map(|(id, amount)| {
                    Effect::new(
                        ItemRef {
                            side: damage.target,
                            id,
                        },
                        EffectKind::Damage {
                            target: source.side,
                            amount,
                            mode: DamageMode::Retaliation,
                        },
                    )
                })
                .collect(),
            DamageMode::Piercing | DamageMode::Retaliation => Vec::new(),
        }
    }

    pub(crate) fn attempt_fall(
        &mut self,
        target: ItemRef,
        cause: FallCause,
        events: &mut Vec<BattleEvent>,
    ) {
        let Some(kind) = self.item(target).map(|item| item.kind()) else {
            return;
        };
        if !kind.can_fall() {
            return;
        }
        if let Some(protector) = self.hero(target.side).bag.protector(target.id) {
            if matches!(cause, FallCause::Forced { .. }) {
                self.fall_telemetry.prevented += 1;
            }
            events.push(BattleEvent::FallPrevented {
                item: target,
                by: ItemRef {
                    side: target.side,
                    id: protector,
                },
            });
            return;
        }
        self.hero_mut(target.side).bag.remove(target.id);
        if matches!(cause, FallCause::Forced { .. }) {
            self.fall_telemetry.succeeded += 1;
        }
        self.hero_mut(target.side).clamp_health_to_max();
        events.push(BattleEvent::ItemFell {
            item: target,
            kind,
            cause,
        });
    }

    fn targets(&self, target: ItemTarget) -> Vec<ItemRef> {
        match target {
            ItemTarget::Adjacent(source) => self
                .hero(source.side)
                .bag
                .adjacent_ids(source.id)
                .map(|id| ItemRef {
                    side: source.side,
                    id,
                })
                .collect(),
            ItemTarget::LightestEdge { side, weapons_only } => self
                .hero(side)
                .bag
                .lightest_edge(weapons_only)
                .map(|id| vec![ItemRef { side, id }])
                .unwrap_or_default(),
            ItemTarget::SoonestActivation { side } => self
                .hero(side)
                .bag
                .items()
                .iter()
                .filter(|item| item.charge().is_some())
                .min_by_key(|item| (item.charge().unwrap_or(u32::MAX), item.id()))
                .map(|item| {
                    vec![ItemRef {
                        side,
                        id: item.id(),
                    }]
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Copy)]
struct Damage {
    target: Side,
    amount: u16,
    mode: DamageMode,
}
