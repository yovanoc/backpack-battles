use std::process::ExitCode;

use backpack_battles::{
    CONTESTED_MAX_COUNTER, CYCLE_EDGE_WINRATE, CYCLE_MIN_SCC, MetaConfig, NO_WELL_MIN_COUNTER,
    ROSTER_MAX_PRESENCE, SUBSTANTIAL_MIN_DISTANCE, run_verdict,
};

pub(crate) fn run(config: MetaConfig) -> ExitCode {
    let report = run_verdict(&config);

    println!(
        "verdict: elite={} bags, seed={}, health={}, ticks={}",
        report.elite_size, config.seed, config.hero_health, config.tick_limit
    );
    println!();

    line(
        "no wells",
        report.pass_no_wells,
        &format!(
            "every bag has a counter >= {:.0}% (weakest best-counter {:.0}%)",
            NO_WELL_MIN_COUNTER * 100.0,
            report.min_best_counter * 100.0
        ),
        &format!(
            "{} bag(s) with no >= {:.0}% counter",
            report.bags_without_counter,
            NO_WELL_MIN_COUNTER * 100.0
        ),
    );
    line(
        "contested counters",
        report.pass_contested,
        &format!(
            "no counter exceeds {:.0}% (hardest counter {:.0}%)",
            CONTESTED_MAX_COUNTER * 100.0,
            report.max_best_counter * 100.0
        ),
        &format!(
            "{} execution counter(s) > {:.0}%",
            report.executions,
            CONTESTED_MAX_COUNTER * 100.0
        ),
    );
    line(
        "substantial counters",
        report.pass_substantial,
        &format!(
            "counters differ by >= {:.0}% (min {:.0}%, median {:.0}%)",
            SUBSTANTIAL_MIN_DISTANCE * 100.0,
            report.min_counter_distance * 100.0,
            report.median_counter_distance * 100.0
        ),
        &format!(
            "{} stat-check counter(s) < {:.0}% distance",
            report.stat_check_counters,
            SUBSTANTIAL_MIN_DISTANCE * 100.0
        ),
    );
    line(
        "cycles present",
        report.pass_cycles,
        &format!(
            "SCC >= {} on the >= {:.0}% beat-graph (largest {})",
            CYCLE_MIN_SCC,
            CYCLE_EDGE_WINRATE * 100.0,
            report.largest_scc
        ),
        &format!("acyclic ranking (largest SCC {})", report.largest_scc),
    );
    let roster_detail = if report.pass_roster {
        format!(
            "every item in 0 < presence <= {:.0}% of elite",
            ROSTER_MAX_PRESENCE * 100.0
        )
    } else {
        let dead: Vec<&str> = report.dead_items.iter().map(|kind| kind.name()).collect();
        let oppressive: Vec<String> = report
            .oppressive_items
            .iter()
            .map(|(kind, presence)| format!("{} {:.0}%", kind.name(), presence * 100.0))
            .collect();
        format!(
            "dead: [{}]; oppressive: [{}]",
            dead.join(", "),
            oppressive.join(", ")
        )
    };
    line(
        "living roster",
        report.pass_roster,
        &roster_detail,
        &roster_detail,
    );

    println!();
    if report.passed() {
        println!("VERDICT: PASS - the meta is a healthy non-transitive web");
        ExitCode::SUCCESS
    } else {
        println!("VERDICT: FAIL - see red criteria above");
        ExitCode::FAILURE
    }
}

fn line(name: &str, pass: bool, green: &str, red: &str) {
    let mark = if pass { "PASS" } else { "FAIL" };
    let detail = if pass { green } else { red };
    println!("[{mark}] {name:<22} {detail}");
}
