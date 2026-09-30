#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic topological chiral
//! fractional quantum Hall phonon entanglement swappers and non-Abelian anyon teleportation
//! bridges across multi-threaded Rayon workers.

use crate::fractional_qh_entanglement_swapper::FractionalQHEntanglementSwapperSolver;
use phonon_models::fractional_qh_entanglement_swapper::{
    FractionalQHEntanglementSwapperMetrics, FractionalQHEntanglementSwapperParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for fractional quantum Hall entanglement swapper parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SwapperBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_bell_state_measurement_fidelity: f64,
    pub min_bell_state_measurement_fidelity: f64,
    pub max_bell_state_measurement_fidelity: f64,
    pub mean_entanglement_teleportation_fidelity: f64,
    pub min_entanglement_teleportation_fidelity: f64,
    pub max_entanglement_teleportation_fidelity: f64,
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
pub struct SwapperBenchmarkRunner;

impl SwapperBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> SwapperBenchmarkResult {
        let sweep_params: Vec<FractionalQHEntanglementSwapperParams> = (0..cycles)
            .map(|i| {
                let filling_factor_nu = 0.25 + 2.10 * (((i * 7) % 50) as f64 / 50.0);
                let topological_tunneling_amplitude_mev =
                    3.0 + 35.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_edge_velocity_m_per_s =
                    600.0 + 3700.0 * (((i * 11) % 40) as f64 / 40.0);
                let anyon_shuttling_distance_um =
                    1.0 + 22.0 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 45.0 * (((i * 19) % 45) as f64 / 45.0);
                let microwave_drive_power_uw =
                    1.0 + 27.0 * (((i * 31) % 40) as f64 / 40.0);
                let heterostructure_dielectric_constant =
                    9.0 + 15.0 * (((i * 17) % 50) as f64 / 50.0);
                let channel_separation_um =
                    0.5 + 9.0 * (((i * 29) % 45) as f64 / 45.0);

                FractionalQHEntanglementSwapperParams::new(
                    filling_factor_nu,
                    topological_tunneling_amplitude_mev,
                    acoustic_edge_velocity_m_per_s,
                    anyon_shuttling_distance_um,
                    cryogenic_temperature_mk,
                    microwave_drive_power_uw,
                    heterostructure_dielectric_constant,
                    channel_separation_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<FractionalQHEntanglementSwapperMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = FractionalQHEntanglementSwapperSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_bsm = 0.0;
        let mut min_bsm = f64::MAX;
        let mut max_bsm = f64::MIN;

        let mut sum_tele = 0.0;
        let mut min_tele = f64::MAX;
        let mut max_tele = f64::MIN;

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
            sum_bsm += m.bell_state_measurement_fidelity;
            min_bsm = min_bsm.min(m.bell_state_measurement_fidelity);
            max_bsm = max_bsm.max(m.bell_state_measurement_fidelity);

            sum_tele += m.entanglement_teleportation_fidelity;
            min_tele = min_tele.min(m.entanglement_teleportation_fidelity);
            max_tele = max_tele.max(m.entanglement_teleportation_fidelity);

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

        SwapperBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_bell_state_measurement_fidelity: sum_bsm / total_f,
            min_bell_state_measurement_fidelity: min_bsm,
            max_bell_state_measurement_fidelity: max_bsm,
            mean_entanglement_teleportation_fidelity: sum_tele / total_f,
            min_entanglement_teleportation_fidelity: min_tele,
            max_entanglement_teleportation_fidelity: max_tele,
            mean_topological_protection_gap_mhz: sum_gap / total_f,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_channel_crosstalk_isolation_db: sum_iso / total_f,
            min_inter_channel_crosstalk_isolation_db: min_iso,
            max_inter_channel_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz: sum_deph / total_f,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction: (compliant_count as f64) / total_f,
        }
    }
}
