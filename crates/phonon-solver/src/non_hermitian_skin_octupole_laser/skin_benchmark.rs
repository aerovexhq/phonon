#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic non-Hermitian higher-order
//! topological skin sensors and chiral octupole phonon lasers across multi-threaded Rayon workers.

use crate::non_hermitian_skin_octupole_laser::NonHermitianSkinOctupoleLaserSolver;
use phonon_models::non_hermitian_skin_octupole_laser::{
    NonHermitianSkinOctupoleLaserMetrics, NonHermitianSkinOctupoleLaserParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for non-Hermitian skin octupole laser parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkinOctupoleBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_corner_lasing_mode_purity: f64,
    pub min_corner_lasing_mode_purity: f64,
    pub max_corner_lasing_mode_purity: f64,
    pub mean_skin_sensitivity_factor: f64,
    pub min_skin_sensitivity_factor: f64,
    pub max_skin_sensitivity_factor: f64,
    pub mean_higher_order_skin_topological_gap_mhz: f64,
    pub min_higher_order_skin_topological_gap_mhz: f64,
    pub max_higher_order_skin_topological_gap_mhz: f64,
    pub mean_corner_to_bulk_crosstalk_isolation_db: f64,
    pub min_corner_to_bulk_crosstalk_isolation_db: f64,
    pub max_corner_to_bulk_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct SkinOctupoleBenchmarkRunner;

impl SkinOctupoleBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> SkinOctupoleBenchmarkResult {
        let sweep_params: Vec<NonHermitianSkinOctupoleLaserParams> = (0..cycles)
            .map(|i| {
                let non_hermitian_asymmetry_factor =
                    1.10 + 1.80 * (((i * 7) % 50) as f64 / 50.0);
                let octupole_hopping_coupling_mev =
                    6.0 + 38.0 * (((i * 13) % 45) as f64 / 45.0);
                let gain_saturation_intensity_uw =
                    2.0 + 46.0 * (((i * 11) % 40) as f64 / 40.0);
                let pump_rate_normalized =
                    1.20 + 3.60 * (((i * 23) % 35) as f64 / 35.0);
                let acoustic_octupole_frequency_ghz =
                    1.5 + 13.0 * (((i * 31) % 40) as f64 / 40.0);
                let cryogenic_temperature_mk =
                    2.0 + 46.0 * (((i * 19) % 45) as f64 / 45.0);
                let lattice_cell_count_3d =
                    5.0 + 18.0 * (((i * 17) % 50) as f64 / 50.0);
                let skin_localization_decay_length_nm =
                    15.0 + 100.0 * (((i * 29) % 45) as f64 / 45.0);

                NonHermitianSkinOctupoleLaserParams::new(
                    non_hermitian_asymmetry_factor,
                    octupole_hopping_coupling_mev,
                    gain_saturation_intensity_uw,
                    pump_rate_normalized,
                    acoustic_octupole_frequency_ghz,
                    cryogenic_temperature_mk,
                    lattice_cell_count_3d,
                    skin_localization_decay_length_nm,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<NonHermitianSkinOctupoleLaserMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = NonHermitianSkinOctupoleLaserSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut max_purity = f64::MIN;

        let mut sum_sensitivity = 0.0;
        let mut min_sensitivity = f64::MAX;
        let mut max_sensitivity = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_purity += m.corner_lasing_mode_purity;
            min_purity = min_purity.min(m.corner_lasing_mode_purity);
            max_purity = max_purity.max(m.corner_lasing_mode_purity);

            sum_sensitivity += m.skin_sensitivity_factor;
            min_sensitivity = min_sensitivity.min(m.skin_sensitivity_factor);
            max_sensitivity = max_sensitivity.max(m.skin_sensitivity_factor);

            sum_gap += m.higher_order_skin_topological_gap_mhz;
            min_gap = min_gap.min(m.higher_order_skin_topological_gap_mhz);
            max_gap = max_gap.max(m.higher_order_skin_topological_gap_mhz);

            sum_isolation += m.corner_to_bulk_crosstalk_isolation_db;
            min_isolation = min_isolation.min(m.corner_to_bulk_crosstalk_isolation_db);
            max_isolation = max_isolation.max(m.corner_to_bulk_crosstalk_isolation_db);

            sum_dephasing += m.topological_mode_dephasing_rate_hz;
            min_dephasing = min_dephasing.min(m.topological_mode_dephasing_rate_hz);
            max_dephasing = max_dephasing.max(m.topological_mode_dephasing_rate_hz);

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        SkinOctupoleBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_corner_lasing_mode_purity: sum_purity / n,
            min_corner_lasing_mode_purity: min_purity,
            max_corner_lasing_mode_purity: max_purity,
            mean_skin_sensitivity_factor: sum_sensitivity / n,
            min_skin_sensitivity_factor: min_sensitivity,
            max_skin_sensitivity_factor: max_sensitivity,
            mean_higher_order_skin_topological_gap_mhz: sum_gap / n,
            min_higher_order_skin_topological_gap_mhz: min_gap,
            max_higher_order_skin_topological_gap_mhz: max_gap,
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
