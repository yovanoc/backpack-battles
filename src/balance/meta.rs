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
    let (candidates, elite, fitness, panel_len) = draft_and_rank(config);
    let elite_size = elite.len();

    let mut counts = [0_u64; ItemKind::COUNT];
    let mut arch_cells = [0_u64; Archetype::COUNT];
    let mut total_cells = 0_u64;
    let mut signatures: HashSet<Vec<u8>> = HashSet::new();
    for &index in &elite {
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

// ---------------------------------------------------------------------------
// Verdict stage: objective pass/fail balance gates on the elite counter-graph.
// Thresholds are exported, documented constants - no buried magic numbers.
// ---------------------------------------------------------------------------

/// Each bag must have at least one counter that beats it this often, or it is a
/// "well": an un-counterable, degenerate strategy.
pub const NO_WELL_MIN_COUNTER: f64 = 0.60;
/// A bag's best counter must not exceed this, or the counter is an "execution"
/// - a hard lock with no intermediate defense to play around.
pub const CONTESTED_MAX_COUNTER: f64 = 0.85;
/// A bag's best counter must differ from it by at least this composition
/// distance, or the "counter" is just the same bag +/- one item (a stat-check).
pub const SUBSTANTIAL_MIN_DISTANCE: f64 = 0.40;
/// A directed edge exists when one bag beats another at least this often.
pub const CYCLE_EDGE_WINRATE: f64 = 0.55;
/// The beat-graph must contain a strongly connected component at least this
/// large, or it is an acyclic disguised ranking with a single best bag.
pub const CYCLE_MIN_SCC: usize = 3;
/// No item may appear in more than this fraction of elite bags (oppressive);
/// an item in zero elite bags is dead. Healthy roster: 0 < presence <= this.
pub const ROSTER_MAX_PRESENCE: f64 = 0.60;

/// How many independent seeds each elite pair is played over (both orientations
/// each) to smooth the beat rate before thresholding.
const MATCHUP_SEEDS: u64 = 6;

pub struct VerdictReport {
    pub elite_size: usize,
    pub bags_without_counter: usize,
    pub min_best_counter: f64,
    pub max_best_counter: f64,
    pub executions: usize,
    pub min_counter_distance: f64,
    pub median_counter_distance: f64,
    pub stat_check_counters: usize,
    pub largest_scc: usize,
    pub scc_ge3_count: usize,
    pub dead_items: Vec<ItemKind>,
    pub oppressive_items: Vec<(ItemKind, f64)>,
    pub pass_no_wells: bool,
    pub pass_contested: bool,
    pub pass_substantial: bool,
    pub pass_cycles: bool,
    pub pass_roster: bool,
}

impl VerdictReport {
    pub fn passed(&self) -> bool {
        self.pass_no_wells
            && self.pass_contested
            && self.pass_substantial
            && self.pass_cycles
            && self.pass_roster
    }
}

/// Run s17n's balance stage: draft an elite, build its paired beat-matrix, and
/// grade the counter-graph against the exported thresholds. Same seed always
/// produces the same verdict.
pub fn run_verdict(config: &MetaConfig) -> VerdictReport {
    // Pool the evolved elite of several independent coevolution runs, so an
    // item counts as dead only if it is absent from every run's elite and
    // oppressive only if it dominates across them (the design doc's merged-run
    // methodology, robust to a single run's incidental gaps).
    const VERDICT_RUNS: u64 = 3;
    let mut pooled: Vec<Bag> = Vec::new();
    for run in 0..VERDICT_RUNS {
        let mut run_config = *config;
        run_config.seed = config.seed.wrapping_add(run);
        let (candidates, elite, _fitness, _panel) = draft_and_rank(&run_config);
        for &index in &elite {
            pooled.push(candidates[index].clone());
        }
    }
    let elite_bags: Vec<&Bag> = pooled.iter().collect();
    let n = elite_bags.len();
    let beat = beat_matrix(config, &elite_bags);

    // best_counter[i] = how hard the strongest other bag beats bag i.
    // best_prey[i]    = how hard bag i beats its own best target.
    // A bag hard-countered (>85%) is fine if it also hard-counters something -
    // that is a legit rock-paper-scissors piece. Only a bag hard-countered with
    // NO hard prey of its own is an "execution": beaten with no counterplay
    // (the design doc's contre-execution sans defense intermediaire).
    let mut best_counter = vec![0.0_f64; n];
    let mut best_prey = vec![0.0_f64; n];
    let mut counter_of = vec![0_usize; n];
    for i in 0..n {
        let mut best = 0.0_f64;
        let mut best_j = i;
        for (j, beat_row) in beat.iter().enumerate() {
            if i != j && beat_row[i] > best {
                best = beat_row[i];
                best_j = j;
            }
        }
        best_counter[i] = best;
        counter_of[i] = best_j;
        best_prey[i] = beat[i]
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, value)| *value)
            .fold(0.0_f64, f64::max);
    }

    let bags_without_counter = best_counter
        .iter()
        .filter(|&&c| c < NO_WELL_MIN_COUNTER)
        .count();
    let min_best_counter = best_counter.iter().copied().fold(1.0_f64, f64::min);
    let max_best_counter = best_counter.iter().copied().fold(0.0_f64, f64::max);
    let executions = (0..n)
        .filter(|&i| {
            best_counter[i] > CONTESTED_MAX_COUNTER && best_prey[i] <= CONTESTED_MAX_COUNTER
        })
        .count();

    let mut distances: Vec<f64> = (0..n)
        .map(|i| composition_distance(elite_bags[i], elite_bags[counter_of[i]]))
        .collect();
    let stat_check_counters = distances
        .iter()
        .filter(|&&d| d < SUBSTANTIAL_MIN_DISTANCE)
        .count();
    let min_counter_distance = distances.iter().copied().fold(1.0_f64, f64::min);
    distances.sort_by(f64::total_cmp);
    let median_counter_distance = if distances.is_empty() {
        0.0
    } else {
        distances[distances.len() / 2]
    };

    let adjacency: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            (0..n)
                .filter(|&j| i != j && beat[i][j] >= CYCLE_EDGE_WINRATE)
                .collect()
        })
        .collect();
    let scc = scc_sizes(n, &adjacency);
    let largest_scc = scc.iter().copied().max().unwrap_or(0);
    let scc_ge3_count = scc.iter().filter(|&&s| s >= CYCLE_MIN_SCC).count();

    let mut counts = [0_u64; ItemKind::COUNT];
    for bag in &elite_bags {
        let mut seen = [false; ItemKind::COUNT];
        for item in bag.items() {
            seen[item.kind() as usize] = true;
        }
        for (kind_index, present) in seen.iter().enumerate() {
            if *present {
                counts[kind_index] += 1;
            }
        }
    }
    let divisor = n.max(1) as f64;
    let dead_items: Vec<ItemKind> = ItemKind::ALL
        .iter()
        .filter(|kind| counts[**kind as usize] == 0)
        .copied()
        .collect();
    let oppressive_items: Vec<(ItemKind, f64)> = ItemKind::ALL
        .iter()
        .filter_map(|kind| {
            let presence = counts[*kind as usize] as f64 / divisor;
            (presence > ROSTER_MAX_PRESENCE).then_some((*kind, presence))
        })
        .collect();

    VerdictReport {
        elite_size: n,
        bags_without_counter,
        min_best_counter,
        max_best_counter,
        executions,
        min_counter_distance,
        median_counter_distance,
        stat_check_counters,
        largest_scc,
        scc_ge3_count,
        pass_no_wells: bags_without_counter == 0,
        pass_contested: executions == 0,
        pass_substantial: stat_check_counters == 0,
        pass_cycles: scc_ge3_count >= 1,
        pass_roster: dead_items.is_empty() && oppressive_items.is_empty(),
        dead_items,
        oppressive_items,
    }
}

