#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for hybrid superconducting
//! opto-acoustic quantum repeaters and entanglement distribution networks
//! across multi-threaded Rayon workers.

use crate::opto_acoustic_quantum_repeater::OptoAcousticQuantumRepeaterSolver;
use phonon_models::opto_acoustic_quantum_repeater::{
    OptoAcousticQuantumRepeaterMetrics, OptoAcousticQuantumRepeaterParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum repeater network parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepeaterBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_bell_state_fidelity: f64,
    pub min_bell_state_fidelity: f64,
    pub max_bell_state_fidelity: f64,
    pub mean_repetition_rate_khz: f64,
    pub min_repetition_rate_khz: f64,
    pub max_repetition_rate_khz: f64,
    pub mean_distribution_latency_us: f64,
    pub min_distribution_latency_us: f64,
    pub max_distribution_latency_us: f64,
    pub mean_memory_transduction_roundtrip_fidelity: f64,
    pub min_memory_transduction_roundtrip_fidelity: f64,
    pub max_memory_transduction_roundtrip_fidelity: f64,
    pub mean_purification_efficiency: f64,
    pub min_purification_efficiency: f64,
    pub max_purification_efficiency: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct RepeaterBenchmarkRunner;

impl RepeaterBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> RepeaterBenchmarkResult {
        let sweep_params: Vec<OptoAcousticQuantumRepeaterParams> = (0..cycles)
            .map(|i| {
                let repeater_nodes_count = 2 + (i % 4);
                let channel_distance_km = 25.0 + 25.0 * ((i % 40) as f64 / 40.0);
                let transducer_efficiency = 0.80 + 0.15 * ((i % 25) as f64 / 25.0);
                let acoustic_memory_coherence_ms = 10.0 + 20.0 * ((i % 30) as f64 / 30.0);
                let optical_fiber_attenuation_db_per_km = 0.18 + 0.10 * ((i % 20) as f64 / 20.0);
                let purification_rounds = 1 + (i % 3);
                let operating_temp_m_k = 5.0 + 20.0 * ((i % 35) as f64 / 35.0);
                let pump_repetition_freq_mhz = 6.0 + 6.0 * ((i % 30) as f64 / 30.0);

                OptoAcousticQuantumRepeaterParams::new(
                    repeater_nodes_count,
                    channel_distance_km,
                    transducer_efficiency,
                    acoustic_memory_coherence_ms,
                    optical_fiber_attenuation_db_per_km,
                    purification_rounds,
                    operating_temp_m_k,
                    pump_repetition_freq_mhz,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<OptoAcousticQuantumRepeaterMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = OptoAcousticQuantumRepeaterSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_bell = 0.0;
        let mut min_bell = f64::MAX;
        let mut max_bell = f64::MIN;

        let mut sum_rate = 0.0;
        let mut min_rate = f64::MAX;
        let mut max_rate = f64::MIN;

        let mut sum_lat = 0.0;
        let mut min_lat = f64::MAX;
        let mut max_lat = f64::MIN;

        let mut sum_roundtrip = 0.0;
        let mut min_roundtrip = f64::MAX;
        let mut max_roundtrip = f64::MIN;

        let mut sum_pur = 0.0;
        let mut min_pur = f64::MAX;
        let mut max_pur = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_bell += m.bell_state_fidelity;
            if m.bell_state_fidelity < min_bell {
                min_bell = m.bell_state_fidelity;
            }
            if m.bell_state_fidelity > max_bell {
                max_bell = m.bell_state_fidelity;
            }

            sum_rate += m.repetition_rate_khz;
            if m.repetition_rate_khz < min_rate {
                min_rate = m.repetition_rate_khz;
            }
            if m.repetition_rate_khz > max_rate {
                max_rate = m.repetition_rate_khz;
            }

            sum_lat += m.distribution_latency_us;
            if m.distribution_latency_us < min_lat {
                min_lat = m.distribution_latency_us;
            }
            if m.distribution_latency_us > max_lat {
                max_lat = m.distribution_latency_us;
            }

            sum_roundtrip += m.memory_transduction_roundtrip_fidelity;
            if m.memory_transduction_roundtrip_fidelity < min_roundtrip {
                min_roundtrip = m.memory_transduction_roundtrip_fidelity;
            }
            if m.memory_transduction_roundtrip_fidelity > max_roundtrip {
                max_roundtrip = m.memory_transduction_roundtrip_fidelity;
            }

            sum_pur += m.purification_efficiency;
            if m.purification_efficiency < min_pur {
                min_pur = m.purification_efficiency;
            }
            if m.purification_efficiency > max_pur {
                max_pur = m.purification_efficiency;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        RepeaterBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_bell_state_fidelity: sum_bell / n,
            min_bell_state_fidelity: min_bell,
            max_bell_state_fidelity: max_bell,
            mean_repetition_rate_khz: sum_rate / n,
            min_repetition_rate_khz: min_rate,
            max_repetition_rate_khz: max_rate,
            mean_distribution_latency_us: sum_lat / n,
            min_distribution_latency_us: min_lat,
            max_distribution_latency_us: max_lat,
            mean_memory_transduction_roundtrip_fidelity: sum_roundtrip / n,
            min_memory_transduction_roundtrip_fidelity: min_roundtrip,
            max_memory_transduction_roundtrip_fidelity: max_roundtrip,
            mean_purification_efficiency: sum_pur / n,
            min_purification_efficiency: min_pur,
            max_purification_efficiency: max_pur,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
