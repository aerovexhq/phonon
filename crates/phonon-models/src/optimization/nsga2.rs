//! Multi-Objective Non-Dominated Sorting Genetic Algorithm II (NSGA-II)
//! for transistor architecture, geometry, and material discovery.
//!
//! Formulates:
//! - Fast non-dominated sorting into Pareto dominance ranks ($\mathcal{F}_0, \mathcal{F}_1, \dots$).
//! - Multi-dimensional crowding distance calculation for diversity preservation.
//! - Crowded-comparison tournament selection.
//! - Simulated Binary Crossover (SBX) and Polynomial Mutation for continuous genes.
//! - Uniform crossover and categorical mutation for material and architecture choices.
//! - Self-contained deterministic PRNG (Xorshift64Star) ensuring reproducible optimization.

use super::genome::{
    ArchitectureType, ChannelMaterial, GeneBounds, OptContactMetal, OptGateDielectric,
    TransistorGenome,
};
use super::physical_fitness::{evaluate_transistor_fitness, FitnessEvaluation};

/// Fast deterministic 64-bit Xorshift PRNG for genetic search without external dependencies.
#[derive(Debug, Clone)]
pub struct FastRng {
    state: u64,
}

impl FastRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xdead_beef_cafe_babe
            } else {
                seed
            },
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 12;
        x ^= x >> 25;
        x ^= x << 27;
        self.state = x;
        x.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        // Uniform in [0, 1)
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    #[inline]
    pub fn next_range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }

    #[inline]
    pub fn next_usize(&mut self, max_exclusive: usize) -> usize {
        if max_exclusive <= 1 {
            0
        } else {
            (self.next_u64() % (max_exclusive as u64)) as usize
        }
    }
}

/// An individual in the evolutionary population.
#[derive(Debug, Clone, PartialEq)]
pub struct Individual {
    pub genome: TransistorGenome,
    pub fitness: FitnessEvaluation,
    pub rank: usize,
    pub crowding_distance: f64,
}

impl Individual {
    pub fn new(genome: TransistorGenome) -> Self {
        let fitness = evaluate_transistor_fitness(&genome);
        Self {
            genome,
            fitness,
            rank: 0,
            crowding_distance: 0.0,
        }
    }
}

/// Configuration parameters for the NSGA-II optimizer.
#[derive(Debug, Clone, PartialEq)]
pub struct Nsga2Config {
    /// Population size $N_{pop}$ (typically $50 - 200$).
    pub population_size: usize,
    /// Number of evolutionary generations (typically $20 - 100$).
    pub max_generations: usize,
    /// Probability of crossover between paired parents $P_c \in [0.7, 1.0]$.
    pub crossover_prob: f64,
    /// Probability of gene mutation $P_m \in [0.05, 0.30]$.
    pub mutation_prob: f64,
    /// Distribution index for Simulated Binary Crossover (SBX) $\eta_c$ (typically $15 - 20$).
    pub eta_c: f64,
    /// Distribution index for Polynomial Mutation $\eta_m$ (typically $20$).
    pub eta_m: f64,
    /// Physical search space bounds.
    pub bounds: GeneBounds,
}

impl Default for Nsga2Config {
    fn default() -> Self {
        Self {
            population_size: 60,
            max_generations: 25,
            crossover_prob: 0.90,
            mutation_prob: 0.20,
            eta_c: 15.0,
            eta_m: 20.0,
            bounds: GeneBounds::default(),
        }
    }
}

/// NSGA-II Multi-Objective Evolutionary Optimizer.
#[derive(Debug, Clone)]
pub struct Nsga2Optimizer {
    pub config: Nsga2Config,
}

impl Nsga2Optimizer {
    pub fn new(config: Nsga2Config) -> Self {
        Self { config }
    }

