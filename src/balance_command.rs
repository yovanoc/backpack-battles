use std::process::ExitCode;

use backpack_battles::{Archetype, BalanceConfig, ItemStat, run_balance};

pub(crate) fn run(config: BalanceConfig) -> ExitCode {
    let seed = config.seed;
    let ticks = config.tick_limit;
    let health = config.hero_health;
    let rotate = config.allow_rotation;
    let report = match run_balance(&config) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "balance: {} battles, seed={seed}, tick_limit={ticks}, health={health}, rotate={rotate}, campaign={}",
        report.battles,
        config.campaign_mode.name()
    );
    let draw_percentage = if report.battles == 0 {
        0.0
    } else {
        report.draws as f64 / report.battles as f64 * 100.0
    };
    println!("draws: {} ({draw_percentage:.1}%)", report.draws);
    println!(
        "sides: left={:.1}%, right={:.1}%, bias={:+.2}pp; duration mean={:.1}, p50={}, p90={}, p99={} ticks",
        report.left_score_rate() * 100.0,
        (1.0 - report.left_score_rate()) * 100.0,
        report.side_bias_percentage_points(),
        report.mean_ticks(),
        report.duration_p50(),
        report.duration_p90(),
        report.duration_p99()
    );
    println!(
        "interest: swing rate={:.1}%, lead changes mean={:.2}, decided at {:.0}% of fight",
        report.swing_rate() * 100.0,
        report.mean_lead_changes(),
        report.mean_decided_fraction() * 100.0
    );
    if report.mirrored_battles > 0 {
        println!(
            "paired side bias: {:+.2}pp across {} original+mirrored battles",
            report.paired_side_bias_percentage_points(),
            report.battles + report.mirrored_battles
        );
    }
    let falls = report.fall_telemetry;
    println!(
        "control: attempts={}, targets={}, none={}, misses={}, prevented={}, falls={}",
        falls.attempts,
        falls.valid_targets,
        falls.no_target,
        falls.chance_miss,
        falls.prevented,
        falls.succeeded
    );
    println!(
        "ranks: shared_activation={}, shared_lethal={}",
        falls.shared_activation_ranks, falls.shared_lethal_ranks
    );
    println!("score% counts draws as half; swap% uses matched size-controlled substitutions");
    println!("ranked by Hoeffding 95% lower bound (score% when no swap partner exists)");
    println!();
    println!(
        "{:<16} {:<10} {:>5} {:>10} {:>7} {:>7} {:>9} {:>9} {:>9} {:>8}",
        "item", "archetype", "cells", "bags", "score%", "swap%", "wins", "losses", "draws", "swaps"
    );

    let mut ranked: Vec<&ItemStat> = report.stats.iter().filter(|stat| stat.bags > 0).collect();
    ranked.sort_by(|a, b| b.rank_score().total_cmp(&a.rank_score()));
    for stat in ranked {
        let swap_percentage = if stat.swap_wins + stat.swap_losses == 0 {
            "      -".to_string()
        } else {
            format!("{:>6.1}%", stat.swap_score_rate() * 100.0)
        };
        println!(
            "{:<16} {:<10} {:>5} {:>10} {:>6.1}% {} {:>9} {:>9} {:>9} {:>8}",
            stat.kind.name(),
            stat.kind.archetype().name(),
            stat.kind.shape().len(),
            stat.bags,
            stat.score_rate() * 100.0,
            swap_percentage,
            stat.wins,
            stat.losses,
            stat.draws,
            stat.swaps()
        );
    }

    println!();
    println!(
        "archetype matchups: left score% ({} battles excluded for tied dominance)",
        report.unclassified_matchups
    );
    print!("{:<12}", "");
    for archetype in Archetype::ALL {
        print!(" {:>10}", archetype.name());
    }
    println!();
    for left in Archetype::ALL {
        print!("{:<12}", left.name());
        for right in Archetype::ALL {
            let matchup = report.matchups[left as usize][right as usize];
            if matchup.battles == 0 {
                print!(" {:>10}", "-");
            } else {
                print!(" {:>9.1}%", matchup.left_score_rate() * 100.0);
            }
        }
        println!();
    }

    ExitCode::SUCCESS
}