fn draft_and_rank(config: &MetaConfig) -> (Vec<Bag>, Vec<usize>, Vec<f64>, usize) {
    const GENERATIONS: usize = 12;
    let mut rng = Rng::new(config.seed);
    let pop_size = (config.candidates as usize).max(4);
    let mut population: Vec<Bag> = (0..pop_size)
        .map(|_| random_bag(&mut rng, config.allow_rotation))
        .collect();

    // Coevolution: each generation, score every bag against the whole
    // population (paired), keep the fittest half, refill by mutating survivors.
    // Bags adapt to each other, so the survivors are an optimised,
    // drafted-quality meta - the right sample to judge balance on, instead of
    // fragile random bags.
    let mut fitness = population_fitness(config, &population);
    for _ in 0..GENERATIONS {
        let mut order: Vec<usize> = (0..population.len()).collect();
        order.sort_by(|&a, &b| fitness[b].total_cmp(&fitness[a]).then(a.cmp(&b)));
        let keep = (population.len() / 2).max(1);
        let survivors: Vec<Bag> = order[..keep]
            .iter()
            .map(|&i| population[i].clone())
            .collect();
        let mut next = survivors.clone();
        while next.len() < population.len() {
            let parent =
                &survivors[usize::try_from(rng.below(survivors.len() as u64)).unwrap_or(0)];
            next.push(super::generator::mutate(
                parent,
                &mut rng,
                config.allow_rotation,
            ));
        }
        population = next;
        fitness = population_fitness(config, &population);
    }

    let mut order: Vec<usize> = (0..population.len()).collect();
    order.sort_by(|&a, &b| fitness[b].total_cmp(&fitness[a]).then(a.cmp(&b)));
    order.truncate(config.elite_size.min(order.len()));
    (population, order, fitness, pop_size)
}

