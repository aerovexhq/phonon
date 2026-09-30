#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for non-Abelian quantum acoustic twisted
//! bilayer topological superfluidity and chiral Majorana vortex networks across multi-threaded
//! Rayon workers.

use crate::twisted_bilayer_topological_superfluid::TwistedBilayerTopologicalSuperfluidSolver;
use phonon_models::twisted_bilayer_topological_superfluid::{
    TwistedBilayerTopologicalSuperfluidMetrics, TwistedBilayerTopologicalSuperfluidParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for twisted bilayer topological superfluid parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistedBilayerSuperfluidBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_vortex_state_fidelity: f64,
    pub min_vortex_state_fidelity: f64,
    pub max_vortex_state_fidelity: f64,
    pub mean_topological_vortex_pinning_gap_mhz: f64,
    pub min_topological_vortex_pinning_gap_mhz: f64,
    pub max_topological_vortex_pinning_gap_mhz: f64,
    pub mean_inter_vortex_crosstalk_isolation_db: f64,
    pub min_inter_vortex_crosstalk_isolation_db: f64,
    pub max_inter_vortex_crosstalk_isolation_db: f64,
    pub mean_topological_vortex_dephasing_rate_hz: f64,
    pub min_topological_vortex_dephasing_rate_hz: f64,
    pub max_topological_vortex_dephasing_rate_hz: f64,
    pub mean_chiral_majorana_mode_purity: f64,
    pub min_chiral_majorana_mode_purity: f64,
    pub max_chiral_majorana_mode_purity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct TwistedBilayerSuperfluidBenchmarkRunner;

impl TwistedBilayerSuperfluidBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> TwistedBilayerSuperfluidBenchmarkResult {
        let sweep_params: Vec<TwistedBilayerTopologicalSuperfluidParams> = (0..cycles)
            .map(|i| {
                let twist_angle_degrees = 0.85 + 0.50 * (((i * 7) % 50) as f64 / 50.0);
                let interlayer_josephson_coupling_mev =
                    7.0 + 41.0 * (((i * 13) % 45) as f64 / 45.0);
                let p_wave_pairing_amplitude_mev =
                    3.0 + 26.0 * (((i * 11) % 40) as f64 / 40.0);
                let acoustic_vortex_frequency_ghz =
                    1.5 + 13.0 * (((i * 23) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk = 2.0 + 46.0 * (((i * 19) % 45) as f64 / 45.0);
                let vortex_core_radius_nm = 15.0 + 125.0 * (((i * 31) % 40) as f64 / 40.0);
                let inter_vortex_separation_um = 0.8 + 8.7 * (((i * 17) % 50) as f64 / 50.0);
                let pinning_potential_barrier_mev =
                    1.5 + 22.5 * (((i * 29) % 45) as f64 / 45.0);

                TwistedBilayerTopologicalSuperfluidParams::new(
                    twist_angle_degrees,
                    interlayer_josephson_coupling_mev,
                    p_wave_pairing_amplitude_mev,
                    acoustic_vortex_frequency_ghz,
                    cryogenic_temperature_mk,
                    vortex_core_radius_nm,
                    inter_vortex_separation_um,
                    pinning_potential_barrier_mev,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TwistedBilayerTopologicalSuperfluidMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TwistedBilayerTopologicalSuperfluidSolver::new(*p);
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

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_dephasing = 0.0;
        let mut min_dephasing = f64::MAX;
        let mut max_dephasing = f64::MIN;

        let mut sum_purity = 0.0;
        let mut min_purity = f64::MAX;
        let mut max_purity = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.vortex_state_fidelity;
            if m.vortex_state_fidelity < min_fidelity {
                min_fidelity = m.vortex_state_fidelity;
            }
            if m.vortex_state_fidelity > max_fidelity {
                max_fidelity = m.vortex_state_fidelity;
            }

            sum_gap += m.topological_vortex_pinning_gap_mhz;
            if m.topological_vortex_pinning_gap_mhz < min_gap {
                min_gap = m.topological_vortex_pinning_gap_mhz;
            }
            if m.topological_vortex_pinning_gap_mhz > max_gap {
                max_gap = m.topological_vortex_pinning_gap_mhz;
            }

            sum_isolation += m.inter_vortex_crosstalk_isolation_db;
            if m.inter_vortex_crosstalk_isolation_db < min_isolation {
                min_isolation = m.inter_vortex_crosstalk_isolation_db;
            }
            if m.inter_vortex_crosstalk_isolation_db > max_isolation {
                max_isolation = m.inter_vortex_crosstalk_isolation_db;
            }

            sum_dephasing += m.topological_vortex_dephasing_rate_hz;
            if m.topological_vortex_dephasing_rate_hz < min_dephasing {
                min_dephasing = m.topological_vortex_dephasing_rate_hz;
            }
            if m.topological_vortex_dephasing_rate_hz > max_dephasing {
                max_dephasing = m.topological_vortex_dephasing_rate_hz;
            }

            sum_purity += m.chiral_majorana_mode_purity;
            if m.chiral_majorana_mode_purity < min_purity {
                min_purity = m.chiral_majorana_mode_purity;
            }
            if m.chiral_majorana_mode_purity > max_purity {
                max_purity = m.chiral_majorana_mode_purity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;

        TwistedBilayerSuperfluidBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_vortex_state_fidelity: sum_fidelity / n,
            min_vortex_state_fidelity: min_fidelity,
            max_vortex_state_fidelity: max_fidelity,
            mean_topological_vortex_pinning_gap_mhz: sum_gap / n,
            min_topological_vortex_pinning_gap_mhz: min_gap,
            max_topological_vortex_pinning_gap_mhz: max_gap,
            mean_inter_vortex_crosstalk_isolation_db: sum_isolation / n,
            min_inter_vortex_crosstalk_isolation_db: min_isolation,
            max_inter_vortex_crosstalk_isolation_db: max_isolation,
            mean_topological_vortex_dephasing_rate_hz: sum_dephasing / n,
            min_topological_vortex_dephasing_rate_hz: min_dephasing,
            max_topological_vortex_dephasing_rate_hz: max_dephasing,
            mean_chiral_majorana_mode_purity: sum_purity / n,
            min_chiral_majorana_mode_purity: min_purity,
            max_chiral_majorana_mode_purity: max_purity,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
