use crate::{Archetype, FallTelemetry, ItemKind, MAX_TICKS};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MatchupStat {
    pub battles: u64,
    pub left_wins: u64,
    pub right_wins: u64,
    pub draws: u64,
}

impl MatchupStat {
    pub fn left_score_rate(&self) -> f64 {
        rate(self.left_wins, self.right_wins, self.draws)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemStat {
    pub kind: ItemKind,
    pub bags: u64,
    pub wins: u64,
    pub losses: u64,
    pub draws: u64,
    pub swap_wins: u64,
    pub swap_losses: u64,
    pub swap_draws: u64,
}

impl ItemStat {
    pub fn score_rate(&self) -> f64 {
        rate(self.wins, self.losses, self.draws)
    }

    pub fn swap_score_rate(&self) -> f64 {
        rate(self.swap_wins, self.swap_losses, self.swap_draws)
    }

    pub fn swaps(&self) -> u64 {
        self.swap_wins + self.swap_losses + self.swap_draws
    }

    pub fn rank_score(&self) -> f64 {
        if self.swaps() > 0 {
            hoeffding_lower_bound(self.swap_wins, self.swap_losses, self.swap_draws)
        } else {
            hoeffding_lower_bound(self.wins, self.losses, self.draws)
        }
    }
}

fn rate(wins: u64, losses: u64, draws: u64) -> f64 {
    let games = wins + losses + draws;
    if games == 0 {
        0.0
    } else {
        (wins as f64 + draws as f64 / 2.0) / games as f64
    }
}

fn hoeffding_lower_bound(wins: u64, losses: u64, draws: u64) -> f64 {
    let n = (wins + losses + draws) as f64;
    if n == 0.0 {
        return 0.0;
    }
    (rate(wins, losses, draws) - (20.0_f64.ln() / (2.0 * n)).sqrt()).max(0.0)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BalanceReport {
    pub battles: u64,
    pub left_wins: u64,
    pub right_wins: u64,
    pub draws: u64,
    pub total_ticks: u64,
    pub unclassified_matchups: u64,
    pub matchups: [[MatchupStat; Archetype::COUNT]; Archetype::COUNT],
    pub mirrored_battles: u64,
    pub mirrored_left_wins: u64,
    pub mirrored_right_wins: u64,
    pub mirrored_draws: u64,
    pub fall_telemetry: FallTelemetry,
    pub duration_histogram: [u64; MAX_TICKS as usize + 1],
    pub lead_changes: u64,
    pub swing_battles: u64,
    pub decided_tick_total: u64,
    pub stats: Vec<ItemStat>,
}

impl BalanceReport {
    pub fn left_score_rate(&self) -> f64 {
        rate(self.left_wins, self.right_wins, self.draws)
    }

    pub fn side_bias_percentage_points(&self) -> f64 {
        if self.battles == 0 {
            0.0
        } else {
            (self.left_wins as f64 - self.right_wins as f64) / self.battles as f64 * 100.0
        }
    }

    pub fn mean_ticks(&self) -> f64 {
        if self.battles == 0 {
            0.0
        } else {
            self.total_ticks as f64 / self.battles as f64
        }
    }

    pub fn paired_side_bias_percentage_points(&self) -> f64 {
        let battles = self.battles + self.mirrored_battles;
        if battles == 0 {
            0.0
        } else {
            let left = self.left_wins + self.mirrored_left_wins;
            let right = self.right_wins + self.mirrored_right_wins;
            (left as f64 - right as f64) / battles as f64 * 100.0
        }
    }

    pub fn duration_p50(&self) -> u16 {
        percentile(&self.duration_histogram, self.battles.div_ceil(2))
    }

    pub fn duration_p90(&self) -> u16 {
        percentile(&self.duration_histogram, (self.battles * 9).div_ceil(10))
    }

    pub fn duration_p99(&self) -> u16 {
        percentile(&self.duration_histogram, (self.battles * 99).div_ceil(100))
    }

    /// Mean HP-lead flips per fight. 0 = one-sided stomps.
    pub fn mean_lead_changes(&self) -> f64 {
        if self.battles == 0 {
            0.0
        } else {
            self.lead_changes as f64 / self.battles as f64
        }
    }

    /// Fraction of fights whose HP lead changed at least once (had a swing).
    pub fn swing_rate(&self) -> f64 {
        if self.battles == 0 {
            0.0
        } else {
            self.swing_battles as f64 / self.battles as f64
        }
    }

    /// Mean fraction of the fight elapsed before the winner locked in the lead;
    /// higher means the outcome stayed live later into the fight.
    pub fn mean_decided_fraction(&self) -> f64 {
        if self.total_ticks == 0 {
            0.0
        } else {
            self.decided_tick_total as f64 / self.total_ticks as f64
        }
    }
}

fn percentile(histogram: &[u64], target: u64) -> u16 {
    if target == 0 {
        return 0;
    }
    let mut cumulative = 0;
    for (tick, count) in histogram.iter().enumerate() {
        cumulative += count;
        if cumulative >= target {
            return u16::try_from(tick).unwrap_or(MAX_TICKS);
        }
    }
    MAX_TICKS
}
