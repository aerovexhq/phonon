#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for chiral phononic Floquet-SBT gauge fields
//! and dissipationless acoustic topological Hall transistors across multi-threaded Rayon workers.

use crate::chiral_floquet_hall_transistor::ChiralFloquetHallTransistorSolver;
use phonon_models::chiral_floquet_hall_transistor::{
    ChiralFloquetHallTransistorMetrics, ChiralFloquetHallTransistorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral phononic Floquet Hall transistor parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetHallTransistorBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_valley_hall_contrast_ratio_db: f64,
    pub min_valley_hall_contrast_ratio_db: f64,
    pub max_valley_hall_contrast_ratio_db: f64,
    pub mean_topological_switching_time_ns: f64,
    pub min_topological_switching_time_ns: f64,
    pub max_topological_switching_time_ns: f64,
    pub mean_cross_talk_isolation_db: f64,
    pub min_cross_talk_isolation_db: f64,
    pub max_cross_talk_isolation_db: f64,
    pub mean_non_adiabatic_insertion_loss_db: f64,
    pub min_non_adiabatic_insertion_loss_db: f64,
    pub max_non_adiabatic_insertion_loss_db: f64,
    pub mean_hall_transistor_state_fidelity: f64,
    pub min_hall_transistor_state_fidelity: f64,
    pub max_hall_transistor_state_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct FloquetHallTransistorBenchmarkRunner;

impl FloquetHallTransistorBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> FloquetHallTransistorBenchmarkResult {
        let sweep_params: Vec<ChiralFloquetHallTransistorParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let floquet_amp = 25.0 + 30.0 * frac; // 25.0 to 55.0 MHz
                let drive_freq = 3.2 + 3.3 * ((i % 55) as f64 / 55.0); // 3.2 to 6.5 GHz
                let strain_torsion = 60.0 + 60.0 * frac; // 60.0 to 120.0 ppm/um
                let valley_coupling = 14.0 + 12.0 * ((i % 45) as f64 / 45.0); // 14.0 to 26.0 MHz
                let gate_voltage = 2.0 + 2.0 * frac; // 2.0 to 4.0 V
                let channel_len = 3.0 + 2.8 * ((i % 65) as f64 / 65.0); // 3.0 to 5.8 um
                let temp_mk = 8.0 + 14.0 * ((i % 40) as f64 / 40.0); // 8.0 to 22.0 mK
                let electromech = 0.06 + 0.06 * frac; // 0.06 to 0.12

                ChiralFloquetHallTransistorParams::new(
                    floquet_amp,
                    drive_freq,
                    strain_torsion,
                    valley_coupling,
                    gate_voltage,
                    channel_len,
                    temp_mk,
                    electromech,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralFloquetHallTransistorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralFloquetHallTransistorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_contrast = 0.0;
        let mut min_contrast = f64::MAX;
        let mut max_contrast = f64::MIN;

        let mut sum_switching = 0.0;
        let mut min_switching = f64::MAX;
        let mut max_switching = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_loss = 0.0;
        let mut min_loss = f64::MAX;
        let mut max_loss = f64::MIN;

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_contrast += m.valley_hall_contrast_ratio_db;
            if m.valley_hall_contrast_ratio_db < min_contrast {
                min_contrast = m.valley_hall_contrast_ratio_db;
            }
            if m.valley_hall_contrast_ratio_db > max_contrast {
                max_contrast = m.valley_hall_contrast_ratio_db;
            }

            sum_switching += m.topological_switching_time_ns;
            if m.topological_switching_time_ns < min_switching {
                min_switching = m.topological_switching_time_ns;
            }
            if m.topological_switching_time_ns > max_switching {
                max_switching = m.topological_switching_time_ns;
            }

            sum_isolation += m.cross_talk_isolation_db;
            if m.cross_talk_isolation_db < min_isolation {
                min_isolation = m.cross_talk_isolation_db;
            }
            if m.cross_talk_isolation_db > max_isolation {
                max_isolation = m.cross_talk_isolation_db;
            }

            sum_loss += m.non_adiabatic_insertion_loss_db;
            if m.non_adiabatic_insertion_loss_db < min_loss {
                min_loss = m.non_adiabatic_insertion_loss_db;
            }
            if m.non_adiabatic_insertion_loss_db > max_loss {
                max_loss = m.non_adiabatic_insertion_loss_db;
            }

            sum_fidelity += m.hall_transistor_state_fidelity;
            if m.hall_transistor_state_fidelity < min_fidelity {
                min_fidelity = m.hall_transistor_state_fidelity;
            }
            if m.hall_transistor_state_fidelity > max_fidelity {
                max_fidelity = m.hall_transistor_state_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;
        let compliance_fraction = (compliant_count as f64) / total_f;

        FloquetHallTransistorBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_valley_hall_contrast_ratio_db: sum_contrast / total_f,
            min_valley_hall_contrast_ratio_db: min_contrast,
            max_valley_hall_contrast_ratio_db: max_contrast,
            mean_topological_switching_time_ns: sum_switching / total_f,
            min_topological_switching_time_ns: min_switching,
            max_topological_switching_time_ns: max_switching,
            mean_cross_talk_isolation_db: sum_isolation / total_f,
            min_cross_talk_isolation_db: min_isolation,
            max_cross_talk_isolation_db: max_isolation,
            mean_non_adiabatic_insertion_loss_db: sum_loss / total_f,
            min_non_adiabatic_insertion_loss_db: min_loss,
            max_non_adiabatic_insertion_loss_db: max_loss,
            mean_hall_transistor_state_fidelity: sum_fidelity / total_f,
            min_hall_transistor_state_fidelity: min_fidelity,
            max_hall_transistor_state_fidelity: max_fidelity,
            physical_compliance_fraction: compliance_fraction,
        }
    }
}
