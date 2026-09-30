#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic non-Abelian
//! chiral topological anyonic knot invariant quantum co-processors and Chern-Simons
//! calculators across multi-threaded Rayon workers.

use crate::anyonic_knot_coprocessor::AnyonicKnotCoprocessorSolver;
use phonon_models::anyonic_knot_coprocessor::{
    AnyonicKnotCoprocessorMetrics, AnyonicKnotCoprocessorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for anyonic knot coprocessor parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnyonicKnotBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_knot_calculation_fidelity: f64,
    pub min_knot_calculation_fidelity: f64,
    pub max_knot_calculation_fidelity: f64,
    pub mean_anyon_state_retention_fraction: f64,
    pub min_anyon_state_retention_fraction: f64,
    pub max_anyon_state_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_knot_crosstalk_isolation_db: f64,
    pub min_inter_knot_crosstalk_isolation_db: f64,
    pub max_inter_knot_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct AnyonicKnotBenchmarkRunner;

impl AnyonicKnotBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> AnyonicKnotBenchmarkResult {
        let sweep_params: Vec<AnyonicKnotCoprocessorParams> = (0..cycles)
            .map(|i| {
                let braid_crossing_coupling_mev =
                    2.0 + 32.0 * (((i * 7) % 50) as f64 / 50.0);
                let chern_simons_level_k =
                    2.0 + 9.5 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_drive_frequency_ghz =
                    1.2 + 10.5 * (((i * 23) % 35) as f64 / 35.0);
                let knot_braiding_speed_m_per_s =
                    250.0 + 2700.0 * (((i * 17) % 50) as f64 / 50.0);
                let cryogenic_temperature_mk =
                    2.0 + 47.0 * (((i * 31) % 40) as f64 / 40.0);
                let microwave_interferometer_power_uw =
                    0.8 + 28.5 * (((i * 11) % 40) as f64 / 40.0);
                let anyon_link_closure_radius_nm =
                    25.0 + 170.0 * (((i * 19) % 45) as f64 / 45.0);
                let knot_complexity_crossings =
                    4.0 + 19.0 * (((i * 29) % 45) as f64 / 45.0);

                AnyonicKnotCoprocessorParams::new(
                    braid_crossing_coupling_mev,
                    chern_simons_level_k,
                    acoustic_drive_frequency_ghz,
                    knot_braiding_speed_m_per_s,
                    cryogenic_temperature_mk,
                    microwave_interferometer_power_uw,
                    anyon_link_closure_radius_nm,
                    knot_complexity_crossings,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<AnyonicKnotCoprocessorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = AnyonicKnotCoprocessorSolver::new(*p);
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
            sum_fid += m.knot_calculation_fidelity;
            min_fid = min_fid.min(m.knot_calculation_fidelity);
            max_fid = max_fid.max(m.knot_calculation_fidelity);

            sum_ret += m.anyon_state_retention_fraction;
            min_ret = min_ret.min(m.anyon_state_retention_fraction);
            max_ret = max_ret.max(m.anyon_state_retention_fraction);

            sum_gap += m.topological_protection_gap_mhz;
            min_gap = min_gap.min(m.topological_protection_gap_mhz);
            max_gap = max_gap.max(m.topological_protection_gap_mhz);

            sum_iso += m.inter_knot_crosstalk_isolation_db;
            min_iso = min_iso.min(m.inter_knot_crosstalk_isolation_db);
            max_iso = max_iso.max(m.inter_knot_crosstalk_isolation_db);

            sum_deph += m.topological_mode_dephasing_rate_hz;
            min_deph = min_deph.min(m.topological_mode_dephasing_rate_hz);
            max_deph = max_deph.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;

        AnyonicKnotBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_knot_calculation_fidelity: sum_fid / total_f,
            min_knot_calculation_fidelity: min_fid,
            max_knot_calculation_fidelity: max_fid,
            mean_anyon_state_retention_fraction: sum_ret / total_f,
            min_anyon_state_retention_fraction: min_ret,
            max_anyon_state_retention_fraction: max_ret,
            mean_topological_protection_gap_mhz: sum_gap / total_f,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_knot_crosstalk_isolation_db: sum_iso / total_f,
            min_inter_knot_crosstalk_isolation_db: min_iso,
            max_inter_knot_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz: sum_deph / total_f,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction: (compliant_count as f64) / total_f,
        }
    }
}
