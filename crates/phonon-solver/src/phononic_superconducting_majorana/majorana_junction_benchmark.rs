#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian chiral Majorana bound states
//! in topological phononic superconducting junctions.

use crate::phononic_superconducting_majorana::PhononicSuperconductingMajoranaSolver;
use phonon_models::phononic_superconducting_majorana::{
    PhononicSuperconductingMajoranaMetrics, PhononicSuperconductingMajoranaParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for phononic superconducting Majorana parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicMajoranaBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_braiding_phase_fidelity: f64,
    pub min_braiding_phase_fidelity: f64,
    pub max_braiding_phase_fidelity: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_non_adiabatic_leakage_probability: f64,
    pub min_non_adiabatic_leakage_probability: f64,
    pub max_non_adiabatic_leakage_probability: f64,
    pub mean_quasiparticle_poisoning_immunity_db: f64,
    pub min_quasiparticle_poisoning_immunity_db: f64,
    pub max_quasiparticle_poisoning_immunity_db: f64,
    pub mean_zero_bias_conductance_error_g0: f64,
    pub min_zero_bias_conductance_error_g0: f64,
    pub max_zero_bias_conductance_error_g0: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct PhononicMajoranaBenchmarkRunner;

impl PhononicMajoranaBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> PhononicMajoranaBenchmarkResult {
        let sweep_params: Vec<PhononicSuperconductingMajoranaParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let gap_sc = 35.0 + 35.0 * frac; // 35.0 to 70.0 MHz
                let so_coupling = 50.0 + 40.0 * ((i % 50) as f64 / 50.0); // 50.0 to 90.0 meV*nm
                let zeeman = 65.0 + 45.0 * frac; // 65.0 to 110.0 MHz
                let drive_freq = 2.0 + 2.8 * ((i % 60) as f64 / 60.0); // 2.0 to 4.8 GHz
                let strain_ppm = 80.0 + 120.0 * frac; // 80.0 to 200.0 ppm
                let temp_mk = 4.0 + 12.0 * ((i % 40) as f64 / 40.0); // 4.0 to 16.0 mK
                let transparency = 0.86 + 0.11 * frac; // 0.86 to 0.97
                let length_um = 2.0 + 2.0 * ((i % 70) as f64 / 70.0); // 2.0 to 4.0 um

                PhononicSuperconductingMajoranaParams::new(
                    gap_sc,
                    so_coupling,
                    zeeman,
                    drive_freq,
                    strain_ppm,
                    temp_mk,
                    transparency,
                    length_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<PhononicSuperconductingMajoranaMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = PhononicSuperconductingMajoranaSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_leak = 0.0;
        let mut min_leak = f64::MAX;
        let mut max_leak = f64::MIN;

        let mut sum_immunity = 0.0;
        let mut min_immunity = f64::MAX;
        let mut max_immunity = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.braiding_phase_fidelity;
            if m.braiding_phase_fidelity < min_fidelity {
                min_fidelity = m.braiding_phase_fidelity;
            }
            if m.braiding_phase_fidelity > max_fidelity {
                max_fidelity = m.braiding_phase_fidelity;
            }

            sum_gap += m.topological_protection_gap_mhz;
            if m.topological_protection_gap_mhz < min_gap {
                min_gap = m.topological_protection_gap_mhz;
            }
            if m.topological_protection_gap_mhz > max_gap {
                max_gap = m.topological_protection_gap_mhz;
            }

            sum_leak += m.non_adiabatic_leakage_probability;
            if m.non_adiabatic_leakage_probability < min_leak {
                min_leak = m.non_adiabatic_leakage_probability;
            }
            if m.non_adiabatic_leakage_probability > max_leak {
                max_leak = m.non_adiabatic_leakage_probability;
            }

            sum_immunity += m.quasiparticle_poisoning_immunity_db;
            if m.quasiparticle_poisoning_immunity_db < min_immunity {
                min_immunity = m.quasiparticle_poisoning_immunity_db;
            }
            if m.quasiparticle_poisoning_immunity_db > max_immunity {
                max_immunity = m.quasiparticle_poisoning_immunity_db;
            }

            sum_error += m.zero_bias_conductance_error_g0;
            if m.zero_bias_conductance_error_g0 < min_error {
                min_error = m.zero_bias_conductance_error_g0;
            }
            if m.zero_bias_conductance_error_g0 > max_error {
                max_error = m.zero_bias_conductance_error_g0;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        PhononicMajoranaBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_braiding_phase_fidelity: sum_fidelity / n,
            min_braiding_phase_fidelity: min_fidelity,
            max_braiding_phase_fidelity: max_fidelity,
            mean_topological_protection_gap_mhz: sum_gap / n,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_non_adiabatic_leakage_probability: sum_leak / n,
            min_non_adiabatic_leakage_probability: min_leak,
            max_non_adiabatic_leakage_probability: max_leak,
            mean_quasiparticle_poisoning_immunity_db: sum_immunity / n,
            min_quasiparticle_poisoning_immunity_db: min_immunity,
            max_quasiparticle_poisoning_immunity_db: max_immunity,
            mean_zero_bias_conductance_error_g0: sum_error / n,
            min_zero_bias_conductance_error_g0: min_error,
            max_zero_bias_conductance_error_g0: max_error,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
