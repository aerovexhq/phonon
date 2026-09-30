#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic non-Abelian
//! chiral topological hyperbolic lattice anyon crystallizers and fractal boundary engines
//! across multi-threaded Rayon workers.

use crate::hyperbolic_crystallizer::HyperbolicCrystallizerSolver;
use phonon_models::hyperbolic_crystallizer::{
    HyperbolicCrystallizerMetrics, HyperbolicCrystallizerParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for hyperbolic anyon crystallizer parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyperbolicCrystallizerBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_crystallization_fidelity: f64,
    pub min_crystallization_fidelity: f64,
    pub max_crystallization_fidelity: f64,
    pub mean_crystallized_state_retention_fraction: f64,
    pub min_crystallized_state_retention_fraction: f64,
    pub max_crystallized_state_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_channel_crosstalk_isolation_db: f64,
    pub min_inter_channel_crosstalk_isolation_db: f64,
    pub max_inter_channel_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct HyperbolicCrystallizerBenchmarkRunner;

impl HyperbolicCrystallizerBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> HyperbolicCrystallizerBenchmarkResult {
        let sweep_params: Vec<HyperbolicCrystallizerParams> = (0..cycles)
            .map(|i| {
                let hyperbolic_coupling_energy_mev =
                    2.0 + 32.0 * (((i * 7) % 50) as f64 / 50.0);
                let topological_fractal_gap_mev =
                    3.0 + 40.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_drive_frequency_ghz =
                    1.2 + 10.5 * (((i * 23) % 35) as f64 / 35.0);
                let crystallization_drift_speed_m_per_s =
                    250.0 + 2700.0 * (((i * 17) % 50) as f64 / 50.0);
                let cryogenic_temperature_mk =
                    2.0 + 47.0 * (((i * 31) % 40) as f64 / 40.0);
                let microwave_pinning_power_uw =
                    0.8 + 28.5 * (((i * 11) % 40) as f64 / 40.0);
                let synthetic_curvature_radius_um =
                    0.2 + 4.6 * (((i * 19) % 45) as f64 / 45.0);
                let hyperbolic_tessellation_pitch_um =
                    1.0 + 18.5 * (((i * 29) % 45) as f64 / 45.0);

                HyperbolicCrystallizerParams::new(
                    hyperbolic_coupling_energy_mev,
                    topological_fractal_gap_mev,
                    acoustic_drive_frequency_ghz,
                    crystallization_drift_speed_m_per_s,
                    cryogenic_temperature_mk,
                    microwave_pinning_power_uw,
                    synthetic_curvature_radius_um,
                    hyperbolic_tessellation_pitch_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<HyperbolicCrystallizerMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = HyperbolicCrystallizerSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_ret = 0.0;
        let mut min_ret = f64::MAX;
        let mut max_ret = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_deph = 0.0;
        let mut min_deph = f64::MAX;
        let mut max_deph = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fid += m.crystallization_fidelity;
            min_fid = min_fid.min(m.crystallization_fidelity);
            max_fid = max_fid.max(m.crystallization_fidelity);

            sum_ret += m.crystallized_state_retention_fraction;
            min_ret = min_ret.min(m.crystallized_state_retention_fraction);
            max_ret = max_ret.max(m.crystallized_state_retention_fraction);

            sum_gap += m.topological_protection_gap_mhz;
            min_gap = min_gap.min(m.topological_protection_gap_mhz);
            max_gap = max_gap.max(m.topological_protection_gap_mhz);

            sum_iso += m.inter_channel_crosstalk_isolation_db;
            min_iso = min_iso.min(m.inter_channel_crosstalk_isolation_db);
            max_iso = max_iso.max(m.inter_channel_crosstalk_isolation_db);

            sum_deph += m.topological_mode_dephasing_rate_hz;
            min_deph = min_deph.min(m.topological_mode_dephasing_rate_hz);
            max_deph = max_deph.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;

        HyperbolicCrystallizerBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_crystallization_fidelity: sum_fid / total_f,
            min_crystallization_fidelity: min_fid,
            max_crystallization_fidelity: max_fid,
            mean_crystallized_state_retention_fraction: sum_ret / total_f,
            min_crystallized_state_retention_fraction: min_ret,
            max_crystallized_state_retention_fraction: max_ret,
            mean_topological_protection_gap_mhz: sum_gap / total_f,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_channel_crosstalk_isolation_db: sum_iso / total_f,
            min_inter_channel_crosstalk_isolation_db: min_iso,
            max_inter_channel_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz: sum_deph / total_f,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction: compliant_count as f64 / total_f,
        }
    }
}
