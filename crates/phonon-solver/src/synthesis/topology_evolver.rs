//! Multi-Core Parallel Graph-Evolutionary Circuit Synthesizer.
//!
//! Uses Rayon parallel iterators to evolve unconstrained circuit topologies,
//! discovering optimal non-standard logic gates, pass-transistor networks, and arithmetic units.

use phonon_models::optimization::FastRng;
use phonon_models::synthesis::{
    CircuitMetricsEvaluator, CircuitTopology, GateElement, GateMetrics, GateNode, TruthTable,
};
use rayon::prelude::*;

/// Configuration parameters for unconstrained gate topology synthesis.
#[derive(Debug, Clone)]
pub struct SynthesisConfig {
    /// Population size (typically 30 - 80).
    pub population_size: usize,
    /// Number of evolutionary generations (typically 5 - 30).
    pub max_generations: usize,
    /// Probability of mutating a circuit topology in each generation.
    pub mutation_prob: f64,
    /// Probability of crossover between two circuit topologies.
    pub crossover_prob: f64,
    /// Target Boolean truth table to synthesize.
    pub target_truth_table: TruthTable,
    /// Metrics evaluator settings.
    pub evaluator: CircuitMetricsEvaluator,
}

impl Default for SynthesisConfig {
    fn default() -> Self {
        Self {
            population_size: 40,
            max_generations: 10,
            mutation_prob: 0.35,
            crossover_prob: 0.70,
            target_truth_table: TruthTable::nand2(),
            evaluator: CircuitMetricsEvaluator::default(),
        }
    }
}

/// An individual candidate topology with evaluated fitness and metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct SynthesizedCandidate {
    pub topology: CircuitTopology,
    pub metrics: GateMetrics,
    pub fitness: f64,
}

impl SynthesizedCandidate {
    pub fn new(
        topology: CircuitTopology,
        truth_table: &TruthTable,
        evaluator: &CircuitMetricsEvaluator,
    ) -> Self {
        let metrics = evaluator.evaluate(&topology, truth_table);

        // Multi-objective scalar fitness:
        // Heavily reward logic correctness, penalize transistor count and propagation delay
        let correctness_bonus = metrics.logic_correctness * 1000.0;
        let transistor_penalty = (metrics.transistor_count as f64) * 5.0;
        let delay_penalty = metrics.propagation_delay_ps * 2.0;
        let noise_margin_bonus = (metrics.noise_margin_high_v + metrics.noise_margin_low_v) * 50.0;
        let drop_penalty = if metrics.has_threshold_drop {
            50.0
        } else {
            0.0
        };

        let fitness = correctness_bonus + noise_margin_bonus
            - transistor_penalty
            - delay_penalty
            - drop_penalty;

        Self {
            topology,
            metrics,
            fitness,
        }
    }
}

/// Evolutionary synthesizer for unconstrained circuit topologies.
#[derive(Debug, Clone)]
pub struct TopologyEvolver {
    pub config: SynthesisConfig,
}

impl TopologyEvolver {
    pub fn new(config: SynthesisConfig) -> Self {
        Self { config }
    }

