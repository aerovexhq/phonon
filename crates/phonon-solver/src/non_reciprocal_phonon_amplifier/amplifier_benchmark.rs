//! Parallel parameter sweep benchmark suite for non-reciprocal topological
//! phonon amplification and directional quantum routing.

use crate::non_reciprocal_phonon_amplifier::NonReciprocalPhononAmplifierSolver;
use phonon_models::non_reciprocal_phonon_amplifier::{
    NonReciprocalAmplifierMetrics, NonReciprocalAmplifierParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark summary report for non-reciprocal phonon amplifier parameter sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AmplifierBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_forward_gain_db: f64,
    pub min_forward_gain_db: f64,
    pub mean_backward_isolation_db: f64,
    pub min_backward_isolation_db: f64,
    pub mean_added_noise_photons: f64,
    pub max_added_noise_photons: f64,
    pub mean_bandwidth_mhz: f64,
    pub min_bandwidth_mhz: f64,
    pub mean_routing_fidelity: f64,
    pub min_routing_fidelity: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct AmplifierBenchmarkRunner;

impl AmplifierBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles (e.g. 10,000).
    pub fn run_benchmark(cycles: usize) -> AmplifierBenchmarkResult {
        let sweep_params: Vec<NonReciprocalAmplifierParams> = (0..cycles)
            .map(|i| {
                let frac = (i as f64) / (cycles as f64);
                let freq_ghz = 2.0 + 8.0 * frac; // 2.0 to 10.0 GHz
                let pump_mod_mhz = 20.0 + 100.0 * ((i % 100) as f64 / 100.0); // 20.0 to 120.0 MHz
                let phase_grad_rad = 1.20 + 0.80 * ((i % 70) as f64 / 70.0); // 1.20 to 2.00 rad
                let coupling_mhz = 16.0 + 20.0 * ((i % 80) as f64 / 80.0); // 16.0 to 36.0 MHz
                let loss_mhz = 0.20 + 1.20 * frac; // 0.20 to 1.40 MHz
                let hopping_mhz = 15.0 + 30.0 * ((i % 60) as f64 / 60.0); // 15.0 to 45.0 MHz
                let pump_power_mw = 3.0 + 15.0 * ((i % 50) as f64 / 50.0); // 3.0 to 18.0 mW
                let temp_mk = 5.0 + 25.0 * ((i % 90) as f64 / 90.0); // 5.0 to 30.0 mK

                NonReciprocalAmplifierParams::new(
                    freq_ghz,
                    pump_mod_mhz,
                    phase_grad_rad,
                    coupling_mhz,
                    loss_mhz,
                    hopping_mhz,
                    pump_power_mw,
                    temp_mk,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<NonReciprocalAmplifierMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = NonReciprocalPhononAmplifierSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_gain = 0.0;
        let mut min_gain = f64::MAX;
        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut sum_noise = 0.0;
        let mut max_noise = f64::MIN;
        let mut sum_bw = 0.0;
        let mut min_bw = f64::MAX;
        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut compliant_count = 0;

        for m in &metrics {
            sum_gain += m.forward_gain_db;
            if m.forward_gain_db < min_gain {
                min_gain = m.forward_gain_db;
            }

            sum_iso += m.backward_isolation_db;
            if m.backward_isolation_db < min_iso {
                min_iso = m.backward_isolation_db;
            }

            sum_noise += m.added_noise_photons;
            if m.added_noise_photons > max_noise {
                max_noise = m.added_noise_photons;
            }

            sum_bw += m.instantaneous_bandwidth_mhz;
            if m.instantaneous_bandwidth_mhz < min_bw {
                min_bw = m.instantaneous_bandwidth_mhz;
            }

            sum_fid += m.directional_routing_fidelity;
            if m.directional_routing_fidelity < min_fid {
                min_fid = m.directional_routing_fidelity;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        AmplifierBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_forward_gain_db: sum_gain / n,
            min_forward_gain_db: min_gain,
            mean_backward_isolation_db: sum_iso / n,
            min_backward_isolation_db: min_iso,
            mean_added_noise_photons: sum_noise / n,
            max_added_noise_photons: max_noise,
            mean_bandwidth_mhz: sum_bw / n,
            min_bandwidth_mhz: min_bw,
            mean_routing_fidelity: sum_fid / n,
            min_routing_fidelity: min_fid,
            physical_compliance_fraction: (compliant_count as f64) / n,
        }
    }
}