    /// Initializes a diverse random population covering the parameter space.
    pub fn initialize_population(&self, seed: u64) -> Vec<Individual> {
        let mut rng = FastRng::new(seed);
        let b = &self.config.bounds;
        let mut population = Vec::with_capacity(self.config.population_size);

        // Include canonical presets as seed anchors
        population.push(Individual::new(TransistorGenome::n2_gaa_nanosheet_preset()));
        population.push(Individual::new(TransistorGenome::cfet_preset()));
        population.push(Individual::new(TransistorGenome::tmd_ultra_scaled_preset()));

        while population.len() < self.config.population_size {
            let arch = match rng.next_usize(4) {
                0 => ArchitectureType::GaaNanosheet,
                1 => ArchitectureType::Cfet,
                2 => ArchitectureType::Ncfet,
                _ => ArchitectureType::FinFet,
            };

            let mat = match rng.next_usize(5) {
                0 => ChannelMaterial::Silicon,
                1 => ChannelMaterial::StrainedGermanium,
                2 => ChannelMaterial::InGaAs,
                3 => ChannelMaterial::MoS2,
                _ => ChannelMaterial::WS2,
            };

            let diel = match rng.next_usize(5) {
                0 => OptGateDielectric::HfO2,
                1 => OptGateDielectric::Al2O3,
                2 => OptGateDielectric::ZrO2,
                3 => OptGateDielectric::HzoFerroelectric,
                _ => OptGateDielectric::SiO2,
            };

            let metal = match rng.next_usize(5) {
                0 => OptContactMetal::Ruthenium,
                1 => OptContactMetal::NiSi,
                2 => OptContactMetal::PtSi,
                3 => OptContactMetal::TiN,
                _ => OptContactMetal::CoSi2,
            };

            let mut genome = TransistorGenome {
                architecture: arch,
                channel_material: mat,
                gate_dielectric: diel,
                contact_species: metal,
                gate_length_nm: rng
                    .next_range_f64(b.gate_length_nm_range.0, b.gate_length_nm_range.1),
                channel_thickness_nm: rng.next_range_f64(
                    b.channel_thickness_nm_range.0,
                    b.channel_thickness_nm_range.1,
                ),
                channel_width_nm: rng
                    .next_range_f64(b.channel_width_nm_range.0, b.channel_width_nm_range.1),
                eot_nm: rng.next_range_f64(b.eot_nm_range.0, b.eot_nm_range.1),
                sd_doping_cm3: rng.next_range_f64(b.sd_doping_range.0, b.sd_doping_range.1),
                body_doping_cm3: rng.next_range_f64(b.body_doping_range.0, b.body_doping_range.1),
                workfunction_ev: rng
                    .next_range_f64(b.workfunction_ev_range.0, b.workfunction_ev_range.1),
                num_sheets: rng.next_usize(3) + 1,
                v_dd_volts: rng.next_range_f64(0.5, 0.8),
            };

            genome.clamp_to_bounds(b);
            population.push(Individual::new(genome));
        }

        self.rank_and_crowd(&mut population);
        population
    }

    /// Evaluates Pareto dominance: returns true if `p` dominates `q` ($p \prec q$).
    #[inline]
    pub fn dominates(p: &Individual, q: &Individual) -> bool {
        let obj_p = p.fitness.objectives();
        let obj_q = q.fitness.objectives();

        let mut strictly_better = false;
        for i in 0..obj_p.len() {
            if obj_p[i] > obj_q[i] {
                return false; // p is worse than q in objective i
            }
            if obj_p[i] < obj_q[i] {
                strictly_better = true;
            }
        }
        strictly_better
    }

