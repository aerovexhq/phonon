#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic topological time crystals
//! and Floquet-symmetry-enriched phononic memories across multi-threaded Rayon workers.

use crate::topological_time_crystal::time_crystal_solver::TopologicalTimeCrystalSolver;
use phonon_models::topological_time_crystal::{
    TopologicalTimeCrystalMetrics, TopologicalTimeCrystalParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological time crystal parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeCrystalBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_time_crystalline_order_fidelity: f64,
    pub min_time_crystalline_order_fidelity: f64,
    pub max_time_crystalline_order_fidelity: f64,
    pub mean_subharmonic_locking_error: f64,
    pub min_subharmonic_locking_error: f64,
    pub max_subharmonic_locking_error: f64,
    pub mean_temporal_crystalline_lifetime_ms: f64,
    pub min_temporal_crystalline_lifetime_ms: f64,
    pub max_temporal_crystalline_lifetime_ms: f64,
    pub mean_memory_retention_isolation_db: f64,
    pub min_memory_retention_isolation_db: f64,
    pub max_memory_retention_isolation_db: f64,
    pub mean_many_body_localization_ratio: f64,
    pub min_many_body_localization_ratio: f64,
    pub max_many_body_localization_ratio: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct TimeCrystalBenchmarkRunner;

impl TimeCrystalBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> TimeCrystalBenchmarkResult {
        let sweep_params: Vec<TopologicalTimeCrystalParams> = (0..cycles)
            .map(|i| {
                let floquet_drive_period_us = 1.2 + 0.6 * ((i % 13) as f64 / 13.0);
                let imperfect_pulse_rotation_error = 0.016 + 0.0035 * ((i % 17) as f64 / 17.0);
                let inter_resonator_interaction_mhz = 30.0 + 10.0 * ((i % 19) as f64 / 19.0);
                let disorder_potential_strength_mhz = 65.0 + 15.0 * ((i % 23) as f64 / 23.0);
                let acoustic_loss_rate_hz = 6.0 + 1.8 * ((i % 29) as f64 / 29.0);
                let operating_temp_m_k = 8.0 + 1.9 * ((i % 31) as f64 / 31.0);
                let phononic_chain_length = 20 + (i % 9);
                let subharmonic_period_multiplier = 2;

                TopologicalTimeCrystalParams::new(
                    floquet_drive_period_us,
                    imperfect_pulse_rotation_error,
                    inter_resonator_interaction_mhz,
                    disorder_potential_strength_mhz,
                    acoustic_loss_rate_hz,
                    operating_temp_m_k,
                    phononic_chain_length,
                    subharmonic_period_multiplier,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TopologicalTimeCrystalMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalTimeCrystalSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_order_fid = 0.0;
        let mut min_order_fid = f64::MAX;
        let mut max_order_fid = f64::MIN;

        let mut sum_lock_err = 0.0;
        let mut min_lock_err = f64::MAX;
        let mut max_lock_err = f64::MIN;

        let mut sum_lifetime = 0.0;
        let mut min_lifetime = f64::MAX;
        let mut max_lifetime = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_mbl = 0.0;
        let mut min_mbl = f64::MAX;
        let mut max_mbl = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_order_fid += m.time_crystalline_order_fidelity;
            if m.time_crystalline_order_fidelity < min_order_fid {
                min_order_fid = m.time_crystalline_order_fidelity;
            }
            if m.time_crystalline_order_fidelity > max_order_fid {
                max_order_fid = m.time_crystalline_order_fidelity;
            }

            sum_lock_err += m.subharmonic_locking_error;
            if m.subharmonic_locking_error < min_lock_err {
                min_lock_err = m.subharmonic_locking_error;
            }
            if m.subharmonic_locking_error > max_lock_err {
                max_lock_err = m.subharmonic_locking_error;
            }

            sum_lifetime += m.temporal_crystalline_lifetime_ms;
            if m.temporal_crystalline_lifetime_ms < min_lifetime {
                min_lifetime = m.temporal_crystalline_lifetime_ms;
            }
            if m.temporal_crystalline_lifetime_ms > max_lifetime {
                max_lifetime = m.temporal_crystalline_lifetime_ms;
            }

            sum_isolation += m.memory_retention_isolation_db;
            if m.memory_retention_isolation_db < min_isolation {
                min_isolation = m.memory_retention_isolation_db;
            }
            if m.memory_retention_isolation_db > max_isolation {
                max_isolation = m.memory_retention_isolation_db;
            }

            sum_mbl += m.many_body_localization_ratio;
            if m.many_body_localization_ratio < min_mbl {
                min_mbl = m.many_body_localization_ratio;
            }
            if m.many_body_localization_ratio > max_mbl {
                max_mbl = m.many_body_localization_ratio;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        TimeCrystalBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_time_crystalline_order_fidelity: sum_order_fid / n,
            min_time_crystalline_order_fidelity: min_order_fid,
            max_time_crystalline_order_fidelity: max_order_fid,
            mean_subharmonic_locking_error: sum_lock_err / n,
            min_subharmonic_locking_error: min_lock_err,
            max_subharmonic_locking_error: max_lock_err,
            mean_temporal_crystalline_lifetime_ms: sum_lifetime / n,
            min_temporal_crystalline_lifetime_ms: min_lifetime,
            max_temporal_crystalline_lifetime_ms: max_lifetime,
            mean_memory_retention_isolation_db: sum_isolation / n,
            min_memory_retention_isolation_db: min_isolation,
            max_memory_retention_isolation_db: max_isolation,
            mean_many_body_localization_ratio: sum_mbl / n,
            min_many_body_localization_ratio: min_mbl,
            max_many_body_localization_ratio: max_mbl,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
