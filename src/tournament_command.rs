use std::process::ExitCode;

use backpack_battles::{Archetype, MetaConfig, run_tournament};

pub(crate) fn run(config: MetaConfig) -> ExitCode {
    let report = run_tournament(&config);

    println!(
        "tournament: {} entrants, {} rounds/gen, {} generations = {} fights (seed={}, health={}, ticks={})",
        report.entrants,
        report.rounds,
        report.generations,
        report.fights,
        config.seed,
        config.hero_health,
        config.tick_limit
    );
    println!(
        "champion: {} wins/{} rounds; {} items",
        report.champion_wins,
        report.rounds,
        report.champion.len()
    );
    for kind in &report.champion {
        println!("  - {} ({})", kind.name(), kind.archetype().name());
    }
    print!("finalist archetype cell-share:");
    for archetype in Archetype::ALL {
        print!(
            " {}={:.0}%",
            archetype.name(),
            report.archetype_share[archetype as usize] * 100.0
        );
    }
    println!();
    println!();
    println!(
        "item presence across the top {} finalists:",
        report.finalists
    );
    println!(
        "{:<16} {:<10} {:>5} {:>9}",
        "item", "archetype", "cells", "presence"
    );
    for stat in &report.presence {
        println!(
            "{:<16} {:<10} {:>5} {:>8.0}%",
            stat.kind.name(),
            stat.kind.archetype().name(),
            stat.kind.shape().len(),
            stat.presence * 100.0
        );
    }

    ExitCode::SUCCESS
}
