#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological acoustic parafermionic fractional
//! Josephson interconnects and non-Abelian quantum logic across multi-threaded Rayon workers.

use crate::fractional_josephson_parafermion::FractionalJosephsonParafermionSolver;
use phonon_models::fractional_josephson_parafermion::{
    FractionalJosephsonParafermionMetrics, FractionalJosephsonParafermionParams,
};
use rayon::prelude::*;
use std::f64::consts::PI;
use std::time::Instant;

/// Benchmark outcome summary for fractional Josephson parafermion parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalJosephsonBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_fractional_braiding_phase_fidelity: f64,
    pub min_fractional_braiding_phase_fidelity: f64,
    pub max_fractional_braiding_phase_fidelity: f64,
    pub mean_fractional_josephson_coherence_ms: f64,
    pub min_fractional_josephson_coherence_ms: f64,
    pub max_fractional_josephson_coherence_ms: f64,
    pub mean_non_adiabatic_excitation_leakage: f64,
    pub min_non_adiabatic_excitation_leakage: f64,
    pub max_non_adiabatic_excitation_leakage: f64,
    pub mean_quasiparticle_parity_poisoning_immunity_db: f64,
    pub min_quasiparticle_parity_poisoning_immunity_db: f64,
    pub max_quasiparticle_parity_poisoning_immunity_db: f64,
    pub mean_fractional_conductance_quantization_error: f64,
    pub min_fractional_conductance_quantization_error: f64,
    pub max_fractional_conductance_quantization_error: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct FractionalJosephsonBenchmarkRunner;

impl FractionalJosephsonBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> FractionalJosephsonBenchmarkResult {
        let sweep_params: Vec<FractionalJosephsonParafermionParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let phase_diff = 0.5 * PI + 4.5 * PI * frac; // 0.5*PI to 5.0*PI rad
                let nu = 0.25 + 0.50 * ((i % 60) as f64 / 60.0); // 0.25 to 0.75
                let pairing_gap = 18.0 + 40.0 * frac; // 18.0 to 58.0 MHz
                let freq_ghz = 2.5 + 6.0 * ((i % 45) as f64 / 45.0); // 2.5 to 8.5 GHz
                let transparency = 0.85 + 0.12 * frac; // 0.85 to 0.97
                let braiding_vel = 600.0 + 1200.0 * ((i % 55) as f64 / 55.0); // 600.0 to 1800.0 m/s
                let cryo_temp = 5.0 + 20.0 * ((i % 40) as f64 / 40.0); // 5.0 to 25.0 mK
                let length_um = 2.0 + 4.0 * frac; // 2.0 to 6.0 um

                FractionalJosephsonParafermionParams::new(
                    phase_diff,
                    nu,
                    pairing_gap,
                    freq_ghz,
                    transparency,
                    braiding_vel,
                    cryo_temp,
                    length_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FractionalJosephsonParafermionMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FractionalJosephsonParafermionSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_coherence = 0.0;
        let mut min_coherence = f64::MAX;
        let mut max_coherence = f64::MIN;

        let mut sum_leakage = 0.0;
        let mut min_leakage = f64::MAX;
        let mut max_leakage = f64::MIN;

        let mut sum_immunity = 0.0;
        let mut min_immunity = f64::MAX;
        let mut max_immunity = f64::MIN;

        let mut sum_error = 0.0;
        let mut min_error = f64::MAX;
        let mut max_error = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.fractional_braiding_phase_fidelity;
            if m.fractional_braiding_phase_fidelity < min_fidelity {
                min_fidelity = m.fractional_braiding_phase_fidelity;
            }
            if m.fractional_braiding_phase_fidelity > max_fidelity {
                max_fidelity = m.fractional_braiding_phase_fidelity;
            }

            sum_coherence += m.fractional_josephson_coherence_ms;
            if m.fractional_josephson_coherence_ms < min_coherence {
                min_coherence = m.fractional_josephson_coherence_ms;
            }
            if m.fractional_josephson_coherence_ms > max_coherence {
                max_coherence = m.fractional_josephson_coherence_ms;
            }

            sum_leakage += m.non_adiabatic_excitation_leakage;
            if m.non_adiabatic_excitation_leakage < min_leakage {
                min_leakage = m.non_adiabatic_excitation_leakage;
            }
            if m.non_adiabatic_excitation_leakage > max_leakage {
                max_leakage = m.non_adiabatic_excitation_leakage;
            }

            sum_immunity += m.quasiparticle_parity_poisoning_immunity_db;
            if m.quasiparticle_parity_poisoning_immunity_db < min_immunity {
                min_immunity = m.quasiparticle_parity_poisoning_immunity_db;
            }
            if m.quasiparticle_parity_poisoning_immunity_db > max_immunity {
                max_immunity = m.quasiparticle_parity_poisoning_immunity_db;
            }

            sum_error += m.fractional_conductance_quantization_error;
            if m.fractional_conductance_quantization_error < min_error {
                min_error = m.fractional_conductance_quantization_error;
            }
            if m.fractional_conductance_quantization_error > max_error {
                max_error = m.fractional_conductance_quantization_error;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;
        let compliance_fraction = (compliant_count as f64) / total_f;

        FractionalJosephsonBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_fractional_braiding_phase_fidelity: sum_fidelity / total_f,
            min_fractional_braiding_phase_fidelity: min_fidelity,
            max_fractional_braiding_phase_fidelity: max_fidelity,
            mean_fractional_josephson_coherence_ms: sum_coherence / total_f,
            min_fractional_josephson_coherence_ms: min_coherence,
            max_fractional_josephson_coherence_ms: max_coherence,
            mean_non_adiabatic_excitation_leakage: sum_leakage / total_f,
            min_non_adiabatic_excitation_leakage: min_leakage,
            max_non_adiabatic_excitation_leakage: max_leakage,
            mean_quasiparticle_parity_poisoning_immunity_db: sum_immunity / total_f,
            min_quasiparticle_parity_poisoning_immunity_db: min_immunity,
            max_quasiparticle_parity_poisoning_immunity_db: max_immunity,
            mean_fractional_conductance_quantization_error: sum_error / total_f,
            min_fractional_conductance_quantization_error: min_error,
            max_fractional_conductance_quantization_error: max_error,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
