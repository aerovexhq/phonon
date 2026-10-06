#![deny(unsafe_code)]

//! NSGA-II Multi-Objective Genetic Algorithm & Pareto Frontier Engine.
//!
//! Implements:
//! - Multi-objective minimization: Power (W), Latency/Cycle Delay (ns), Silicon Area (mm^2), Cost (USD)
//! - Fast non-dominated sorting algorithm into Pareto fronts F_1, F_2, ...
//! - Crowding distance metric for diversity preservation along high-dimensional frontiers
//! - Simulated Binary Crossover (SBX) and Polynomial Mutation for continuous architectural genomes

use super::packaging_trades::{evaluate_packaging_ppac, PackagingTechnology};

/// Multi-parameter architectural genome representing a hardware candidate in design space.
#[derive(Debug, Clone)]
pub struct DesignGenome {
    pub vdd_v: f64,
    pub clock_freq_ghz: f64,
    pub gate_length_nm: f64,
    pub nanosheet_width_nm: f64,
    pub num_nanosheets: usize,
    pub num_cores: usize,
    pub cache_l3_mb: f64,
    pub packaging: PackagingTechnology,
}

impl Default for DesignGenome {
    fn default() -> Self {
        Self {
            vdd_v: 0.85,
            clock_freq_ghz: 3.20,
            gate_length_nm: 16.0,
            nanosheet_width_nm: 30.0,
            num_nanosheets: 3,
            num_cores: 16,
            cache_l3_mb: 32.0,
            packaging: PackagingTechnology::SiliconInterposerCoWoS,
        }
    }
}

/// Evaluated multi-objective targets for Pareto dominance testing (all normalized for minimization).
#[derive(Debug, Clone)]
pub struct ObjectiveValues {
    pub power_w: f64,
    pub latency_ns: f64,
    pub silicon_area_mm2: f64,
    pub unit_cost_usd: f64,
}

impl ObjectiveValues {
    /// Returns true if self strictly Pareto-dominates other.
    pub fn dominates(&self, other: &ObjectiveValues) -> bool {
        let no_worse = self.power_w <= other.power_w
            && self.latency_ns <= other.latency_ns
            && self.silicon_area_mm2 <= other.silicon_area_mm2
            && self.unit_cost_usd <= other.unit_cost_usd;

        let strictly_better = self.power_w < other.power_w
            || self.latency_ns < other.latency_ns
            || self.silicon_area_mm2 < other.silicon_area_mm2
            || self.unit_cost_usd < other.unit_cost_usd;

        no_worse && strictly_better
    }
}

/// Individual in the NSGA-II population with genome, evaluated objectives, and ranking metadata.
#[derive(Debug, Clone)]
pub struct Individual {
    pub genome: DesignGenome,
    pub objectives: ObjectiveValues,
    pub rank: usize,
    pub crowding_distance: f64,
}

/// Evaluates objectives for a given design genome.
pub fn evaluate_genome(genome: &DesignGenome, wafer_cost_usd: f64) -> ObjectiveValues {
    let ppac = evaluate_packaging_ppac(
        genome.packaging,
        genome.num_cores,
        genome.cache_l3_mb,
        genome.clock_freq_ghz,
        genome.vdd_v,
        wafer_cost_usd,
    );

    let latency_ns = 1.0 / genome.clock_freq_ghz.max(0.1);

    ObjectiveValues {
        power_w: ppac.total_power_w,
        latency_ns,
        silicon_area_mm2: ppac.total_silicon_area_mm2,
        unit_cost_usd: ppac.unit_manufacturing_cost_usd,
    }
}

/// Performs fast non-dominated sorting partitioning population into Pareto fronts.
pub fn non_dominated_sort(population: &mut [Individual]) -> Vec<Vec<usize>> {
    let n = population.len();
    let mut domination_counts = vec![0usize; n];
    let mut dominated_sets = vec![Vec::new(); n];
    let mut fronts = Vec::new();
    let mut first_front = Vec::new();

    for p in 0..n {
        for q in 0..n {
            if p == q {
                continue;
            }
            if population[p].objectives.dominates(&population[q].objectives) {
                dominated_sets[p].push(q);
            } else if population[q].objectives.dominates(&population[p].objectives) {
                domination_counts[p] += 1;
            }
        }
        if domination_counts[p] == 0 {
            population[p].rank = 1;
            first_front.push(p);
        }
    }

    fronts.push(first_front);

    let mut current_front_idx = 0;
    while current_front_idx < fronts.len() {
        let mut next_front = Vec::new();
        for &p in &fronts[current_front_idx] {
            for &q in &dominated_sets[p] {
                domination_counts[q] -= 1;
                if domination_counts[q] == 0 {
                    population[q].rank = current_front_idx + 2;
                    next_front.push(q);
                }
            }
        }
        if !next_front.is_empty() {
            fronts.push(next_front);
            current_front_idx += 1;
        } else {
            break;
        }
    }

    fronts
}