    /// Performs fast non-dominated sorting and crowding distance assignment on a population.
    pub fn rank_and_crowd(&self, population: &mut [Individual]) {
        let n = population.len();
        if n == 0 {
            return;
        }

        let mut domination_counts = vec![0usize; n];
        let mut dominated_sets: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut fronts: Vec<Vec<usize>> = Vec::new();
        let mut current_front = Vec::new();

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }
                if Self::dominates(&population[i], &population[j]) {
                    dominated_sets[i].push(j);
                } else if Self::dominates(&population[j], &population[i]) {
                    domination_counts[i] += 1;
                }
            }
            if domination_counts[i] == 0 {
                population[i].rank = 0;
                current_front.push(i);
            }
        }

        let mut rank = 0;
        while !current_front.is_empty() {
            fronts.push(current_front.clone());
            let mut next_front = Vec::new();
            for &i in &current_front {
                for &j in &dominated_sets[i] {
                    domination_counts[j] -= 1;
                    if domination_counts[j] == 0 {
                        population[j].rank = rank + 1;
                        next_front.push(j);
                    }
                }
            }
            rank += 1;
            current_front = next_front;
        }

        // Crowding distance calculation per front
        for front in &fronts {
            let m_front = front.len();
            if m_front == 0 {
                continue;
            }
            if m_front <= 2 {
                for &idx in front {
                    population[idx].crowding_distance = f64::INFINITY;
                }
                continue;
            }

            for &idx in front {
                population[idx].crowding_distance = 0.0;
            }

            let num_objs = population[front[0]].fitness.objectives().len();
            for m in 0..num_objs {
                let mut sorted_front = front.clone();
                sorted_front.sort_by(|&a, &b| {
                    let val_a = population[a].fitness.objectives()[m];
                    let val_b = population[b].fitness.objectives()[m];
                    val_a
                        .partial_cmp(&val_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let min_val = population[*sorted_front.first().unwrap()]
                    .fitness
                    .objectives()[m];
                let max_val = population[*sorted_front.last().unwrap()]
                    .fitness
                    .objectives()[m];
                let range = (max_val - min_val).max(1e-12);

                population[*sorted_front.first().unwrap()].crowding_distance = f64::INFINITY;
                population[*sorted_front.last().unwrap()].crowding_distance = f64::INFINITY;

                for k in 1..(m_front - 1) {
                    let prev = population[sorted_front[k - 1]].fitness.objectives()[m];
                    let next = population[sorted_front[k + 1]].fitness.objectives()[m];
                    let diff = (next - prev) / range;
                    population[sorted_front[k]].crowding_distance += diff;
                }
            }
        }
    }

    /// Binary tournament selection based on rank and crowding distance.
    #[inline]
    fn tournament_select<'a>(&self, a: &'a Individual, b: &'a Individual) -> &'a Individual {
        if a.rank < b.rank {
            a
        } else if b.rank < a.rank {
            b
        } else if a.crowding_distance > b.crowding_distance {
            a
        } else {
            b
        }
    }

    /// Simulated Binary Crossover (SBX) on two scalar variables.
    fn sbx_crossover_scalar(
        &self,
        p1: f64,
        p2: f64,
        rng: &mut FastRng,
        min: f64,
        max: f64,
    ) -> (f64, f64) {
        if (p1 - p2).abs() < 1e-12 {
            return (p1, p2);
        }

        let u = rng.next_f64();
        let beta = if u <= 0.5 {
            (2.0 * u).powf(1.0 / (self.config.eta_c + 1.0))
        } else {
            (1.0 / (2.0 * (1.0 - u))).powf(1.0 / (self.config.eta_c + 1.0))
        };

        let c1 = 0.5 * ((1.0 + beta) * p1 + (1.0 - beta) * p2);
        let c2 = 0.5 * ((1.0 - beta) * p1 + (1.0 + beta) * p2);

        (c1.clamp(min, max), c2.clamp(min, max))
    }

    /// Polynomial Mutation on a continuous variable.
    fn polynomial_mutate(&self, val: f64, rng: &mut FastRng, min: f64, max: f64) -> f64 {
        if rng.next_f64() > self.config.mutation_prob {
            return val;
        }

        let u = rng.next_f64();
        let delta = if u < 0.5 {
            (2.0 * u).powf(1.0 / (self.config.eta_m + 1.0)) - 1.0
        } else {
            1.0 - (2.0 * (1.0 - u)).powf(1.0 / (self.config.eta_m + 1.0))
        };

        (val + delta * (max - min)).clamp(min, max)
    }

    /// Crosses over two parent genomes to produce two offspring.
    pub fn crossover(
        &self,
        p1: &TransistorGenome,
        p2: &TransistorGenome,
        rng: &mut FastRng,
    ) -> (TransistorGenome, TransistorGenome) {
        let b = &self.config.bounds;
        let mut c1 = p1.clone();
        let mut c2 = p2.clone();

        if rng.next_f64() > self.config.crossover_prob {
            return (c1, c2);
        }

        // Discrete genes: uniform crossover
        if rng.next_f64() < 0.5 {
            std::mem::swap(&mut c1.architecture, &mut c2.architecture);
        }
        if rng.next_f64() < 0.5 {
            std::mem::swap(&mut c1.channel_material, &mut c2.channel_material);
        }
        if rng.next_f64() < 0.5 {
            std::mem::swap(&mut c1.gate_dielectric, &mut c2.gate_dielectric);
        }
        if rng.next_f64() < 0.5 {
            std::mem::swap(&mut c1.contact_species, &mut c2.contact_species);
        }
        if rng.next_f64() < 0.5 {
            std::mem::swap(&mut c1.num_sheets, &mut c2.num_sheets);
        }

        // Continuous genes: SBX crossover
        let (l1, l2) = self.sbx_crossover_scalar(
            p1.gate_length_nm,
            p2.gate_length_nm,
            rng,
            b.gate_length_nm_range.0,
            b.gate_length_nm_range.1,
        );
        c1.gate_length_nm = l1;
        c2.gate_length_nm = l2;

        let (t1, t2) = self.sbx_crossover_scalar(
            p1.channel_thickness_nm,
            p2.channel_thickness_nm,
            rng,
            b.channel_thickness_nm_range.0,
            b.channel_thickness_nm_range.1,
        );
        c1.channel_thickness_nm = t1;
        c2.channel_thickness_nm = t2;

        let (w1, w2) = self.sbx_crossover_scalar(
            p1.channel_width_nm,
            p2.channel_width_nm,
            rng,
            b.channel_width_nm_range.0,
            b.channel_width_nm_range.1,
        );
        c1.channel_width_nm = w1;
        c2.channel_width_nm = w2;

        let (e1, e2) = self.sbx_crossover_scalar(
            p1.eot_nm,
            p2.eot_nm,
            rng,
            b.eot_nm_range.0,
            b.eot_nm_range.1,
        );
        c1.eot_nm = e1;
        c2.eot_nm = e2;

        let (wf1, wf2) = self.sbx_crossover_scalar(
            p1.workfunction_ev,
            p2.workfunction_ev,
            rng,
            b.workfunction_ev_range.0,
            b.workfunction_ev_range.1,
        );
        c1.workfunction_ev = wf1;
        c2.workfunction_ev = wf2;

        c1.clamp_to_bounds(b);
        c2.clamp_to_bounds(b);

        (c1, c2)
    }

    /// Applies mutation to an offspring genome.
    pub fn mutate(&self, genome: &mut TransistorGenome, rng: &mut FastRng) {
        let b = &self.config.bounds;

        // Discrete mutation
        if rng.next_f64() < self.config.mutation_prob {
            genome.architecture = match rng.next_usize(4) {
                0 => ArchitectureType::GaaNanosheet,
                1 => ArchitectureType::Cfet,
                2 => ArchitectureType::Ncfet,
                _ => ArchitectureType::FinFet,
            };
        }
        if rng.next_f64() < self.config.mutation_prob {
            genome.channel_material = match rng.next_usize(5) {
                0 => ChannelMaterial::Silicon,
                1 => ChannelMaterial::StrainedGermanium,
                2 => ChannelMaterial::InGaAs,
                3 => ChannelMaterial::MoS2,
                _ => ChannelMaterial::WS2,
            };
        }
        if rng.next_f64() < self.config.mutation_prob {
            genome.gate_dielectric = match rng.next_usize(5) {
                0 => OptGateDielectric::HfO2,
                1 => OptGateDielectric::Al2O3,
                2 => OptGateDielectric::ZrO2,
                3 => OptGateDielectric::HzoFerroelectric,
                _ => OptGateDielectric::SiO2,
            };
        }
        if rng.next_f64() < self.config.mutation_prob {
            genome.contact_species = match rng.next_usize(5) {
                0 => OptContactMetal::Ruthenium,
                1 => OptContactMetal::NiSi,
                2 => OptContactMetal::PtSi,
                3 => OptContactMetal::TiN,
                _ => OptContactMetal::CoSi2,
            };
        }

        // Continuous polynomial mutation
        genome.gate_length_nm = self.polynomial_mutate(
            genome.gate_length_nm,
            rng,
            b.gate_length_nm_range.0,
            b.gate_length_nm_range.1,
        );
        genome.channel_thickness_nm = self.polynomial_mutate(
            genome.channel_thickness_nm,
            rng,
            b.channel_thickness_nm_range.0,
            b.channel_thickness_nm_range.1,
        );
        genome.channel_width_nm = self.polynomial_mutate(
            genome.channel_width_nm,
            rng,
            b.channel_width_nm_range.0,
            b.channel_width_nm_range.1,
        );
        genome.eot_nm =
            self.polynomial_mutate(genome.eot_nm, rng, b.eot_nm_range.0, b.eot_nm_range.1);
        genome.workfunction_ev = self.polynomial_mutate(
            genome.workfunction_ev,
            rng,
            b.workfunction_ev_range.0,
            b.workfunction_ev_range.1,
        );

        genome.clamp_to_bounds(b);
    }

    /// Advances the population by one generation using NSGA-II elitist selection.
    pub fn evolve_generation(&self, current_pop: Vec<Individual>, seed: u64) -> Vec<Individual> {
        let n = current_pop.len();
        let mut rng = FastRng::new(seed);

        // Generate offspring population of size N
        let mut offspring_genomes = Vec::with_capacity(n);
        while offspring_genomes.len() < n {
            let i1 = rng.next_usize(n);
            let i2 = rng.next_usize(n);
            let i3 = rng.next_usize(n);
            let i4 = rng.next_usize(n);

            let parent1 = self.tournament_select(&current_pop[i1], &current_pop[i2]);
            let parent2 = self.tournament_select(&current_pop[i3], &current_pop[i4]);

            let (mut c1, mut c2) = self.crossover(&parent1.genome, &parent2.genome, &mut rng);
            self.mutate(&mut c1, &mut rng);
            self.mutate(&mut c2, &mut rng);

            offspring_genomes.push(c1);
            if offspring_genomes.len() < n {
                offspring_genomes.push(c2);
            }
        }

        // Evaluate offspring
        let offspring: Vec<Individual> =
            offspring_genomes.into_iter().map(Individual::new).collect();

        // Combine parent + offspring (2N pool)
        let mut combined = current_pop;
        combined.extend(offspring);

        // Rank and crowd the 2N pool
        self.rank_and_crowd(&mut combined);

        // Sort combined pool by rank ascending, then crowding distance descending
        combined.sort_by(|a, b| {
            if a.rank != b.rank {
                a.rank.cmp(&b.rank)
            } else {
                b.crowding_distance
                    .partial_cmp(&a.crowding_distance)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        });

        // Select the best N individuals to form next generation
        combined.truncate(n);
        combined
    }

    /// Extracts the non-dominated Pareto front (Rank 0) from a population.
    pub fn extract_pareto_front(population: &[Individual]) -> Vec<Individual> {
        population
            .iter()
            .filter(|ind| ind.rank == 0 && ind.fitness.is_physically_viable)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_rng_distribution() {
        let mut rng = FastRng::new(42);
        for _ in 0..100 {
            let val = rng.next_f64();
            assert!((0.0..1.0).contains(&val));
            let idx = rng.next_usize(5);
            assert!(idx < 5);
        }
    }

    #[test]
    fn test_nsga2_initialization_and_ranking() {
        let config = Nsga2Config {
            population_size: 20,
            ..Default::default()
        };
        let opt = Nsga2Optimizer::new(config);
        let pop = opt.initialize_population(12345);

        assert_eq!(pop.len(), 20);
        let front = Nsga2Optimizer::extract_pareto_front(&pop);
        assert!(!front.is_empty(), "Must have at least one Rank 0 solution");
    }

    #[test]
    fn test_nsga2_evolution_convergence() {
        let config = Nsga2Config {
            population_size: 20,
            max_generations: 5,
            ..Default::default()
        };
        let opt = Nsga2Optimizer::new(config);
        let mut pop = opt.initialize_population(999);

        for gen in 0..5 {
            pop = opt.evolve_generation(pop, 1000 + gen as u64);
        }

        let front = Nsga2Optimizer::extract_pareto_front(&pop);
        assert!(!front.is_empty());
        let max_ratio = front
            .iter()
            .map(|ind| ind.fitness.ion_ioff_ratio)
            .fold(0.0, f64::max);
        assert!(
            max_ratio > 1000.0,
            "Front should discover high on/off ratio design: max={}",
            max_ratio
        );
        for ind in &front {
            assert!(ind.fitness.ion_ioff_ratio > 50.0);
            assert!(ind.fitness.subthreshold_swing_mv_per_dec < 90.0);
        }
    }
}
