use std::process::ExitCode;

use backpack_battles::{Archetype, MetaConfig, run_meta};

// Presence thresholds from the designer's balancing notes: an item absent from
// every elite bag is dead; one above the ceiling is oppressive or too universal.
const DEAD: f64 = 0.0;
const OPPRESSIVE: f64 = 0.65;

pub(crate) fn run(config: MetaConfig) -> ExitCode {
    let report = run_meta(&config);

    println!(
        "meta: {} candidates, panel={}, elite={}, seed={}, health={}, ticks={}",
        report.candidates,
        report.panel,
        report.elite_size,
        config.seed,
        config.hero_health,
        config.tick_limit
    );
    println!(
        "elite fitness mean: {:.1}%; build diversity: {}/{} distinct elite builds",
        report.mean_elite_fitness * 100.0,
        report.distinct_signatures,
        report.elite_size
    );
    print!("elite archetype cell-share:");
    for archetype in Archetype::ALL {
        print!(
            " {}={:.0}%",
            archetype.name(),
            report.archetype_share[archetype as usize] * 100.0
        );
    }
    println!();
    println!(
        "(dead = in 0 elite bags; oppressive = in >{:.0}% of them)",
        OPPRESSIVE * 100.0
    );
    println!();
    println!(
        "{:<16} {:<10} {:>5} {:>10} {:>8}  flag",
        "item", "archetype", "cells", "presence", "elite"
    );

    for stat in &report.presence {
        let flag = if stat.presence <= DEAD {
            "DEAD"
        } else if stat.presence > OPPRESSIVE {
            "oppressive"
        } else {
            ""
        };
        println!(
            "{:<16} {:<10} {:>5} {:>9.1}% {:>8}  {}",
            stat.kind.name(),
            stat.kind.archetype().name(),
            stat.kind.shape().len(),
            stat.presence * 100.0,
            stat.elite_count,
            flag
        );
    }

    ExitCode::SUCCESS
}
