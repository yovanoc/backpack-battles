use backpack_battles::{BAG_WIDTH, Bag, BattleEvent, BattleResult, FallCause, Item, TickReport};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

use super::{
    BLUE, BORDER, Color, GREEN, MUTED, PURPLE, RED, TEXT, YELLOW, archetype_color,
    status_color_for_result,
};
use crate::watch::BagLayout;

fn item_color(layout: &BagLayout, index: usize) -> Color {
    archetype_color(layout.items[index].1.archetype())
}

pub(super) fn bag_lines(layout: &BagLayout, bag: &Bag, compact: bool) -> Vec<Line<'static>> {
    let width = usize::from(BAG_WIDTH);
    let mut lines = Vec::new();
    for row in layout.cells.chunks(width) {
        let mut spans = vec![Span::raw("  ")];
        for cell in row {
            match cell {
                None => spans.push(Span::styled("· ", Style::new().fg(BORDER))),
                Some(index) if layout.is_alive(*index, bag) => spans.push(Span::styled(
                    format!("{} ", item_glyph(*index)),
                    Style::new().fg(item_color(layout, *index)).bold(),
                )),
                Some(_) => spans.push(Span::styled(
                    "× ",
                    Style::new().fg(RED).add_modifier(Modifier::DIM),
                )),
            }
        }
        lines.push(Line::from(spans));
    }
    if !compact {
        lines.push(Line::raw(""));
    }
    for (index, (id, kind)) in layout.items.iter().enumerate() {
        let alive = layout.is_alive(index, bag);
        let color = item_color(layout, index);
        let glyph = if alive { item_glyph(index) } else { '×' };
        let label_style = if alive {
            Style::new().fg(color)
        } else {
            Style::new().fg(RED).add_modifier(Modifier::DIM)
        };
        let progress = bag
            .items()
            .iter()
            .find(|item| item.id() == *id)
            .and_then(Item::charge_progress);
        let mut spans = vec![Span::styled(
            format!("{glyph} {:<15}", kind.name()),
            label_style,
        )];
        spans.extend(charge_bar(progress, alive, color));
        lines.push(Line::from(spans));
    }
    lines
}

fn charge_bar(progress: Option<f64>, alive: bool, color: Color) -> Vec<Span<'static>> {
    const CELLS: usize = 6;
    if !alive {
        return vec![Span::styled(
            "fallen",
            Style::new().fg(RED).add_modifier(Modifier::DIM),
        )];
    }
    let Some(progress) = progress else {
        return vec![Span::styled("passive", Style::new().fg(MUTED))];
    };
    if progress >= 1.0 {
        return vec![
            Span::styled("▮".repeat(CELLS), Style::new().fg(GREEN).bold()),
            Span::styled(" ready", Style::new().fg(GREEN).bold()),
        ];
    }
    let filled = (progress * CELLS as f64).round() as usize;
    vec![
        Span::styled("▮".repeat(filled), Style::new().fg(color)),
        Span::styled("▯".repeat(CELLS - filled), Style::new().fg(BORDER)),
    ]
}

pub(in crate::watch) fn intro_line(seed: u64) -> Line<'static> {
    Line::from(vec![
        Span::styled("t000  ", Style::new().fg(MUTED)),
        Span::styled(
            format!("battle initialized with seed {seed}"),
            Style::new().fg(TEXT),
        ),
    ])
}

pub(in crate::watch) fn append_report(journal: &mut Vec<Line<'static>>, report: &TickReport) {
    if report.events.is_empty() {
        return;
    }
    journal.push(Line::from(vec![
        Span::styled(format!("t{:03}  ", report.tick), Style::new().fg(MUTED)),
        Span::styled(
            format!(
                "LEFT {}  //  RIGHT {}",
                report.left_health, report.right_health
            ),
            Style::new().fg(TEXT).bold(),
        ),
    ]));
    journal.extend(report.events.iter().map(event_line));
}

pub(in crate::watch) fn append_result(journal: &mut Vec<Line<'static>>, result: BattleResult) {
    journal.push(Line::raw(""));
    journal.push(Line::styled(
        format!(
            "{} after {} ticks",
            result.outcome.name().to_uppercase(),
            result.ticks
        ),
        Style::new().fg(status_color_for_result(result)).bold(),
    ));
}

fn event_line(event: &BattleEvent) -> Line<'static> {
    let (icon, text, color) = match event {
        BattleEvent::ItemActivated { item, kind } => (
            "◆",
            format!("{} {} activates", item.side.name(), kind.name()),
            BLUE,
        ),
        BattleEvent::DamageDealt {
            source,
            target,
            amount,
            mode,
        } => (
            "✷",
            format!(
                "{} hits {} for {amount} ({mode:?})",
                source.side.name(),
                target.name()
            ),
            RED,
        ),
        BattleEvent::HealthLost { target, amount, .. } => {
            ("✷", format!("{} loses {amount} health", target.name()), RED)
        }
        BattleEvent::Healed { target, amount, .. } => {
            ("♥", format!("{} heals {amount}", target.name()), GREEN)
        }
        BattleEvent::BlockChanged { hero, block } => {
            ("◈", format!("{} block becomes {block}", hero.name()), BLUE)
        }
        BattleEvent::ItemSpeedChanged { item, basis_points } => (
            "»",
            format!("{} speeds an item by {basis_points} bps", item.side.name()),
            YELLOW,
        ),
        BattleEvent::ItemFell { item, kind, cause } => {
            let cause = match cause {
                FallCause::Natural => "natural",
                FallCause::Forced { .. } => "forced",
            };
            (
                "▼",
                format!("{} loses {} ({cause})", item.side.name(), kind.name()),
                PURPLE,
            )
        }
        BattleEvent::FallPrevented { by, .. } => {
            ("▲", format!("{} prevents a fall", by.side.name()), YELLOW)
        }
        BattleEvent::ItemConsumed { item, kind } => (
            "·",
            format!("{} consumes {}", item.side.name(), kind.name()),
            MUTED,
        ),
    };
    Line::from(vec![
        Span::styled("   ", Style::new()),
        Span::styled(format!("{icon} "), Style::new().fg(color)),
        Span::styled(text, Style::new().fg(color)),
    ])
}

fn item_glyph(index: usize) -> char {
    char::from_digit(u32::try_from(index).unwrap_or(35) + 1, 36).unwrap_or('?')
}
