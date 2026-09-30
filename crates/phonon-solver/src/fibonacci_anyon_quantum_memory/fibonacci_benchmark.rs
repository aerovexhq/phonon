#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic non-Abelian anyonic
//! quantum memory and chiral Fibonacci braiding gate fabrics across multi-threaded Rayon workers.

use crate::fibonacci_anyon_quantum_memory::FibonacciAnyonQuantumMemorySolver;
use phonon_models::fibonacci_anyon_quantum_memory::{
    FibonacciAnyonQuantumMemoryMetrics, FibonacciAnyonQuantumMemoryParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for Fibonacci anyon quantum memory parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibonacciBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_braiding_gate_fidelity: f64,
    pub min_braiding_gate_fidelity: f64,
    pub max_braiding_gate_fidelity: f64,
    pub mean_anyon_memory_retention_fraction: f64,
    pub min_anyon_memory_retention_fraction: f64,
    pub max_anyon_memory_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_qubit_crosstalk_isolation_db: f64,
    pub min_inter_qubit_crosstalk_isolation_db: f64,
    pub max_inter_qubit_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct FibonacciBenchmarkRunner;

impl FibonacciBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> FibonacciBenchmarkResult {
        let sweep_params: Vec<FibonacciAnyonQuantumMemoryParams> = (0..cycles)
            .map(|i| {
                let golden_ratio_tau = 1.52 + 0.16 * (((i * 7) % 50) as f64 / 50.0);
                let topological_gap_energy_mhz =
                    32.0 + 56.0 * (((i * 13) % 45) as f64 / 45.0);
                let braid_word_length = 12.0 + 85.0 * (((i * 11) % 40) as f64 / 40.0);
                let acoustic_clock_frequency_ghz =
                    1.2 + 13.5 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 46.0 * (((i * 31) % 40) as f64 / 40.0);
                let inter_anyon_separation_um =
                    0.8 + 7.0 * (((i * 19) % 45) as f64 / 45.0);
                let memory_retention_time_us =
                    15.0 + 470.0 * (((i * 17) % 50) as f64 / 50.0);
                let strain_shuttling_velocity_mps =
                    250.0 + 2700.0 * (((i * 29) % 45) as f64 / 45.0);

                FibonacciAnyonQuantumMemoryParams::new(
                    golden_ratio_tau,
                    topological_gap_energy_mhz,
                    braid_word_length,
                    acoustic_clock_frequency_ghz,
                    cryogenic_temperature_mk,
                    inter_anyon_separation_um,
                    memory_retention_time_us,
                    strain_shuttling_velocity_mps,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FibonacciAnyonQuantumMemoryMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FibonacciAnyonQuantumMemorySolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_retention = 0.0;
        let mut min_retention = f64::MAX;
        let mut max_retention = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_crosstalk = 0.0;
        let mut min_crosstalk = f64::MAX;
        let mut max_crosstalk = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.braiding_gate_fidelity;
            min_fidelity = min_fidelity.min(m.braiding_gate_fidelity);
            max_fidelity = max_fidelity.max(m.braiding_gate_fidelity);

            sum_retention += m.anyon_memory_retention_fraction;
            min_retention = min_retention.min(m.anyon_memory_retention_fraction);
            max_retention = max_retention.max(m.anyon_memory_retention_fraction);

            sum_gap += m.topological_protection_gap_mhz;
            min_gap = min_gap.min(m.topological_protection_gap_mhz);
            max_gap = max_gap.max(m.topological_protection_gap_mhz);

            sum_crosstalk += m.inter_qubit_crosstalk_isolation_db;
            min_crosstalk = min_crosstalk.min(m.inter_qubit_crosstalk_isolation_db);
            max_crosstalk = max_crosstalk.max(m.inter_qubit_crosstalk_isolation_db);

            sum_dephasing += m.topological_mode_dephasing_rate_hz;
            min_dephasing = min_dephasing.min(m.topological_mode_dephasing_rate_hz);
            max_dephasing = max_dephasing.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        FibonacciBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_braiding_gate_fidelity: sum_fidelity / n,
            min_braiding_gate_fidelity: min_fidelity,
            max_braiding_gate_fidelity: max_fidelity,
            mean_anyon_memory_retention_fraction: sum_retention / n,
            min_anyon_memory_retention_fraction: min_retention,
            max_anyon_memory_retention_fraction: max_retention,
            mean_topological_protection_gap_mhz: sum_gap / n,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_qubit_crosstalk_isolation_db: sum_crosstalk / n,
            min_inter_qubit_crosstalk_isolation_db: min_crosstalk,
            max_inter_qubit_crosstalk_isolation_db: max_crosstalk,
            mean_topological_mode_dephasing_rate_hz: sum_dephasing / n,
            min_topological_mode_dephasing_rate_hz: min_dephasing,
            max_topological_mode_dephasing_rate_hz: max_dephasing,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
