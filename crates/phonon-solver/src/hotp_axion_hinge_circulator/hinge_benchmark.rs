#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic higher-order axion
//! electrodynamics and chiral quadrupole-hinge polariton circulators across multi-threaded
//! Rayon workers.

use crate::hotp_axion_hinge_circulator::HotpAxionHingeCirculatorSolver;
use phonon_models::hotp_axion_hinge_circulator::{
    HotpAxionHingeCirculatorMetrics, HotpAxionHingeCirculatorParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for higher-order axion hinge circulator parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotpAxionHingeBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_hinge_polariton_transmission_fidelity: f64,
    pub min_hinge_polariton_transmission_fidelity: f64,
    pub max_hinge_polariton_transmission_fidelity: f64,
    pub mean_higher_order_topological_gap_mhz: f64,
    pub min_higher_order_topological_gap_mhz: f64,
    pub max_higher_order_topological_gap_mhz: f64,
    pub mean_dynamic_non_reciprocal_isolation_db: f64,
    pub min_dynamic_non_reciprocal_isolation_db: f64,
    pub max_dynamic_non_reciprocal_isolation_db: f64,
    pub mean_inter_hinge_crosstalk_isolation_db: f64,
    pub min_inter_hinge_crosstalk_isolation_db: f64,
    pub max_inter_hinge_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct HotpAxionHingeBenchmarkRunner;

impl HotpAxionHingeBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> HotpAxionHingeBenchmarkResult {
        let sweep_params: Vec<HotpAxionHingeCirculatorParams> = (0..cycles)
            .map(|i| {
                let axion_angle_theta_pi = 0.82 + 0.36 * (((i * 7) % 50) as f64 / 50.0);
                let quadrupole_polarization_qxy =
                    0.37 + 0.26 * (((i * 13) % 45) as f64 / 45.0);
                let magnetoelectric_hinge_coupling_alpha =
                    0.15 + 0.78 * (((i * 11) % 40) as f64 / 40.0);
                let acoustic_hinge_frequency_ghz =
                    1.2 + 13.5 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 46.0 * (((i * 31) % 40) as f64 / 40.0);
                let hinge_channel_length_um =
                    1.5 + 18.0 * (((i * 19) % 45) as f64 / 45.0);
                let inter_hinge_separation_um =
                    0.8 + 9.0 * (((i * 17) % 50) as f64 / 50.0);
                let cavity_resonance_quality_factor =
                    1.5e4 + 4.7e5 * (((i * 29) % 45) as f64 / 45.0);

                HotpAxionHingeCirculatorParams::new(
                    axion_angle_theta_pi,
                    quadrupole_polarization_qxy,
                    magnetoelectric_hinge_coupling_alpha,
                    acoustic_hinge_frequency_ghz,
                    cryogenic_temperature_mk,
                    hinge_channel_length_um,
                    inter_hinge_separation_um,
                    cavity_resonance_quality_factor,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<HotpAxionHingeCirculatorMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = HotpAxionHingeCirculatorSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_non_reciprocal = 0.0;
        let mut min_non_reciprocal = f64::MAX;
        let mut max_non_reciprocal = f64::MIN;

        let mut sum_crosstalk = 0.0;
        let mut min_crosstalk = f64::MAX;
        let mut max_crosstalk = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.hinge_polariton_transmission_fidelity;
            min_fidelity = min_fidelity.min(m.hinge_polariton_transmission_fidelity);
            max_fidelity = max_fidelity.max(m.hinge_polariton_transmission_fidelity);

            sum_gap += m.higher_order_topological_gap_mhz;
            min_gap = min_gap.min(m.higher_order_topological_gap_mhz);
            max_gap = max_gap.max(m.higher_order_topological_gap_mhz);

            sum_non_reciprocal += m.dynamic_non_reciprocal_isolation_db;
            min_non_reciprocal = min_non_reciprocal.min(m.dynamic_non_reciprocal_isolation_db);
            max_non_reciprocal = max_non_reciprocal.max(m.dynamic_non_reciprocal_isolation_db);

            sum_crosstalk += m.inter_hinge_crosstalk_isolation_db;
            min_crosstalk = min_crosstalk.min(m.inter_hinge_crosstalk_isolation_db);
            max_crosstalk = max_crosstalk.max(m.inter_hinge_crosstalk_isolation_db);

            sum_dephasing += m.topological_mode_dephasing_rate_hz;
            min_dephasing = min_dephasing.min(m.topological_mode_dephasing_rate_hz);
            max_dephasing = max_dephasing.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        HotpAxionHingeBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_hinge_polariton_transmission_fidelity: sum_fidelity / n,
            min_hinge_polariton_transmission_fidelity: min_fidelity,
            max_hinge_polariton_transmission_fidelity: max_fidelity,
            mean_higher_order_topological_gap_mhz: sum_gap / n,
            min_higher_order_topological_gap_mhz: min_gap,
            max_higher_order_topological_gap_mhz: max_gap,
            mean_dynamic_non_reciprocal_isolation_db: sum_non_reciprocal / n,
            min_dynamic_non_reciprocal_isolation_db: min_non_reciprocal,
            max_dynamic_non_reciprocal_isolation_db: max_non_reciprocal,
            mean_inter_hinge_crosstalk_isolation_db: sum_crosstalk / n,
            min_inter_hinge_crosstalk_isolation_db: min_crosstalk,
            max_inter_hinge_crosstalk_isolation_db: max_crosstalk,
            mean_topological_mode_dephasing_rate_hz: sum_dephasing / n,
            min_topological_mode_dephasing_rate_hz: min_dephasing,
            max_topological_mode_dephasing_rate_hz: max_dephasing,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
