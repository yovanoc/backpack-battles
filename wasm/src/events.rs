use backpack_battles::{BattleEvent, DamageMode, FallCause, ItemRef, Side};

use crate::dto::{DamageModeView, EventKind, EventView, FallCauseView, SideView};

fn side_view(side: Side) -> SideView {
    match side {
        Side::Left => SideView::Left,
        Side::Right => SideView::Right,
    }
}

fn item_ref_cell(item: ItemRef) -> [u8; 2] {
    let cell = item.id.cell();
    [cell.x, cell.y]
}

pub(crate) fn event_view(event: &BattleEvent) -> EventView {
    match event {
        BattleEvent::ItemActivated { item, kind } => EventView {
            kind: EventKind::Activated,
            side: Some(side_view(item.side)),
            item: Some(item_ref_cell(*item)),
            item_kind: Some(kind.name().to_string()),
            ..Default::default()
        },
        BattleEvent::DamageDealt {
            source,
            target,
            mode,
            amount,
        } => EventView {
            kind: EventKind::Damage,
            side: Some(side_view(*target)),
            item: Some(item_ref_cell(*source)),
            amount: Some(*amount),
            mode: Some(damage_mode_view(*mode)),
            ..Default::default()
        },
        BattleEvent::HealthLost {
            source,
            target,
            amount,
        } => EventView {
            kind: EventKind::HealthLost,
            side: Some(side_view(*target)),
            item: Some(item_ref_cell(*source)),
            amount: Some(*amount),
            ..Default::default()
        },
        BattleEvent::Healed {
            source,
            target,
            amount,
        } => EventView {
            kind: EventKind::Healed,
            side: Some(side_view(*target)),
            item: Some(item_ref_cell(*source)),
            amount: Some(*amount),
            ..Default::default()
        },
        BattleEvent::BlockChanged { hero, block } => EventView {
            kind: EventKind::Block,
            side: Some(side_view(*hero)),
            amount: Some(*block),
            ..Default::default()
        },
        BattleEvent::ItemSpeedChanged { item, basis_points } => EventView {
            kind: EventKind::Speed,
            side: Some(side_view(item.side)),
            item: Some(item_ref_cell(*item)),
            amount: Some(*basis_points),
            ..Default::default()
        },
        BattleEvent::ItemFell { item, kind, cause } => EventView {
            kind: EventKind::Fell,
            side: Some(side_view(item.side)),
            item: Some(item_ref_cell(*item)),
            item_kind: Some(kind.name().to_string()),
            cause: Some(fall_cause_view(cause)),
            ..Default::default()
        },
        BattleEvent::FallPrevented { item, by } => EventView {
            kind: EventKind::FallPrevented,
            side: Some(side_view(item.side)),
            item: Some(item_ref_cell(*item)),
            by: Some(item_ref_cell(*by)),
            ..Default::default()
        },
        BattleEvent::ItemConsumed { item, kind } => EventView {
            kind: EventKind::Consumed,
            side: Some(side_view(item.side)),
            item: Some(item_ref_cell(*item)),
            item_kind: Some(kind.name().to_string()),
            ..Default::default()
        },
        BattleEvent::Poisoned { target, stacks } => EventView {
            kind: EventKind::Poisoned,
            side: Some(side_view(*target)),
            amount: Some(*stacks),
            ..Default::default()
        },
        BattleEvent::PoisonDamage { target, amount } => EventView {
            kind: EventKind::PoisonDamage,
            side: Some(side_view(*target)),
            amount: Some(*amount),
            ..Default::default()
        },
        BattleEvent::PoisonCleansed { target, remaining } => EventView {
            kind: EventKind::PoisonCleansed,
            side: Some(side_view(*target)),
            amount: Some(*remaining),
            ..Default::default()
        },
    }
}

fn damage_mode_view(mode: DamageMode) -> DamageModeView {
    match mode {
        DamageMode::Normal => DamageModeView::Normal,
        DamageMode::Piercing => DamageModeView::Piercing,
        DamageMode::Retaliation => DamageModeView::Retaliation,
    }
}

fn fall_cause_view(cause: &FallCause) -> FallCauseView {
    match cause {
        FallCause::Natural => FallCauseView::Natural,
        FallCause::Forced { .. } => FallCauseView::Forced,
    }
}
