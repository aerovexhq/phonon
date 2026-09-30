#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic non-Abelian
//! chiral topological skyrmion-vortex polariton networks and non-Clifford
//! geometric braiding engines across multi-threaded Rayon workers.

use crate::skyrmion_vortex_polariton::SkyrmionVortexPolaritonSolver;
use phonon_models::skyrmion_vortex_polariton::{
    SkyrmionVortexPolaritonMetrics, SkyrmionVortexPolaritonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for skyrmion-vortex polariton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionVortexBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_gate_fidelity: f64,
    pub min_gate_fidelity: f64,
    pub max_gate_fidelity: f64,
    pub mean_polariton_state_retention_fraction: f64,
    pub min_polariton_state_retention_fraction: f64,
    pub max_polariton_state_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_polariton_crosstalk_isolation_db: f64,
    pub min_inter_polariton_crosstalk_isolation_db: f64,
    pub max_inter_polariton_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct SkyrmionVortexBenchmarkRunner;

impl SkyrmionVortexBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> SkyrmionVortexBenchmarkResult {
        let sweep_params: Vec<SkyrmionVortexPolaritonParams> = (0..cycles)
            .map(|i| {
                let dzyaloshinskii_moriya_interaction_mev =
                    1.5 + 33.0 * (((i * 7) % 50) as f64 / 50.0);
                let superconducting_vortex_gap_mev =
                    2.5 + 37.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_drive_frequency_ghz =
                    1.2 + 10.5 * (((i * 11) % 40) as f64 / 40.0);
                let skyrmion_shuttling_velocity_m_per_s =
                    220.0 + 2750.0 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 47.0 * (((i * 19) % 45) as f64 / 45.0);
                let microwave_drive_power_uw =
                    0.8 + 28.5 * (((i * 31) % 40) as f64 / 40.0);
                let polariton_core_radius_nm =
                    18.0 + 140.0 * (((i * 17) % 50) as f64 / 50.0);
                let magnetic_anisotropy_energy_mev =
                    0.8 + 23.5 * (((i * 29) % 45) as f64 / 45.0);

                SkyrmionVortexPolaritonParams::new(
                    dzyaloshinskii_moriya_interaction_mev,
                    superconducting_vortex_gap_mev,
                    acoustic_drive_frequency_ghz,
                    skyrmion_shuttling_velocity_m_per_s,
                    cryogenic_temperature_mk,
                    microwave_drive_power_uw,
                    polariton_core_radius_nm,
                    magnetic_anisotropy_energy_mev,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<SkyrmionVortexPolaritonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = SkyrmionVortexPolaritonSolver::new(*p);
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
            sum_fid += m.gate_fidelity;
            min_fid = min_fid.min(m.gate_fidelity);
            max_fid = max_fid.max(m.gate_fidelity);

            sum_ret += m.polariton_state_retention_fraction;
            min_ret = min_ret.min(m.polariton_state_retention_fraction);
            max_ret = max_ret.max(m.polariton_state_retention_fraction);

            sum_gap += m.topological_protection_gap_mhz;
            min_gap = min_gap.min(m.topological_protection_gap_mhz);
            max_gap = max_gap.max(m.topological_protection_gap_mhz);

            sum_iso += m.inter_polariton_crosstalk_isolation_db;
            min_iso = min_iso.min(m.inter_polariton_crosstalk_isolation_db);
            max_iso = max_iso.max(m.inter_polariton_crosstalk_isolation_db);

            sum_deph += m.topological_mode_dephasing_rate_hz;
            min_deph = min_deph.min(m.topological_mode_dephasing_rate_hz);
            max_deph = max_deph.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;

        SkyrmionVortexBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_gate_fidelity: sum_fid / total_f,
            min_gate_fidelity: min_fid,
            max_gate_fidelity: max_fid,
            mean_polariton_state_retention_fraction: sum_ret / total_f,
            min_polariton_state_retention_fraction: min_ret,
            max_polariton_state_retention_fraction: max_ret,
            mean_topological_protection_gap_mhz: sum_gap / total_f,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_polariton_crosstalk_isolation_db: sum_iso / total_f,
            min_inter_polariton_crosstalk_isolation_db: min_iso,
            max_inter_polariton_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz: sum_deph / total_f,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction: (compliant_count as f64) / total_f,
        }
    }
}
