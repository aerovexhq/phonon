//! Multi-threaded Inverse Device Design and Material Discovery Engine.
//!
//! Utilizes Rayon parallel iterators to distribute non-linear device physics evaluations,
//! quantum confinement shifts, band-to-band tunneling, and thermal self-heating calculations
//! across all available CPU cores.

use phonon_models::optimization::{
    AdjointRefiner, FitnessEvaluation, Individual, IrdsNodeTarget, Nsga2Config, Nsga2Optimizer,
    OptimizationTarget, RoadmapComplianceReport, TransistorGenome,
};
use rayon::prelude::*;

/// Configuration for the parallel inverse design engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub nsga2_config: Nsga2Config,
    pub enable_adjoint_refinement: bool,
    pub adjoint_steps: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            nsga2_config: Nsga2Config::default(),
            enable_adjoint_refinement: true,
            adjoint_steps: 8,
        }
    }
}

/// Multi-threaded inverse device design and material discovery engine.
#[derive(Debug, Clone)]
pub struct InverseDesignEngine {
    pub config: EngineConfig,
    optimizer: Nsga2Optimizer,
    refiner: AdjointRefiner,
}

impl InverseDesignEngine {
    pub fn new(config: EngineConfig) -> Self {
        let optimizer = Nsga2Optimizer::new(config.nsga2_config.clone());
        let refiner = AdjointRefiner::new(config.nsga2_config.bounds.clone(), config.adjoint_steps);
        Self {
            config,
            optimizer,
            refiner,
        }
    }

    /// Evaluates a batch of transistor genomes in parallel across all CPU cores.
    pub fn evaluate_batch_parallel(&self, genomes: Vec<TransistorGenome>) -> Vec<Individual> {
        genomes.into_par_iter().map(Individual::new).collect()
    }

    /// Advances the population by one generation using parallel offspring evaluation.
    pub fn evolve_generation_parallel(
        &self,
        current_pop: Vec<Individual>,
        seed: u64,
    ) -> Vec<Individual> {
        let n = current_pop.len();
        let mut rng = phonon_models::optimization::FastRng::new(seed);

        // Generate offspring genomes sequentially (fast breeding)
        let mut offspring_genomes = Vec::with_capacity(n);
        while offspring_genomes.len() < n {
            let i1 = rng.next_usize(n);
            let i2 = rng.next_usize(n);
            let i3 = rng.next_usize(n);
            let i4 = rng.next_usize(n);

            let p1 = if current_pop[i1].rank < current_pop[i2].rank
                || (current_pop[i1].rank == current_pop[i2].rank
                    && current_pop[i1].crowding_distance > current_pop[i2].crowding_distance)
            {
                &current_pop[i1]
            } else {
                &current_pop[i2]
            };

            let p2 = if current_pop[i3].rank < current_pop[i4].rank
                || (current_pop[i3].rank == current_pop[i4].rank
                    && current_pop[i3].crowding_distance > current_pop[i4].crowding_distance)
            {
                &current_pop[i3]
            } else {
                &current_pop[i4]
            };

            let (mut c1, mut c2) = self.optimizer.crossover(&p1.genome, &p2.genome, &mut rng);
            self.optimizer.mutate(&mut c1, &mut rng);
            self.optimizer.mutate(&mut c2, &mut rng);

            offspring_genomes.push(c1);
            if offspring_genomes.len() < n {
                offspring_genomes.push(c2);
            }
        }

        // Parallel evaluation of offspring on all CPU cores
        let offspring = self.evaluate_batch_parallel(offspring_genomes);

        // Combine parent + offspring (2N pool)
        let mut combined = current_pop;
        combined.extend(offspring);

        // Rank and crowd the 2N pool
        self.optimizer.rank_and_crowd(&mut combined);

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

        combined.truncate(n);
        combined
    }

    /// Executes multi-generational NSGA-II optimization across CPU cores.
    pub fn run_optimization(&self, generations: usize, seed: u64) -> Vec<Individual> {
        let mut population = self.optimizer.initialize_population(seed);

        for gen in 0..generations {
            let gen_seed = seed
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_mul(gen as u64 + 1);
            population = self.evolve_generation_parallel(population, gen_seed);
        }

        let mut pareto_front = Nsga2Optimizer::extract_pareto_front(&population);

        if self.config.enable_adjoint_refinement && !pareto_front.is_empty() {
            pareto_front = self
                .refine_pareto_front_parallel(&pareto_front, OptimizationTarget::MaximizeIonIoff);
        }

        pareto_front
    }

    /// Performs parallel adjoint sensitivity refinement on all individuals in the Pareto front.
    pub fn refine_pareto_front_parallel(
        &self,
        front: &[Individual],
        target: OptimizationTarget,
    ) -> Vec<Individual> {
        let refiner = &self.refiner;
        front
            .par_iter()
            .map(|ind| {
                let refined_genome = refiner.refine_candidate(&ind.genome, target);
                Individual::new(refined_genome)
            })
            .collect()
    }

    /// Filters and assesses compliance of Pareto solutions against an IRDS roadmap target.
    pub fn evaluate_roadmap_compliance(
        &self,
        solutions: &[Individual],
        target: &IrdsNodeTarget,
    ) -> Vec<(Individual, RoadmapComplianceReport)> {
        solutions
            .par_iter()
            .map(|ind| {
                let report = target.evaluate_compliance(&ind.fitness);
                (ind.clone(), report)
            })
            .collect()
    }

    /// Selects the best performing solution from the Pareto front for a specific metric.
    pub fn find_best_by<F>(&self, solutions: &[Individual], metric_fn: F) -> Option<Individual>
    where
        F: Fn(&FitnessEvaluation) -> f64 + Sync + Send,
    {
        solutions
            .par_iter()
            .max_by(|a, b| {
                let val_a = metric_fn(&a.fitness);
                let val_b = metric_fn(&b.fitness);
                val_a
                    .partial_cmp(&val_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_batch_evaluation() {
        let genomes = vec![
            TransistorGenome::n2_gaa_nanosheet_preset(),
            TransistorGenome::cfet_preset(),
            TransistorGenome::tmd_ultra_scaled_preset(),
        ];
        let engine = InverseDesignEngine::new(EngineConfig::default());
        let individuals = engine.evaluate_batch_parallel(genomes);

        assert_eq!(individuals.len(), 3);
        for ind in &individuals {
            assert!(ind.fitness.is_physically_viable);
            assert!(ind.fitness.i_on_a > 0.0);
        }
    }

    #[test]
    fn test_parallel_optimization_run() {
        let mut config = EngineConfig::default();
        config.nsga2_config.population_size = 24;
        config.nsga2_config.max_generations = 4;
        config.enable_adjoint_refinement = true;
        config.adjoint_steps = 4;

        let engine = InverseDesignEngine::new(config);
        let pareto_front = engine.run_optimization(4, 42);

        assert!(
            !pareto_front.is_empty(),
            "Must discover non-dominated designs"
        );
        for ind in &pareto_front {
            assert!(ind.fitness.is_physically_viable);
            assert!(ind.fitness.subthreshold_swing_mv_per_dec < 85.0);
        }
    }
}
