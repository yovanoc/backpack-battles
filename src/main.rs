use std::{process::ExitCode, thread};

use backpack_battles::{
    BalanceConfig, Battle, BattleConfig, BattleEvent, BattleUpdate, CampaignMode, FallCause,
    MetaConfig, TICK_DURATION, demo_heroes,
};
use clap::{Parser, Subcommand, ValueEnum};

mod balance_command;
mod catalog_command;
mod meta_command;
mod verdict_command;
mod watch;

#[derive(Parser)]
#[command(
    name = "backpack-battles",
    about = "Deterministic backpack autobattler",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum CampaignArg {
    Random,
    Pure,
    Hybrid,
    Elite,
}

impl From<CampaignArg> for CampaignMode {
    fn from(value: CampaignArg) -> Self {
        match value {
            CampaignArg::Random => Self::Random,
            CampaignArg::Pure => Self::Pure,
            CampaignArg::Hybrid => Self::Hybrid,
            CampaignArg::Elite => Self::Elite,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Show exact item rules and opening activation order.
    Catalog,
    /// Run a single deterministic battle between the demo heroes.
    Battle {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u16).range(1..=600))]
        ticks: u16,
        /// Wait 100 ms per tick instead of resolving instantly.
        #[arg(long)]
        live: bool,
    },
    /// Generate random bags and simulate many battles to surface overpowered items.
    Balance {
        #[arg(long, default_value_t = 100_000)]
        battles: u64,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u16).range(1..=600))]
        ticks: u16,
        /// Base health each generated hero starts with.
        #[arg(long, default_value_t = 100)]
        health: u16,
        /// Place every item upright instead of using random rotations.
        #[arg(long)]
        no_rotate: bool,
        /// Replay every matchup with Bags swapped to isolate engine side bias.
        #[arg(long)]
        mirror: bool,
        #[arg(long, value_enum, default_value_t = CampaignArg::Random)]
        campaign: CampaignArg,
    },
    /// Watch a battle in a full-screen terminal UI.
    Watch {
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u16).range(1..=600))]
        ticks: u16,
        /// Playback speed multiplier (2.0 = twice as fast as real time).
        #[arg(long, default_value_t = 1.0, value_parser = watch::parse_speed)]
        speed: f64,
    },
    /// Draft many bags, select the winning elite, and report item presence and
    /// build diversity - the mixed-bag meta health, not pure archetypes.
    Meta {
        #[arg(long, default_value_t = 400)]
        candidates: u64,
        #[arg(long, default_value_t = 96)]
        panel: u64,
        #[arg(long, default_value_t = 40)]
        elite: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u16).range(1..=600))]
        ticks: u16,
        #[arg(long, default_value_t = 100)]
        health: u16,
        #[arg(long)]
        no_rotate: bool,
    },
    /// Grade the meta against s17n's pass/fail balance criteria on the elite
    /// counter-graph: no wells, contested counters, substantial counters,
    /// cycles present, living roster.
    Verdict {
        #[arg(long, default_value_t = 400)]
        candidates: u64,
        #[arg(long, default_value_t = 96)]
        panel: u64,
        #[arg(long, default_value_t = 32)]
        elite: usize,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 600, value_parser = clap::value_parser!(u16).range(1..=600))]
        ticks: u16,
        #[arg(long, default_value_t = 100)]
        health: u16,
        #[arg(long)]
        no_rotate: bool,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Commands::Catalog => catalog_command::run(),
        Commands::Battle { seed, ticks, live } => run_battle(seed, ticks, live),
        Commands::Balance {
            battles,
            seed,
            ticks,
            health,
            no_rotate,
            mirror,
            campaign,
        } => balance_command::run(BalanceConfig {
            battles,
            seed,
            tick_limit: ticks,
            hero_health: health,
            allow_rotation: !no_rotate,
            mirror_sides: mirror,
            campaign_mode: campaign.into(),
        }),
        Commands::Watch { seed, ticks, speed } => watch::run(seed, ticks, speed),
        Commands::Meta {
            candidates,
            panel,
            elite,
            seed,
            ticks,
            health,
            no_rotate,
        } => meta_command::run(MetaConfig {
            candidates,
            panel,
            seed,
            tick_limit: ticks,
            hero_health: health,
            allow_rotation: !no_rotate,
            elite_size: elite,
        }),
        Commands::Verdict {
            candidates,
            panel,
            elite,
            seed,
            ticks,
            health,
            no_rotate,
        } => verdict_command::run(MetaConfig {
            candidates,
            panel,
            seed,
            tick_limit: ticks,
            hero_health: health,
            allow_rotation: !no_rotate,
            elite_size: elite,
        }),
    }
}

