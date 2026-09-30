#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic higher-order
//! topological quadrupole-octupole superlattices and non-Hermitian corner metasurfaces
//! across multi-threaded Rayon workers.

use crate::hotp_quadrupole_octupole_metasurface::HotpQuadrupoleOctupoleSolver;
use phonon_models::hotp_quadrupole_octupole_metasurface::{
    HotpQuadrupoleOctupoleMetrics, HotpQuadrupoleOctupoleParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quadrupole-octupole superlattice parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotpBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_corner_state_localization_fidelity: f64,
    pub min_corner_state_localization_fidelity: f64,
    pub max_corner_state_localization_fidelity: f64,
    pub mean_higher_order_topological_gap_mhz: f64,
    pub min_higher_order_topological_gap_mhz: f64,
    pub max_higher_order_topological_gap_mhz: f64,
    pub mean_multipole_topological_charge: f64,
    pub min_multipole_topological_charge: f64,
    pub max_multipole_topological_charge: f64,
    pub mean_corner_to_bulk_crosstalk_isolation_db: f64,
    pub min_corner_to_bulk_crosstalk_isolation_db: f64,
    pub max_corner_to_bulk_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct HotpBenchmarkRunner;

impl HotpBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> HotpBenchmarkResult {
        let sweep_params: Vec<HotpQuadrupoleOctupoleParams> = (0..cycles)
            .map(|i| {
                let intra_cell_hopping_gamma = 0.15 + 0.70 * (((i * 7) % 50) as f64 / 50.0);
                let inter_cell_hopping_lambda = 0.90 + 1.50 * (((i * 13) % 45) as f64 / 45.0);
                let non_hermitian_gain_loss_gamma =
                    0.02 + 0.35 * (((i * 11) % 40) as f64 / 40.0);
                let acoustic_corner_frequency_ghz =
                    1.5 + 13.0 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk = 2.0 + 45.0 * (((i * 19) % 45) as f64 / 45.0);
                let multipole_order = if (i % 3) == 0 { 3.0 } else { 2.0 };
                let superlattice_dimension_cells =
                    8.0 + 22.0 * (((i * 31) % 40) as f64 / 40.0);
                let synthetic_gauge_flux_pi = 0.85 + 0.30 * (((i * 17) % 50) as f64 / 50.0);

                HotpQuadrupoleOctupoleParams::new(
                    intra_cell_hopping_gamma,
                    inter_cell_hopping_lambda,
                    non_hermitian_gain_loss_gamma,
                    acoustic_corner_frequency_ghz,
                    cryogenic_temperature_mk,
                    multipole_order,
                    superlattice_dimension_cells,
                    synthetic_gauge_flux_pi,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<HotpQuadrupoleOctupoleMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = HotpQuadrupoleOctupoleSolver::new(*p);
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

        let mut sum_charge = 0.0;
        let mut min_charge = f64::MAX;
        let mut max_charge = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.corner_state_localization_fidelity;
            if m.corner_state_localization_fidelity < min_fidelity {
                min_fidelity = m.corner_state_localization_fidelity;
            }
            if m.corner_state_localization_fidelity > max_fidelity {
                max_fidelity = m.corner_state_localization_fidelity;
            }

            sum_gap += m.higher_order_topological_gap_mhz;
            if m.higher_order_topological_gap_mhz < min_gap {
                min_gap = m.higher_order_topological_gap_mhz;
            }
            if m.higher_order_topological_gap_mhz > max_gap {
                max_gap = m.higher_order_topological_gap_mhz;
            }

            sum_charge += m.multipole_topological_charge;
            if m.multipole_topological_charge < min_charge {
                min_charge = m.multipole_topological_charge;
            }
            if m.multipole_topological_charge > max_charge {
                max_charge = m.multipole_topological_charge;
            }

            sum_isolation += m.corner_to_bulk_crosstalk_isolation_db;
            if m.corner_to_bulk_crosstalk_isolation_db < min_isolation {
                min_isolation = m.corner_to_bulk_crosstalk_isolation_db;
            }
            if m.corner_to_bulk_crosstalk_isolation_db > max_isolation {
                max_isolation = m.corner_to_bulk_crosstalk_isolation_db;
            }

            sum_dephasing += m.topological_mode_dephasing_rate_hz;
            if m.topological_mode_dephasing_rate_hz < min_dephasing {
                min_dephasing = m.topological_mode_dephasing_rate_hz;
            }
            if m.topological_mode_dephasing_rate_hz > max_dephasing {
                max_dephasing = m.topological_mode_dephasing_rate_hz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        HotpBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_corner_state_localization_fidelity: sum_fidelity / n,
            min_corner_state_localization_fidelity: min_fidelity,
            max_corner_state_localization_fidelity: max_fidelity,
            mean_higher_order_topological_gap_mhz: sum_gap / n,
            min_higher_order_topological_gap_mhz: min_gap,
            max_higher_order_topological_gap_mhz: max_gap,
            mean_multipole_topological_charge: sum_charge / n,
            min_multipole_topological_charge: min_charge,
            max_multipole_topological_charge: max_charge,
            mean_corner_to_bulk_crosstalk_isolation_db: sum_isolation / n,
            min_corner_to_bulk_crosstalk_isolation_db: min_isolation,
            max_corner_to_bulk_crosstalk_isolation_db: max_isolation,
            mean_topological_mode_dephasing_rate_hz: sum_dephasing / n,
            min_topological_mode_dephasing_rate_hz: min_dephasing,
            max_topological_mode_dephasing_rate_hz: max_dephasing,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
