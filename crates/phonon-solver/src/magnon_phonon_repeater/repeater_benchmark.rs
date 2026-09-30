#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for the Phonon Universal Multi-Scale
//! Visual Studio Autonomous Acoustically Mediated Magnon-Phonon Entanglement Swapping &
//! Quantum Repeater Node Engine across multi-threaded Rayon workers.

use crate::magnon_phonon_repeater::MagnonPhononRepeaterSolver;
use phonon_models::magnon_phonon_repeater::{
    MagnonPhononRepeaterMetrics, MagnonPhononRepeaterParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for autonomous magnon-phonon quantum repeater sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnonPhononRepeaterBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_entanglement_swapping_fidelity: f64,
    pub min_entanglement_swapping_fidelity: f64,
    pub max_entanglement_swapping_fidelity: f64,
    pub mean_repeater_state_retention_fraction: f64,
    pub min_repeater_state_retention_fraction: f64,
    pub max_repeater_state_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_node_crosstalk_isolation_db: f64,
    pub min_inter_node_crosstalk_isolation_db: f64,
    pub max_inter_node_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MagnonPhononRepeaterBenchmarkRunner;

impl MagnonPhononRepeaterBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> MagnonPhononRepeaterBenchmarkResult {
        let sweep_params: Vec<MagnonPhononRepeaterParams> = (0..cycles)
            .map(|i| {
                let swapping_coupling_mev =
                    1.0 + 34.0 * (((i * 7) % 50) as f64 / 50.0);
                let topological_repeater_gap_mev =
                    2.0 + 43.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_drive_frequency_ghz =
                    1.0 + 11.0 * (((i * 23) % 35) as f64 / 35.0);
                let repeater_dispatch_speed_m_per_s =
                    200.0 + 2800.0 * (((i * 17) % 50) as f64 / 50.0);
                let cryogenic_temperature_mk =
                    1.0 + 49.0 * (((i * 29) % 45) as f64 / 45.0);
                let microwave_probe_power_uw =
                    0.5 + 29.5 * (((i * 31) % 40) as f64 / 40.0);
                let synthetic_repeater_nodes_factor =
                    1.0 + 7.0 * (((i * 11) % 40) as f64 / 40.0);
                let repeater_node_pitch_um =
                    0.5 + 19.5 * (((i * 19) % 45) as f64 / 45.0);

                MagnonPhononRepeaterParams::new(
                    swapping_coupling_mev,
                    topological_repeater_gap_mev,
                    acoustic_drive_frequency_ghz,
                    repeater_dispatch_speed_m_per_s,
                    cryogenic_temperature_mk,
                    microwave_probe_power_uw,
                    synthetic_repeater_nodes_factor,
                    repeater_node_pitch_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MagnonPhononRepeaterMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = MagnonPhononRepeaterSolver::new(*p);
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

        let mut compliant_count = 0usize;

        for m in &metrics {
            sum_fid += m.entanglement_swapping_fidelity;
            if m.entanglement_swapping_fidelity < min_fid {
                min_fid = m.entanglement_swapping_fidelity;
            }
            if m.entanglement_swapping_fidelity > max_fid {
                max_fid = m.entanglement_swapping_fidelity;
            }

            sum_ret += m.repeater_state_retention_fraction;
            if m.repeater_state_retention_fraction < min_ret {
                min_ret = m.repeater_state_retention_fraction;
            }
            if m.repeater_state_retention_fraction > max_ret {
                max_ret = m.repeater_state_retention_fraction;
            }

            sum_gap += m.topological_protection_gap_mhz;
            if m.topological_protection_gap_mhz < min_gap {
                min_gap = m.topological_protection_gap_mhz;
            }
            if m.topological_protection_gap_mhz > max_gap {
                max_gap = m.topological_protection_gap_mhz;
            }

            sum_iso += m.inter_node_crosstalk_isolation_db;
            if m.inter_node_crosstalk_isolation_db < min_iso {
                min_iso = m.inter_node_crosstalk_isolation_db;
            }
            if m.inter_node_crosstalk_isolation_db > max_iso {
                max_iso = m.inter_node_crosstalk_isolation_db;
            }

            sum_deph += m.topological_mode_dephasing_rate_hz;
            if m.topological_mode_dephasing_rate_hz < min_deph {
                min_deph = m.topological_mode_dephasing_rate_hz;
            }
            if m.topological_mode_dephasing_rate_hz > max_deph {
                max_deph = m.topological_mode_dephasing_rate_hz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let total_f = cycles as f64;
        let mean_entanglement_swapping_fidelity = sum_fid / total_f;
        let mean_repeater_state_retention_fraction = sum_ret / total_f;
        let mean_topological_protection_gap_mhz = sum_gap / total_f;
        let mean_inter_node_crosstalk_isolation_db = sum_iso / total_f;
        let mean_topological_mode_dephasing_rate_hz = sum_deph / total_f;
        let physical_compliance_fraction = (compliant_count as f64) / total_f;

        MagnonPhononRepeaterBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_entanglement_swapping_fidelity,
            min_entanglement_swapping_fidelity: min_fid,
            max_entanglement_swapping_fidelity: max_fid,
            mean_repeater_state_retention_fraction,
            min_repeater_state_retention_fraction: min_ret,
            max_repeater_state_retention_fraction: max_ret,
            mean_topological_protection_gap_mhz,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_node_crosstalk_isolation_db,
            min_inter_node_crosstalk_isolation_db: min_iso,
            max_inter_node_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction,
        }
    }
}
