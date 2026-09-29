#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for quantum acoustic metasurface
//! holography and chiral phonon beamforming arrays across multi-threaded Rayon workers.

use crate::chiral_holographic_beamforming::beamforming_solver::ChiralHolographicBeamformingSolver;
use phonon_models::chiral_holographic_beamforming::{
    ChiralHolographicBeamformingMetrics, ChiralHolographicBeamformingParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for chiral holographic beamforming parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolographicBeamformingBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_holographic_reconstruction_fidelity: f64,
    pub min_holographic_reconstruction_fidelity: f64,
    pub max_holographic_reconstruction_fidelity: f64,
    pub mean_acoustic_beam_directivity_db: f64,
    pub min_acoustic_beam_directivity_db: f64,
    pub max_acoustic_beam_directivity_db: f64,
    pub mean_beam_steering_angular_resolution_deg: f64,
    pub min_beam_steering_angular_resolution_deg: f64,
    pub max_beam_steering_angular_resolution_deg: f64,
    pub mean_side_lobe_suppression_ratio_db: f64,
    pub min_side_lobe_suppression_ratio_db: f64,
    pub max_side_lobe_suppression_ratio_db: f64,
    pub mean_acoustic_mode_insertion_loss_db: f64,
    pub min_acoustic_mode_insertion_loss_db: f64,
    pub max_acoustic_mode_insertion_loss_db: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner executing large-scale parallel parameter sweeps via Rayon.
pub struct HolographicBeamformingBenchmarkRunner;

impl HolographicBeamformingBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> HolographicBeamformingBenchmarkResult {
        let sweep_params: Vec<ChiralHolographicBeamformingParams> = (0..cycles)
            .map(|i| {
                let metasurface_elements_count = 44 + 2 * (i % 7); // 44..56 (default 48)
                let element_spacing_um = 0.82 + 0.06 * ((i % 11) as f64 / 11.0); // 0.82..0.88 (default 0.85)
                let operating_frequency_ghz = 3.65 + 0.30 * ((i % 13) as f64 / 13.0); // 3.65..3.95 (default 3.8)
                let synthetic_gauge_phase_gradient_rad_per_um =
                    3.05 + 0.30 * ((i % 17) as f64 / 17.0); // 3.05..3.35 (default 3.2)
                let piezoelectric_coupling_efficiency = 0.895 + 0.030 * ((i % 19) as f64 / 19.0); // 0.895..0.925 (default 0.91)
                let sub_diffraction_focusing_ratio = 2.02 + 0.16 * ((i % 23) as f64 / 23.0); // 2.02..2.18 (default 2.10)
                let cryogenic_temperature_mk = 18.0 + 4.0 * ((i % 29) as f64 / 29.0); // 18.0..22.0 (default 20.0)
                let chiral_isolation_db = 34.5 + 3.0 * ((i % 31) as f64 / 31.0); // 34.5..37.5 (default 36.0)

                ChiralHolographicBeamformingParams::new(
                    metasurface_elements_count,
                    element_spacing_um,
                    operating_frequency_ghz,
                    synthetic_gauge_phase_gradient_rad_per_um,
                    piezoelectric_coupling_efficiency,
                    sub_diffraction_focusing_ratio,
                    cryogenic_temperature_mk,
                    chiral_isolation_db,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<ChiralHolographicBeamformingMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = ChiralHolographicBeamformingSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fidelity = 0.0;
        let mut min_fidelity = f64::MAX;
        let mut max_fidelity = f64::MIN;

        let mut sum_directivity = 0.0;
        let mut min_directivity = f64::MAX;
        let mut max_directivity = f64::MIN;

        let mut sum_angular_res = 0.0;
        let mut min_angular_res = f64::MAX;
        let mut max_angular_res = f64::MIN;

        let mut sum_slsr = 0.0;
        let mut min_slsr = f64::MAX;
        let mut max_slsr = f64::MIN;

        let mut sum_loss = 0.0;
        let mut min_loss = f64::MAX;
        let mut max_loss = f64::MIN;

        let mut compliant_count = 0;

        for m in &metrics {
            sum_fidelity += m.holographic_reconstruction_fidelity;
            if m.holographic_reconstruction_fidelity < min_fidelity {
                min_fidelity = m.holographic_reconstruction_fidelity;
            }
            if m.holographic_reconstruction_fidelity > max_fidelity {
                max_fidelity = m.holographic_reconstruction_fidelity;
            }

            sum_directivity += m.acoustic_beam_directivity_db;
            if m.acoustic_beam_directivity_db < min_directivity {
                min_directivity = m.acoustic_beam_directivity_db;
            }
            if m.acoustic_beam_directivity_db > max_directivity {
                max_directivity = m.acoustic_beam_directivity_db;
            }

            sum_angular_res += m.beam_steering_angular_resolution_deg;
            if m.beam_steering_angular_resolution_deg < min_angular_res {
                min_angular_res = m.beam_steering_angular_resolution_deg;
            }
            if m.beam_steering_angular_resolution_deg > max_angular_res {
                max_angular_res = m.beam_steering_angular_resolution_deg;
            }

            sum_slsr += m.side_lobe_suppression_ratio_db;
            if m.side_lobe_suppression_ratio_db < min_slsr {
                min_slsr = m.side_lobe_suppression_ratio_db;
            }
            if m.side_lobe_suppression_ratio_db > max_slsr {
                max_slsr = m.side_lobe_suppression_ratio_db;
            }

            sum_loss += m.acoustic_mode_insertion_loss_db;
            if m.acoustic_mode_insertion_loss_db < min_loss {
                min_loss = m.acoustic_mode_insertion_loss_db;
            }
            if m.acoustic_mode_insertion_loss_db > max_loss {
                max_loss = m.acoustic_mode_insertion_loss_db;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        HolographicBeamformingBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_holographic_reconstruction_fidelity: sum_fidelity / n,
            min_holographic_reconstruction_fidelity: min_fidelity,
            max_holographic_reconstruction_fidelity: max_fidelity,
            mean_acoustic_beam_directivity_db: sum_directivity / n,
            min_acoustic_beam_directivity_db: min_directivity,
            max_acoustic_beam_directivity_db: max_directivity,
            mean_beam_steering_angular_resolution_deg: sum_angular_res / n,
            min_beam_steering_angular_resolution_deg: min_angular_res,
            max_beam_steering_angular_resolution_deg: max_angular_res,
            mean_side_lobe_suppression_ratio_db: sum_slsr / n,
            min_side_lobe_suppression_ratio_db: min_slsr,
            max_side_lobe_suppression_ratio_db: max_slsr,
            mean_acoustic_mode_insertion_loss_db: sum_loss / n,
            min_acoustic_mode_insertion_loss_db: min_loss,
            max_acoustic_mode_insertion_loss_db: max_loss,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
