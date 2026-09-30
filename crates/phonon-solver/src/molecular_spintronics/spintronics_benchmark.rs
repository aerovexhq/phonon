#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for the Phonon Universal Multi-Scale
//! Visual Studio Autonomous Molecular Spintronic Qubit Interface & Diamond NV-Center Acoustic Transducer Engine
//! across multi-threaded Rayon workers.

use crate::molecular_spintronics::MolecularSpintronicsSolver;
use phonon_models::molecular_spintronics::{
    MolecularSpintronicsMetrics, MolecularSpintronicsParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for autonomous molecular spintronic qubit interface sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MolecularSpintronicsBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_transduction_fidelity: f64,
    pub min_transduction_fidelity: f64,
    pub max_transduction_fidelity: f64,
    pub mean_spin_state_retention_fraction: f64,
    pub min_spin_state_retention_fraction: f64,
    pub max_spin_state_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_qubit_crosstalk_isolation_db: f64,
    pub min_inter_qubit_crosstalk_isolation_db: f64,
    pub max_inter_qubit_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct MolecularSpintronicsBenchmarkRunner;

impl MolecularSpintronicsBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> MolecularSpintronicsBenchmarkResult {
        let sweep_params: Vec<MolecularSpintronicsParams> = (0..cycles)
            .map(|i| {
                let spintronic_coupling_mev =
                    1.0 + 34.0 * (((i * 7) % 50) as f64 / 50.0);
                let topological_spintronic_gap_mev =
                    2.0 + 43.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_drive_frequency_ghz =
                    1.0 + 11.0 * (((i * 23) % 35) as f64 / 35.0);
                let spin_acoustic_dispatch_speed_m_per_s =
                    200.0 + 2800.0 * (((i * 17) % 50) as f64 / 50.0);
                let cryogenic_temperature_mk =
                    1.0 + 49.0 * (((i * 29) % 45) as f64 / 45.0);
                let microwave_probe_power_uw =
                    0.5 + 29.5 * (((i * 31) % 40) as f64 / 40.0);
                let synthetic_nv_cavity_factor =
                    1.0 + 7.0 * (((i * 11) % 40) as f64 / 40.0);
                let spintronic_cell_pitch_um =
                    0.5 + 19.5 * (((i * 19) % 45) as f64 / 45.0);

                MolecularSpintronicsParams::new(
                    spintronic_coupling_mev,
                    topological_spintronic_gap_mev,
                    acoustic_drive_frequency_ghz,
                    spin_acoustic_dispatch_speed_m_per_s,
                    cryogenic_temperature_mk,
                    microwave_probe_power_uw,
                    synthetic_nv_cavity_factor,
                    spintronic_cell_pitch_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<MolecularSpintronicsMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = MolecularSpintronicsSolver::new(*p);
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
            sum_fid += m.transduction_fidelity;
            if m.transduction_fidelity < min_fid {
                min_fid = m.transduction_fidelity;
            }
            if m.transduction_fidelity > max_fid {
                max_fid = m.transduction_fidelity;
            }

            sum_ret += m.spin_state_retention_fraction;
            if m.spin_state_retention_fraction < min_ret {
                min_ret = m.spin_state_retention_fraction;
            }
            if m.spin_state_retention_fraction > max_ret {
                max_ret = m.spin_state_retention_fraction;
            }

            sum_gap += m.topological_protection_gap_mhz;
            if m.topological_protection_gap_mhz < min_gap {
                min_gap = m.topological_protection_gap_mhz;
            }
            if m.topological_protection_gap_mhz > max_gap {
                max_gap = m.topological_protection_gap_mhz;
            }

            sum_iso += m.inter_qubit_crosstalk_isolation_db;
            if m.inter_qubit_crosstalk_isolation_db < min_iso {
                min_iso = m.inter_qubit_crosstalk_isolation_db;
            }
            if m.inter_qubit_crosstalk_isolation_db > max_iso {
                max_iso = m.inter_qubit_crosstalk_isolation_db;
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

        let n = cycles as f64;
        MolecularSpintronicsBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_transduction_fidelity: sum_fid / n,
            min_transduction_fidelity: min_fid,
            max_transduction_fidelity: max_fid,
            mean_spin_state_retention_fraction: sum_ret / n,
            min_spin_state_retention_fraction: min_ret,
            max_spin_state_retention_fraction: max_ret,
            mean_topological_protection_gap_mhz: sum_gap / n,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_qubit_crosstalk_isolation_db: sum_iso / n,
            min_inter_qubit_crosstalk_isolation_db: min_iso,
            max_inter_qubit_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz: sum_deph / n,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