/// Computes crowding distances for individuals within a Pareto front.
pub fn assign_crowding_distance(front: &[usize], population: &mut [Individual]) {
    let l = front.len();
    if l == 0 {
        return;
    }
    if l <= 2 {
        for &idx in front {
            population[idx].crowding_distance = f64::INFINITY;
        }
        return;
    }

    for &idx in front {
        population[idx].crowding_distance = 0.0;
    }

    // 4 objectives to calculate cuboid distances
    let num_objectives = 4;
    for m in 0..num_objectives {
        let mut sorted_front = front.to_vec();
        sorted_front.sort_by(|&a, &b| {
            let val_a = get_objective(&population[a].objectives, m);
            let val_b = get_objective(&population[b].objectives, m);
            val_a.partial_cmp(&val_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let min_idx = sorted_front[0];
        let max_idx = sorted_front[l - 1];
        population[min_idx].crowding_distance = f64::INFINITY;
        population[max_idx].crowding_distance = f64::INFINITY;

        let obj_min = get_objective(&population[min_idx].objectives, m);
        let obj_max = get_objective(&population[max_idx].objectives, m);
        let range = (obj_max - obj_min).max(1e-9);

        for i in 1..(l - 1) {
            let prev = sorted_front[i - 1];
            let next = sorted_front[i + 1];
            let curr = sorted_front[i];

            let dist_contribution = (get_objective(&population[next].objectives, m)
                - get_objective(&population[prev].objectives, m))
                / range;

            if population[curr].crowding_distance.is_finite() {
                population[curr].crowding_distance += dist_contribution;
            }
        }
    }
}

fn get_objective(obj: &ObjectiveValues, index: usize) -> f64 {
    match index {
        0 => obj.power_w,
        1 => obj.latency_ns,
        2 => obj.silicon_area_mm2,
        _ => obj.unit_cost_usd,
    }
}

/// Deterministic pseudo-random number generator for genetic operations.
pub struct GeneticRng {
    state: u64,
}

impl GeneticRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9e3779b97f4a7c15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn in_range(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }

    pub fn choose_usize(&mut self, options: &[usize]) -> usize {
        let idx = (self.next_u64() as usize) % options.len();
        options[idx]
    }
}

/// Simulated Binary Crossover (SBX) with eta = 20.0.
pub fn sbx_crossover(
    parent1: &DesignGenome,
    parent2: &DesignGenome,
    rng: &mut GeneticRng,
) -> (DesignGenome, DesignGenome) {
    let eta = 20.0;
    let mut c1 = parent1.clone();
    let mut c2 = parent2.clone();

    // Continuous parameter crossover
    let crossover_val = |p1: f64, p2: f64, min: f64, max: f64, rng: &mut GeneticRng| -> (f64, f64) {
        let u = rng.next_f64();
        let beta = if u <= 0.5 {
            (2.0 * u).powf(1.0 / (eta + 1.0))
        } else {
            (1.0 / (2.0 * (1.0 - u))).powf(1.0 / (eta + 1.0))
        };
        let o1 = 0.5 * ((1.0 + beta) * p1 + (1.0 - beta) * p2);
        let o2 = 0.5 * ((1.0 - beta) * p1 + (1.0 + beta) * p2);
        (o1.clamp(min, max), o2.clamp(min, max))
    };

    let (v1, v2) = crossover_val(parent1.vdd_v, parent2.vdd_v, 0.65, 1.10, rng);
    c1.vdd_v = v1;
    c2.vdd_v = v2;

    let (f1, f2) = crossover_val(parent1.clock_freq_ghz, parent2.clock_freq_ghz, 1.2, 4.5, rng);
    c1.clock_freq_ghz = f1;
    c2.clock_freq_ghz = f2;

    let (l1, l2) = crossover_val(parent1.gate_length_nm, parent2.gate_length_nm, 12.0, 22.0, rng);
    c1.gate_length_nm = l1;
    c2.gate_length_nm = l2;

    let (w1, w2) = crossover_val(parent1.nanosheet_width_nm, parent2.nanosheet_width_nm, 15.0, 50.0, rng);
    c1.nanosheet_width_nm = w1;
    c2.nanosheet_width_nm = w2;

    // Discrete parameters: uniform swap
    if rng.next_f64() < 0.5 {
        std::mem::swap(&mut c1.num_nanosheets, &mut c2.num_nanosheets);
    }
    if rng.next_f64() < 0.5 {
        std::mem::swap(&mut c1.num_cores, &mut c2.num_cores);
    }
    if rng.next_f64() < 0.5 {
        std::mem::swap(&mut c1.packaging, &mut c2.packaging);
    }

    (c1, c2)
}

/// Polynomial mutation with eta = 20.0.
pub fn polynomial_mutation(genome: &mut DesignGenome, rng: &mut GeneticRng) {
    let eta = 20.0;
    let pm = 0.25;

    let mutate_val = |val: f64, min: f64, max: f64, rng: &mut GeneticRng| -> f64 {
        if rng.next_f64() < pm {
            let u = rng.next_f64();
            let delta = if u < 0.5 {
                (2.0 * u).powf(1.0 / (eta + 1.0)) - 1.0
            } else {
                1.0 - (2.0 * (1.0 - u)).powf(1.0 / (eta + 1.0))
            };
            (val + delta * (max - min)).clamp(min, max)
        } else {
            val
        }
    };

    genome.vdd_v = mutate_val(genome.vdd_v, 0.65, 1.10, rng);
    genome.clock_freq_ghz = mutate_val(genome.clock_freq_ghz, 1.2, 4.5, rng);
    genome.gate_length_nm = mutate_val(genome.gate_length_nm, 12.0, 22.0, rng);
    genome.nanosheet_width_nm = mutate_val(genome.nanosheet_width_nm, 15.0, 50.0, rng);

    if rng.next_f64() < 0.20 {
        let cores = [4, 8, 16, 32];
        genome.num_cores = rng.choose_usize(&cores);
    }

    if rng.next_f64() < 0.20 {
        let tech_options = [
            PackagingTechnology::MonolithicSoc,
            PackagingTechnology::OrganicMcm,
            PackagingTechnology::SiliconInterposerCoWoS,
            PackagingTechnology::HybridBonding3D,
        ];
        let idx = (rng.next_u64() as usize) % tech_options.len();
        genome.packaging = tech_options[idx];
    }
}