/// Mean paired win rate of each bag against the whole population (both sides).
fn population_fitness(config: &MetaConfig, population: &[Bag]) -> Vec<f64> {
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let mut fitness = vec![0.0_f64; population.len()];
    std::thread::scope(|scope| {
        let population = &population;
        let handles: Vec<_> = (0..threads)
            .map(|offset| {
                scope.spawn(move || {
                    let mut local = Vec::new();
                    let mut index = offset;
                    while index < population.len() {
                        local.push((
                            index,
                            candidate_fitness(config, population, index, population.len()),
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
    fitness
}

/// Paired win rate of every elite bag against every other: beat[i][j] is the
/// fraction of games bag i wins against bag j across MATCHUP_SEEDS seeds and
/// both orientations (side bias cancels).
fn beat_matrix(config: &MetaConfig, bags: &[&Bag]) -> Vec<Vec<f64>> {
    let n = bags.len();
    let threads = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let mut beat = vec![vec![0.0_f64; n]; n];
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|offset| {
                scope.spawn(move || {
                    let mut rows = Vec::new();
                    let mut i = offset;
                    while i < n {
                        let mut row = vec![0.0_f64; n];
                        for (j, cell) in row.iter_mut().enumerate() {
                            if i == j {
                                continue;
                            }
                            let mut wins = 0.0_f64;
                            for k in 0..MATCHUP_SEEDS {
                                let base = config.seed.wrapping_add(k.wrapping_mul(2_654_435_761));
                                match duel(config, bags[i], bags[j], seed_for(base, i, j, 0)) {
                                    Outcome::LeftWins => wins += 1.0,
                                    Outcome::Draw => wins += 0.5,
                                    Outcome::RightWins => {}
                                }
                                match duel(config, bags[j], bags[i], seed_for(base, i, j, 1)) {
                                    Outcome::RightWins => wins += 1.0,
                                    Outcome::Draw => wins += 0.5,
                                    Outcome::LeftWins => {}
                                }
                            }
                            *cell = wins / (MATCHUP_SEEDS as f64 * 2.0);
                        }
                        rows.push((i, row));
                        i += threads;
                    }
                    rows
                })
            })
            .collect();
        for handle in handles {
            for (i, row) in handle.join().expect("beat-matrix thread panicked") {
                beat[i] = row;
            }
        }
    });
    beat
}

/// Multiset Jaccard distance between two bags' item compositions: 0.0 identical,
/// 1.0 fully disjoint. "Same bag +/- one item" scores near 0.
fn composition_distance(a: &Bag, b: &Bag) -> f64 {
    let mut counts_a = [0_u32; ItemKind::COUNT];
    let mut counts_b = [0_u32; ItemKind::COUNT];
    for item in a.items() {
        counts_a[item.kind() as usize] += 1;
    }
    for item in b.items() {
        counts_b[item.kind() as usize] += 1;
    }
    let mut intersection = 0_u32;
    let mut union = 0_u32;
    for (ca, cb) in counts_a.iter().zip(counts_b.iter()) {
        intersection += (*ca).min(*cb);
        union += (*ca).max(*cb);
    }
    if union == 0 {
        0.0
    } else {
        1.0 - f64::from(intersection) / f64::from(union)
    }
}

/// Sizes of the strongly connected components of a small directed graph, via a
/// transitive-closure mutual-reachability grouping (n is tiny, so O(n^3) is
/// fine and far simpler than Tarjan).
#[allow(clippy::needless_range_loop)]
fn scc_sizes(n: usize, adjacency: &[Vec<usize>]) -> Vec<usize> {
    let mut reach = vec![vec![false; n]; n];
    for (i, neighbours) in adjacency.iter().enumerate() {
        for &j in neighbours {
            reach[i][j] = true;
        }
    }
    for k in 0..n {
        for i in 0..n {
            if reach[i][k] {
                for j in 0..n {
                    if reach[k][j] {
                        reach[i][j] = true;
                    }
                }
            }
        }
    }
    let mut component = vec![usize::MAX; n];
    let mut sizes = Vec::new();
    for i in 0..n {
        if component[i] != usize::MAX {
            continue;
        }
        let id = sizes.len();
        component[i] = id;
        let mut size = 1;
        for j in (i + 1)..n {
            if component[j] == usize::MAX && reach[i][j] && reach[j][i] {
                component[j] = id;
                size += 1;
            }
        }
        sizes.push(size);
    }
    sizes
}
