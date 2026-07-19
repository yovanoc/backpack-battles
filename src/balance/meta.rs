use std::collections::HashSet;

use super::random_bag;
use crate::{Archetype, Bag, BattleConfig, Hero, ItemKind, Outcome, rng::Rng, simulate};

/// Configuration for a meta-health analysis: how many candidate bags to draft,
/// how large a shared opponent panel each is scored against, and how many of the
/// fittest bags form the "elite" whose item presence we report.
#[derive(Clone, Copy)]
pub struct MetaConfig {
    pub candidates: u64,
    pub panel: u64,
    pub seed: u64,
    pub tick_limit: u16,
    pub hero_health: u16,
    pub allow_rotation: bool,
    pub elite_size: usize,
}

pub struct ItemPresence {
    pub kind: ItemKind,
    /// Fraction of elite bags containing at least one copy of this item.
    pub presence: f64,
    pub elite_count: u64,
}

pub struct MetaReport {
    pub candidates: u64,
    pub panel: u64,
    pub elite_size: usize,
    pub mean_elite_fitness: f64,
    /// Distinct item-set signatures among the elite: how many genuinely
    /// different builds win. Higher = healthier, more diverse meta.
    pub distinct_signatures: usize,
    pub archetype_share: [f64; Archetype::COUNT],
    /// Every item, sorted by elite presence descending.
    pub presence: Vec<ItemPresence>,
}

/// Draft `candidates` random bags, score each by paired win rate against a shared
/// panel (both orientations, so side bias cancels), then report which items the
/// fittest bags actually run. This measures the mixed-bag meta - the lens the
/// designer's balancing notes rely on - rather than an artificial pure-archetype
/// matchup. Same seed always produces the same report.
pub fn run_meta(config: &MetaConfig) -> MetaReport {
    let mut rng = Rng::new(config.seed);
    let candidates: Vec<Bag> = (0..config.candidates)
        .map(|_| random_bag(&mut rng, config.allow_rotation))
        .collect();
    let panel_len = (config.panel as usize).min(candidates.len());

    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let mut fitness = vec![0.0_f64; candidates.len()];
    std::thread::scope(|scope| {
        let candidates = &candidates;
        let handles: Vec<_> = (0..threads)
            .map(|offset| {
                scope.spawn(move || {
                    let mut local = Vec::new();
                    let mut index = offset;
                    while index < candidates.len() {
                        local.push((
                            index,
                            candidate_fitness(config, candidates, index, panel_len),
                        ));
                        index += threads;
                    }
                    local
                })
            })
            .collect();
        for handle in handles {
            for (index, score) in handle.join().expect("meta fitness thread panicked") {
                fitness[index] = score;
            }
        }
    });

    let mut order: Vec<usize> = (0..candidates.len()).collect();
    order.sort_by(|&a, &b| fitness[b].total_cmp(&fitness[a]).then(a.cmp(&b)));
    let elite_size = config.elite_size.min(order.len());
    let elite = &order[..elite_size];

    let mut counts = [0_u64; ItemKind::COUNT];
    let mut arch_cells = [0_u64; Archetype::COUNT];
    let mut total_cells = 0_u64;
    let mut signatures: HashSet<Vec<u8>> = HashSet::new();
    for &index in elite {
        let bag = &candidates[index];
        let mut seen = [false; ItemKind::COUNT];
        let mut signature: Vec<u8> = Vec::new();
        for item in bag.items() {
            let kind = item.kind();
            seen[kind as usize] = true;
            signature.push(kind as u8);
            let cells = item.shape().len() as u64;
            arch_cells[kind.archetype() as usize] += cells;
            total_cells += cells;
        }
        for (kind_index, present) in seen.iter().enumerate() {
            if *present {
                counts[kind_index] += 1;
            }
        }
        signature.sort_unstable();
        signatures.insert(signature);
    }

    let divisor = elite_size.max(1) as f64;
    let mut presence: Vec<ItemPresence> = ItemKind::ALL
        .iter()
        .map(|kind| ItemPresence {
            kind: *kind,
            presence: counts[*kind as usize] as f64 / divisor,
            elite_count: counts[*kind as usize],
        })
        .collect();
    presence.sort_by(|a, b| {
        b.presence
            .total_cmp(&a.presence)
            .then((a.kind as usize).cmp(&(b.kind as usize)))
    });

    let mut archetype_share = [0.0_f64; Archetype::COUNT];
    if total_cells > 0 {
        for (share, cells) in archetype_share.iter_mut().zip(arch_cells) {
            *share = cells as f64 / total_cells as f64;
        }
    }

    let mean_elite_fitness = if elite.is_empty() {
        0.0
    } else {
        elite.iter().map(|&index| fitness[index]).sum::<f64>() / elite.len() as f64
    };

    MetaReport {
        candidates: config.candidates,
        panel: panel_len as u64,
        elite_size,
        mean_elite_fitness,
        distinct_signatures: signatures.len(),
        archetype_share,
        presence,
    }
}

fn candidate_fitness(
    config: &MetaConfig,
    candidates: &[Bag],
    index: usize,
    panel_len: usize,
) -> f64 {
    let candidate = &candidates[index];
    let mut score = 0.0_f64;
    let mut battles = 0.0_f64;
    for (opponent_index, opponent) in candidates.iter().take(panel_len).enumerate() {
        if opponent_index == index {
            continue;
        }
        // Candidate on the left, then on the right: side bias cancels.
        score += match duel(
            config,
            candidate,
            opponent,
            seed_for(config.seed, index, opponent_index, 0),
        ) {
            Outcome::LeftWins => 1.0,
            Outcome::Draw => 0.5,
            Outcome::RightWins => 0.0,
        };
        score += match duel(
            config,
            opponent,
            candidate,
            seed_for(config.seed, index, opponent_index, 1),
        ) {
            Outcome::RightWins => 1.0,
            Outcome::Draw => 0.5,
            Outcome::LeftWins => 0.0,
        };
        battles += 2.0;
    }
    if battles == 0.0 { 0.0 } else { score / battles }
}

fn duel(config: &MetaConfig, left: &Bag, right: &Bag, seed: u64) -> Outcome {
    let left = Hero::new("left", config.hero_health, left.clone());
    let right = Hero::new("right", config.hero_health, right.clone());
    let battle_config =
        BattleConfig::new(config.tick_limit, seed).expect("tick limit validated by caller");
    simulate(left, right, battle_config).outcome
}

fn seed_for(base: u64, candidate: usize, opponent: usize, orientation: u64) -> u64 {
    let mut mixer = Rng::new(
        base.wrapping_add((candidate as u64).wrapping_mul(1_000_003))
            .wrapping_add((opponent as u64).wrapping_mul(97))
            .wrapping_add(orientation),
    );
    mixer.next_u64()
}
