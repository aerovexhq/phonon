#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic twisted bilayer moire
//! polariton superlattices and flat-band phonon superconductors across multi-threaded Rayon workers.

use crate::twisted_bilayer_moire_polariton::TwistedBilayerMoirePolaritonSolver;
use phonon_models::twisted_bilayer_moire_polariton::{
    TwistedBilayerMoirePolaritonMetrics, TwistedBilayerMoirePolaritonParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum acoustic twisted bilayer moire polariton parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwistedBilayerMoireBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_polariton_superconducting_fidelity: f64,
    pub min_polariton_superconducting_fidelity: f64,
    pub max_polariton_superconducting_fidelity: f64,
    pub mean_flat_band_group_velocity_mps: f64,
    pub min_flat_band_group_velocity_mps: f64,
    pub max_flat_band_group_velocity_mps: f64,
    pub mean_tc_enhancement_factor: f64,
    pub min_tc_enhancement_factor: f64,
    pub max_tc_enhancement_factor: f64,
    pub mean_inter_valley_crosstalk_isolation_db: f64,
    pub min_inter_valley_crosstalk_isolation_db: f64,
    pub max_inter_valley_crosstalk_isolation_db: f64,
    pub mean_magic_angle_alignment_tolerance_fraction: f64,
    pub min_magic_angle_alignment_tolerance_fraction: f64,
    pub max_magic_angle_alignment_tolerance_fraction: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct TwistedBilayerMoireBenchmarkRunner;

impl TwistedBilayerMoireBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> TwistedBilayerMoireBenchmarkResult {
        let sweep_params: Vec<TwistedBilayerMoirePolaritonParams> = (0..cycles)
            .map(|i| {
                let twist_angle_degrees = 0.85 + 0.50 * (((i * 7) % 50) as f64 / 50.0);
                let interlayer_tunneling_energy_mev =
                    55.0 + 90.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_deformation_potential_ev =
                    2.5 + 12.0 * (((i * 11) % 40) as f64 / 40.0);
                let moire_acoustic_frequency_ghz =
                    0.8 + 8.8 * (((i * 17) % 35) as f64 / 35.0);
                let cryogenic_temperature_mk =
                    2.0 + 45.0 * (((i * 19) % 45) as f64 / 45.0);
                let electron_phonon_coupling_lambda =
                    0.30 + 2.10 * (((i * 23) % 30) as f64 / 30.0);
                let inter_valley_coherence_length_nm =
                    30.0 + 260.0 * (((i * 29) % 32) as f64 / 32.0);
                let superconducting_channel_length_um =
                    1.5 + 18.0 * (((i * 31) % 25) as f64 / 25.0);

                TwistedBilayerMoirePolaritonParams::new(
                    twist_angle_degrees,
                    interlayer_tunneling_energy_mev,
                    acoustic_deformation_potential_ev,
                    moire_acoustic_frequency_ghz,
                    cryogenic_temperature_mk,
                    electron_phonon_coupling_lambda,
                    inter_valley_coherence_length_nm,
                    superconducting_channel_length_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<TwistedBilayerMoirePolaritonMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = TwistedBilayerMoirePolaritonSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_velocity = 0.0;
        let mut min_velocity = f64::MAX;
        let mut max_velocity = f64::MIN;

        let mut sum_tc = 0.0;
        let mut min_tc = f64::MAX;
        let mut max_tc = f64::MIN;

        let mut sum_isolation = 0.0;
        let mut min_isolation = f64::MAX;
        let mut max_isolation = f64::MIN;

        let mut sum_tolerance = 0.0;
        let mut min_tolerance = f64::MAX;
        let mut max_tolerance = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.polariton_superconducting_fidelity;
            if m.polariton_superconducting_fidelity < min_fidelity {
                min_fidelity = m.polariton_superconducting_fidelity;
            }
            if m.polariton_superconducting_fidelity > max_fidelity {
                max_fidelity = m.polariton_superconducting_fidelity;
            }

            sum_velocity += m.flat_band_group_velocity_mps;
            if m.flat_band_group_velocity_mps < min_velocity {
                min_velocity = m.flat_band_group_velocity_mps;
            }
            if m.flat_band_group_velocity_mps > max_velocity {
                max_velocity = m.flat_band_group_velocity_mps;
            }

            sum_tc += m.tc_enhancement_factor;
            if m.tc_enhancement_factor < min_tc {
                min_tc = m.tc_enhancement_factor;
            }
            if m.tc_enhancement_factor > max_tc {
                max_tc = m.tc_enhancement_factor;
            }

            sum_isolation += m.inter_valley_crosstalk_isolation_db;
            if m.inter_valley_crosstalk_isolation_db < min_isolation {
                min_isolation = m.inter_valley_crosstalk_isolation_db;
            }
            if m.inter_valley_crosstalk_isolation_db > max_isolation {
                max_isolation = m.inter_valley_crosstalk_isolation_db;
            }

            sum_tolerance += m.magic_angle_alignment_tolerance_fraction;
            if m.magic_angle_alignment_tolerance_fraction < min_tolerance {
                min_tolerance = m.magic_angle_alignment_tolerance_fraction;
            }
            if m.magic_angle_alignment_tolerance_fraction > max_tolerance {
                max_tolerance = m.magic_angle_alignment_tolerance_fraction;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        TwistedBilayerMoireBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_polariton_superconducting_fidelity: sum_fidelity / n,
            min_polariton_superconducting_fidelity: min_fidelity,
            max_polariton_superconducting_fidelity: max_fidelity,
            mean_flat_band_group_velocity_mps: sum_velocity / n,
            min_flat_band_group_velocity_mps: min_velocity,
            max_flat_band_group_velocity_mps: max_velocity,
            mean_tc_enhancement_factor: sum_tc / n,
            min_tc_enhancement_factor: min_tc,
            max_tc_enhancement_factor: max_tc,
            mean_inter_valley_crosstalk_isolation_db: sum_isolation / n,
            min_inter_valley_crosstalk_isolation_db: min_isolation,
            max_inter_valley_crosstalk_isolation_db: max_isolation,
            mean_magic_angle_alignment_tolerance_fraction: sum_tolerance / n,
            min_magic_angle_alignment_tolerance_fraction: min_tolerance,
            max_magic_angle_alignment_tolerance_fraction: max_tolerance,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