fn run_battle(seed: u64, ticks: u16, live: bool) -> ExitCode {
    let config = match BattleConfig::new(ticks, seed) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let (left, right) = demo_heroes(seed);
    println!(
        "battle: {} vs {}, seed={}, tick_limit={}, live={live}",
        left.name(),
        right.name(),
        config.seed(),
        config.tick_limit()
    );
    println!("left bag [{}]:", left.name());
    print!("{}", left.bag());
    println!("right bag [{}]:", right.name());
    print!("{}", right.bag());
    let mut battle = Battle::new(left, right, config);
    loop {
        match battle.advance() {
            BattleUpdate::Tick(report) => {
                for event in &report.events {
                    print_event(report.tick, event);
                }
                if live {
                    thread::sleep(TICK_DURATION);
                }
            }
            BattleUpdate::Finished(result) => {
                println!(
                    "result: {} after {} ticks (left={} hp, right={} hp)",
                    result.outcome.name(),
                    result.ticks,
                    result.left_health,
                    result.right_health
                );
                return ExitCode::SUCCESS;
            }
        }
    }
}

fn print_event(tick: u16, event: &BattleEvent) {
    match event {
        BattleEvent::ItemActivated { item, kind } => println!(
            "tick {tick:03}: {} hero activated {}",
            item.side.name(),
            kind.name()
        ),
        BattleEvent::DamageDealt {
            source,
            target,
            mode,
            amount,
        } => println!(
            "tick {tick:03}: {} dealt {amount} to {} ({mode:?})",
            source.side.name(),
            target.name()
        ),
        BattleEvent::HealthLost {
            source,
            target,
            amount,
        } => println!(
            "tick {tick:03}: {} drained {amount} health from {}",
            source.side.name(),
            target.name()
        ),
        BattleEvent::Healed {
            source,
            target,
            amount,
        } => println!(
            "tick {tick:03}: {} healed {} for {amount}",
            source.side.name(),
            target.name()
        ),
        BattleEvent::BlockChanged { hero, block } => {
            println!("tick {tick:03}: {} block is now {block}", hero.name())
        }
        BattleEvent::ItemSpeedChanged { item, basis_points } => println!(
            "tick {tick:03}: {} item at ({}, {}) has +{basis_points} bps speed",
            item.side.name(),
            item.id.cell().x,
            item.id.cell().y
        ),
        BattleEvent::ItemFell { item, kind, cause } => {
            let cause = match cause {
                FallCause::Natural => "natural",
                FallCause::Forced { .. } => "forced",
            };
            println!(
                "tick {tick:03}: {} hero lost {} ({cause})",
                item.side.name(),
                kind.name()
            );
        }
        BattleEvent::FallPrevented { item, by } => println!(
            "tick {tick:03}: {} item at ({}, {}) protected item at ({}, {})",
            by.side.name(),
            by.id.cell().x,
            by.id.cell().y,
            item.id.cell().x,
            item.id.cell().y
        ),
        BattleEvent::ItemConsumed { item, kind } => println!(
            "tick {tick:03}: {} hero consumed {} at ({}, {})",
            item.side.name(),
            kind.name(),
            item.id.cell().x,
            item.id.cell().y
        ),
        BattleEvent::Poisoned { target, stacks } => {
            println!(
                "tick {tick:03}: {} poisoned to {stacks} stacks",
                target.name()
            )
        }
        BattleEvent::PoisonDamage { target, amount } => {
            println!(
                "tick {tick:03}: {} takes {amount} poison damage",
                target.name()
            )
        }
        BattleEvent::PoisonCleansed { target, remaining } => {
            println!(
                "tick {tick:03}: {} cleanses poison to {remaining}",
                target.name()
            )
        }
    }
}
