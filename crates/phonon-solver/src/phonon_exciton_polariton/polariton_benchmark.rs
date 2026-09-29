#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum phonon-exciton
//! polariton condensates and chiral optomechanical transducers across Rayon workers.

use crate::phonon_exciton_polariton::PhononExcitonPolaritonSolver;
use phonon_models::phonon_exciton_polariton::{
    PhononExcitonPolaritonMetrics, PhononExcitonPolaritonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for phonon-exciton polariton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_quantum_state_fidelity: f64,
    pub min_quantum_state_fidelity: f64,
    pub max_quantum_state_fidelity: f64,
    pub mean_condensation_threshold_pump_mw: f64,
    pub min_condensation_threshold_pump_mw: f64,
    pub max_condensation_threshold_pump_mw: f64,
    pub mean_polariton_coherence_time_ps: f64,
    pub min_polariton_coherence_time_ps: f64,
    pub max_polariton_coherence_time_ps: f64,
    pub mean_vortex_topological_charge: f64,
    pub min_vortex_topological_charge: i32,
    pub max_vortex_topological_charge: i32,
    pub mean_optomechanical_coupling_rate_mhz: f64,
    pub min_optomechanical_coupling_rate_mhz: f64,
    pub max_optomechanical_coupling_rate_mhz: f64,
    pub physical_compliance_fraction: f64,
}

pub type PhononExcitonPolaritonBenchmarkResult = PolaritonBenchmarkResult;

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct PolaritonBenchmarkRunner;

pub type PhononExcitonPolaritonBenchmarkRunner = PolaritonBenchmarkRunner;

impl PolaritonBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> PolaritonBenchmarkResult {
        let sweep_params: Vec<PhononExcitonPolaritonParams> = (0..cycles)
            .map(|i| {
                let optical_cavity_freq_thz = 370.0 + 10.0 * ((i % 17) as f64 / 17.0);
                let acoustic_phonon_freq_ghz = 6.0 + 2.0 * ((i % 19) as f64 / 19.0);
                let exciton_binding_energy_mev = 26.0 + 4.0 * ((i % 23) as f64 / 23.0);
                let rabi_splitting_energy_mev = 11.0 + 2.0 * ((i % 29) as f64 / 29.0);
                let piezo_deform_coupling_mhz = 52.0 + 6.0 * ((i % 31) as f64 / 31.0);
                let optical_pump_power_mw = 2.2 + 0.6 * ((i % 37) as f64 / 37.0);
                let operating_temp_k = 0.25 + 0.08 * ((i % 41) as f64 / 41.0);
                let cavity_quality_factor = 1.4e5 + 2.0e4 * ((i % 43) as f64 / 43.0);

                PhononExcitonPolaritonParams::new(
                    optical_cavity_freq_thz,
                    acoustic_phonon_freq_ghz,
                    exciton_binding_energy_mev,
                    rabi_splitting_energy_mev,
                    piezo_deform_coupling_mhz,
                    optical_pump_power_mw,
                    operating_temp_k,
                    cavity_quality_factor,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<PhononExcitonPolaritonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = PhononExcitonPolaritonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_pth = 0.0;
        let mut min_pth = f64::MAX;
        let mut max_pth = f64::MIN;

        let mut sum_tau = 0.0;
        let mut min_tau = f64::MAX;
        let mut max_tau = f64::MIN;

        let mut sum_q = 0.0;
        let mut min_q = i32::MAX;
        let mut max_q = i32::MIN;

        let mut sum_gom = 0.0;
        let mut min_gom = f64::MAX;
        let mut max_gom = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.quantum_state_fidelity;
            if m.quantum_state_fidelity < min_fid {
                min_fid = m.quantum_state_fidelity;
            }
            if m.quantum_state_fidelity > max_fid {
                max_fid = m.quantum_state_fidelity;
            }

            sum_pth += m.condensation_threshold_pump_mw;
            if m.condensation_threshold_pump_mw < min_pth {
                min_pth = m.condensation_threshold_pump_mw;
            }
            if m.condensation_threshold_pump_mw > max_pth {
                max_pth = m.condensation_threshold_pump_mw;
            }

            sum_tau += m.polariton_coherence_time_ps;
            if m.polariton_coherence_time_ps < min_tau {
                min_tau = m.polariton_coherence_time_ps;
            }
            if m.polariton_coherence_time_ps > max_tau {
                max_tau = m.polariton_coherence_time_ps;
            }

            sum_q += m.vortex_topological_charge as f64;
            if m.vortex_topological_charge < min_q {
                min_q = m.vortex_topological_charge;
            }
            if m.vortex_topological_charge > max_q {
                max_q = m.vortex_topological_charge;
            }

            sum_gom += m.optomechanical_coupling_rate_mhz;
            if m.optomechanical_coupling_rate_mhz < min_gom {
                min_gom = m.optomechanical_coupling_rate_mhz;
            }
            if m.optomechanical_coupling_rate_mhz > max_gom {
                max_gom = m.optomechanical_coupling_rate_mhz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        PolaritonBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_quantum_state_fidelity: sum_fid / n,
            min_quantum_state_fidelity: min_fid,
            max_quantum_state_fidelity: max_fid,
            mean_condensation_threshold_pump_mw: sum_pth / n,
            min_condensation_threshold_pump_mw: min_pth,
            max_condensation_threshold_pump_mw: max_pth,
            mean_polariton_coherence_time_ps: sum_tau / n,
            min_polariton_coherence_time_ps: min_tau,
            max_polariton_coherence_time_ps: max_tau,
            mean_vortex_topological_charge: sum_q / n,
            min_vortex_topological_charge: min_q,
            max_vortex_topological_charge: max_q,
            mean_optomechanical_coupling_rate_mhz: sum_gom / n,
            min_optomechanical_coupling_rate_mhz: min_gom,
            max_optomechanical_coupling_rate_mhz: max_gom,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