    /// Generates an initial diverse population of candidate topologies.
    pub fn initialize_population(&self, seed: u64) -> Vec<SynthesizedCandidate> {
        let mut rng = FastRng::new(seed);
        let num_inputs = self.config.target_truth_table.num_inputs;
        let num_outputs = self.config.target_truth_table.num_outputs;

        let mut population = Vec::with_capacity(self.config.population_size);

        // Seed with canonical presets if suitable for target truth table
        if self.config.target_truth_table.name == "NAND2" {
            population.push(SynthesizedCandidate::new(
                CircuitTopology::static_cmos_nand2(),
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
        } else if self.config.target_truth_table.name == "XOR2" {
            population.push(SynthesizedCandidate::new(
                CircuitTopology::ptl_xor2(),
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
        } else if self.config.target_truth_table.name == "FullAdder1Bit" {
            population.push(SynthesizedCandidate::new(
                CircuitTopology::static_cmos_full_adder_28t(),
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
            population.push(SynthesizedCandidate::new(
                CircuitTopology::hybrid_full_adder_14t(),
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
            population.push(SynthesizedCandidate::new(
                CircuitTopology::ptl_full_adder_10t(),
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
            population.push(SynthesizedCandidate::new(
                CircuitTopology::rtd_mobile_full_adder(),
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
        }

        // Fill remainder with randomized unconstrained topological networks
        while population.len() < self.config.population_size {
            let mut top = CircuitTopology::new(format!("EvolvedCandidate_{}", population.len()));
            let vdd = top.add_node(GateNode::PowerVdd);
            let gnd = top.add_node(GateNode::GroundGnd);

            let mut in_nodes = Vec::new();
            for i in 0..num_inputs {
                in_nodes.push(top.add_node(GateNode::PrimaryInput(i)));
            }

            let mut out_nodes = Vec::new();
            for i in 0..num_outputs {
                out_nodes.push(top.add_node(GateNode::PrimaryOutput(i)));
            }

            let internal_count = rng.next_usize(3) + 1;
            let mut internal_nodes = Vec::new();
            for i in 0..internal_count {
                internal_nodes.push(top.add_node(GateNode::Internal(i)));
            }

            let all_nodes = [
                vec![vdd, gnd],
                in_nodes.clone(),
                out_nodes.clone(),
                internal_nodes.clone(),
            ]
            .concat();
            let num_elements = rng.next_usize(6) + 3;

            for _ in 0..num_elements {
                let n1 = all_nodes[rng.next_usize(all_nodes.len())];
                let n2 = all_nodes[rng.next_usize(all_nodes.len())];
                let n3 = all_nodes[rng.next_usize(all_nodes.len())];

                match rng.next_usize(4) {
                    0 => top.add_element(GateElement::Nmos {
                        drain: n1,
                        gate: n2,
                        source: n3,
                        width_nm: rng.next_range_f64(20.0, 50.0),
                        length_nm: 12.0,
                    }),
                    1 => top.add_element(GateElement::Pmos {
                        drain: n1,
                        gate: n2,
                        source: n3,
                        width_nm: rng.next_range_f64(25.0, 60.0),
                        length_nm: 12.0,
                    }),
                    2 => top.add_element(GateElement::TransmissionGate {
                        input: n1,
                        output: n2,
                        n_gate: n3,
                        p_gate: in_nodes[rng.next_usize(in_nodes.len())],
                        width_nm: rng.next_range_f64(25.0, 45.0),
                    }),
                    _ => top.add_element(GateElement::Ndr {
                        anode: n1,
                        cathode: n2,
                        peak_current_a: rng.next_range_f64(0.8e-3, 1.5e-3),
                    }),
                }
            }

            population.push(SynthesizedCandidate::new(
                top,
                &self.config.target_truth_table,
                &self.config.evaluator,
            ));
        }

        population
    }

    /// Mutates a candidate circuit topology.
    pub fn mutate_topology(&self, topology: &mut CircuitTopology, rng: &mut FastRng) {
        if rng.next_f64() > self.config.mutation_prob {
            return;
        }

        let num_nodes = topology.nodes.len();
        if num_nodes == 0 {
            return;
        }

        match rng.next_usize(4) {
            0 => {
                // Add a pass transistor or transmission gate
                let n1 = rng.next_usize(num_nodes);
                let n2 = rng.next_usize(num_nodes);
                let n3 = rng.next_usize(num_nodes);
                if rng.next_f64() < 0.5 {
                    topology.add_element(GateElement::Nmos {
                        drain: n1,
                        gate: n2,
                        source: n3,
                        width_nm: 30.0,
                        length_nm: 12.0,
                    });
                } else {
                    topology.add_element(GateElement::Pmos {
                        drain: n1,
                        gate: n2,
                        source: n3,
                        width_nm: 35.0,
                        length_nm: 12.0,
                    });
                }
            }
            1 => {
                // Remove an element if redundant (preserving at least 2 elements)
                if topology.elements.len() > 3 {
                    let remove_idx = rng.next_usize(topology.elements.len());
                    topology.elements.swap_remove(remove_idx);
                }
            }
            2 => {
                // Adjust transistor width for drive strength
                let num_elems = topology.elements.len();
                if num_elems > 0 {
                    let idx = rng.next_usize(num_elems);
                    if let Some(elem) = topology.elements.get_mut(idx) {
                        match elem {
                            GateElement::Nmos { width_nm, .. }
                            | GateElement::Pmos { width_nm, .. } => {
                                *width_nm =
                                    (*width_nm + rng.next_range_f64(-5.0, 5.0)).clamp(15.0, 80.0);
                            }
                            GateElement::TransmissionGate { width_nm, .. } => {
                                *width_nm =
                                    (*width_nm + rng.next_range_f64(-5.0, 5.0)).clamp(20.0, 60.0);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {
                // Insert a direct material switch (NDR / RTD)
                let n1 = rng.next_usize(num_nodes);
                let n2 = rng.next_usize(num_nodes);
                topology.add_element(GateElement::Ndr {
                    anode: n1,
                    cathode: n2,
                    peak_current_a: 1.0e-3,
                });
            }
        }
    }

    /// Advances the population by one generation using multi-core Rayon parallel evaluation.
    pub fn evolve_generation(
        &self,
        current_pop: Vec<SynthesizedCandidate>,
        seed: u64,
    ) -> Vec<SynthesizedCandidate> {
        let n = current_pop.len();
        let mut rng = FastRng::new(seed);

        // Generate offspring topologies
        let mut offspring_topologies = Vec::with_capacity(n);
        while offspring_topologies.len() < n {
            let i1 = rng.next_usize(n);
            let i2 = rng.next_usize(n);
            let parent = if current_pop[i1].fitness >= current_pop[i2].fitness {
                &current_pop[i1]
            } else {
                &current_pop[i2]
            };

            let mut child = parent.topology.clone();
            self.mutate_topology(&mut child, &mut rng);
            offspring_topologies.push(child);
        }

        let tt = &self.config.target_truth_table;
        let evaluator = &self.config.evaluator;

        // Parallel evaluation of offspring on all CPU cores
        let offspring: Vec<SynthesizedCandidate> = offspring_topologies
            .into_par_iter()
            .map(|top| SynthesizedCandidate::new(top, tt, evaluator))
            .collect();

        // Elitist selection: pool parents + offspring (2N), sort by fitness descending, truncate to N
        let mut combined = current_pop;
        combined.extend(offspring);
        combined.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        combined.truncate(n);

        combined
    }

    /// Executes multi-generational unconstrained synthesis across CPU cores.
    pub fn run_synthesis(&self, generations: usize, seed: u64) -> Vec<SynthesizedCandidate> {
        let mut population = self.initialize_population(seed);

        for gen in 0..generations {
            let gen_seed = seed
                .wrapping_add(0x8421_1248_cafe)
                .wrapping_mul(gen as u64 + 1);
            population = self.evolve_generation(population, gen_seed);
        }

        // Filter and return physically viable candidates sorted by minimal transistor count
        let mut viable: Vec<_> = population
            .into_iter()
            .filter(|c| c.metrics.is_physically_viable)
            .collect();

        viable.sort_by_key(|c| c.metrics.transistor_count);
        viable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_evolver_initialization() {
        let config = SynthesisConfig {
            population_size: 20,
            target_truth_table: TruthTable::nand2(),
            ..Default::default()
        };
        let evolver = TopologyEvolver::new(config);
        let pop = evolver.initialize_population(100);

        assert_eq!(pop.len(), 20);
        assert!(pop.iter().any(|c| c.metrics.logic_correctness == 1.0));
    }

    #[test]
    fn test_topology_synthesis_run() {
        let config = SynthesisConfig {
            population_size: 24,
            max_generations: 4,
            target_truth_table: TruthTable::xor2(),
            ..Default::default()
        };
        let evolver = TopologyEvolver::new(config);
        let viable = evolver.run_synthesis(4, 999);

        assert!(!viable.is_empty(), "Must discover valid XOR2 topologies");
        let best = &viable[0];
        assert_eq!(best.metrics.logic_correctness, 1.0);
        assert!(best.metrics.transistor_count <= 8);
    }
}
