#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for topological moire acoustic
//! polaritonic lattices and flat-band phonon superfluidity across Rayon workers.

use crate::topological_moire_polariton::TopologicalMoirePolaritonSolver;
use phonon_models::topological_moire_polariton::{
    TopologicalMoirePolaritonMetrics, TopologicalMoirePolaritonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for topological moire polariton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoirePolaritonBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_superfluid_velocity_m_per_s: f64,
    pub min_superfluid_velocity_m_per_s: f64,
    pub max_superfluid_velocity_m_per_s: f64,
    pub mean_propagation_loss_db_per_cm: f64,
    pub min_propagation_loss_db_per_cm: f64,
    pub max_propagation_loss_db_per_cm: f64,
    pub mean_condensation_threshold_density: f64,
    pub min_condensation_threshold_density: f64,
    pub max_condensation_threshold_density: f64,
    pub mean_chern_number: f64,
    pub min_chern_number: i32,
    pub max_chern_number: i32,
    pub mean_flat_band_bandwidth_mhz: f64,
    pub min_flat_band_bandwidth_mhz: f64,
    pub max_flat_band_bandwidth_mhz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct MoirePolaritonBenchmarkRunner;

impl MoirePolaritonBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> MoirePolaritonBenchmarkResult {
        let sweep_params: Vec<TopologicalMoirePolaritonParams> = (0..cycles)
            .map(|i| {
                let twist_angle_deg = 1.04 + 0.08 * ((i % 17) as f64 / 17.0);
                let acoustic_center_freq_ghz = 3.8 + 0.8 * ((i % 19) as f64 / 19.0);
                let interlayer_tunneling_mhz = 50.0 + 10.0 * ((i % 23) as f64 / 23.0);
                let moire_period_nm = 170.0 + 20.0 * ((i % 29) as f64 / 29.0);
                let non_linear_polariton_interaction_uev_um2 =
                    5.0 + 1.2 * ((i % 31) as f64 / 31.0);
                let operating_temp_m_k = 12.0 + 5.0 * ((i % 37) as f64 / 37.0);
                let polariton_lifetime_ps = 320.0 + 60.0 * ((i % 41) as f64 / 41.0);
                let acoustic_quality_factor = 2.2e7 + 0.6e7 * ((i % 43) as f64 / 43.0);

                TopologicalMoirePolaritonParams::new(
                    twist_angle_deg,
                    acoustic_center_freq_ghz,
                    interlayer_tunneling_mhz,
                    moire_period_nm,
                    non_linear_polariton_interaction_uev_um2,
                    operating_temp_m_k,
                    polariton_lifetime_ps,
                    acoustic_quality_factor,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TopologicalMoirePolaritonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TopologicalMoirePolaritonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_v_s = 0.0;
        let mut min_v_s = f64::MAX;
        let mut max_v_s = f64::MIN;

        let mut sum_loss = 0.0;
        let mut min_loss = f64::MAX;
        let mut max_loss = f64::MIN;

        let mut sum_nth = 0.0;
        let mut min_nth = f64::MAX;
        let mut max_nth = f64::MIN;

        let mut sum_chern = 0.0;
        let mut min_chern = i32::MAX;
        let mut max_chern = i32::MIN;

        let mut sum_w = 0.0;
        let mut min_w = f64::MAX;
        let mut max_w = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_v_s += m.superfluid_velocity_m_per_s;
            if m.superfluid_velocity_m_per_s < min_v_s {
                min_v_s = m.superfluid_velocity_m_per_s;
            }
            if m.superfluid_velocity_m_per_s > max_v_s {
                max_v_s = m.superfluid_velocity_m_per_s;
            }

            sum_loss += m.propagation_loss_db_per_cm;
            if m.propagation_loss_db_per_cm < min_loss {
                min_loss = m.propagation_loss_db_per_cm;
            }
            if m.propagation_loss_db_per_cm > max_loss {
                max_loss = m.propagation_loss_db_per_cm;
            }

            sum_nth += m.condensation_threshold_density;
            if m.condensation_threshold_density < min_nth {
                min_nth = m.condensation_threshold_density;
            }
            if m.condensation_threshold_density > max_nth {
                max_nth = m.condensation_threshold_density;
            }

            sum_chern += m.chern_number as f64;
            if m.chern_number < min_chern {
                min_chern = m.chern_number;
            }
            if m.chern_number > max_chern {
                max_chern = m.chern_number;
            }

            sum_w += m.flat_band_bandwidth_mhz;
            if m.flat_band_bandwidth_mhz < min_w {
                min_w = m.flat_band_bandwidth_mhz;
            }
            if m.flat_band_bandwidth_mhz > max_w {
                max_w = m.flat_band_bandwidth_mhz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles.max(1) as f64;
        MoirePolaritonBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_superfluid_velocity_m_per_s: sum_v_s / n,
            min_superfluid_velocity_m_per_s: min_v_s,
            max_superfluid_velocity_m_per_s: max_v_s,
            mean_propagation_loss_db_per_cm: sum_loss / n,
            min_propagation_loss_db_per_cm: min_loss,
            max_propagation_loss_db_per_cm: max_loss,
            mean_condensation_threshold_density: sum_nth / n,
            min_condensation_threshold_density: min_nth,
            max_condensation_threshold_density: max_nth,
            mean_chern_number: sum_chern / n,
            min_chern_number: min_chern,
            max_chern_number: max_chern,
            mean_flat_band_bandwidth_mhz: sum_w / n,
            min_flat_band_bandwidth_mhz: min_w,
            max_flat_band_bandwidth_mhz: max_w,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
