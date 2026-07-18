use std::process::ExitCode;

use backpack_battles::ItemKind;

pub(crate) fn run() -> ExitCode {
    println!("item catalog:");
    for kind in ItemKind::ALL {
        let timing = match (kind.first_activation(), kind.cadence()) {
            (Some(first), Some(cadence)) => format!("first {first}, every {cadence}"),
            _ => "passive".to_string(),
        };
        println!(
            "{:<16} {:<10} {:>2} cells  {:<20} {}",
            kind.name(),
            kind.archetype().name(),
            kind.shape().len(),
            timing,
            kind.effect_description()
        );
    }

    let mut timeline = ItemKind::ALL.to_vec();
    timeline.sort_by_key(|kind| (kind.first_activation().unwrap_or(u16::MAX), kind.name()));
    println!();
    println!("opening timeline:");
    for kind in timeline {
        if let Some(tick) = kind.first_activation() {
            println!("tick {tick:>3}: {}", kind.name());
        }
    }
    ExitCode::SUCCESS
}
