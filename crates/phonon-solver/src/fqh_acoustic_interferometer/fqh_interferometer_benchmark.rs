//! Multi-threaded Rayon benchmark runner for fractional quantum Hall acoustic interferometers
//! and anyonic braiding noise probes across 10,000 parameter sweeps.

use super::fqh_interferometer_solver::FqhInterferometerSolver;
use phonon_models::fqh_acoustic_interferometer::FqhInterferometerParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point for FQH acoustic interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FqhInterferometerSweepPoint {
    pub precision_error: f64,
    pub fano_factor: f64,
    pub fringe_visibility_pct: f64,
    pub coherence_length_um: f64,
    pub cross_correlation_db: f64,
    pub readout_snr_db: f64,
}

/// Comprehensive benchmark report for FQH acoustic interferometers.
#[derive(Debug, Clone, PartialEq)]
pub struct FqhInterferometerBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean charge precision error ($\\le 1.0\\times 10^{-4}$ required).
    pub mean_precision_error: f64,
    /// Maximum charge precision error.
    pub max_precision_error: f64,
    /// Mean evaluated Fano factor ($0.25 \\pm 1.0\\times 10^{-4}$ required).
    pub mean_fano_factor: f64,
    /// Mean fringe visibility in percent ($\\ge 90.0\\%$ required).
    pub mean_visibility_pct: f64,
    /// Minimum fringe visibility in percent.
    pub min_visibility_pct: f64,
    /// Mean phase coherence length in micrometers ($\\ge 25.0\\,\\mu\\text{m}$ required).
    pub mean_coherence_length_um: f64,
    /// Minimum phase coherence length in micrometers.
    pub min_coherence_length_um: f64,
    /// Mean noise cross-correlation in dB ($\\le -20.0\\text{ dB}$ required).
    pub mean_cross_correlation_db: f64,
    /// Maximum noise cross-correlation in dB.
    pub max_cross_correlation_db: f64,
    /// Mean readout SNR in dB ($\\ge 25.0\\text{ dB}$ required).
    pub mean_readout_snr_db: f64,
    /// Minimum readout SNR in dB.
    pub min_readout_snr_db: f64,
    /// Fraction of parameter sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for FQH acoustic interferometers.
#[derive(Debug, Clone)]
pub struct FqhInterferometerBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for FqhInterferometerBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl FqhInterferometerBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> FqhInterferometerBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<FqhInterferometerSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Parameter variations across physical operating envelopes:
                let nu = 2.5; // Moore-Read Pfaffian filling factor
                let t_mk = 10.0 + 35.0 * frac; // 10 to 45 mK
                let v_edge = 30_000.0 + 30_000.0 * pseudo_hash; // 30k to 60k m/s
                let tau_ns = 0.8 + 1.6 * frac; // 0.8 to 2.4 ns
                let v_saw = 8.0 + 24.0 * pseudo_hash; // 8 to 32 mV

                let params = FqhInterferometerParams {
                    filling_factor_nu: nu,
                    quasiparticle_charge_ratio: 0.25,
                    temperature_mk: t_mk,
                    edge_velocity_m_s: v_edge,
                    dephasing_time_ns: tau_ns,
                    saw_drive_amplitude_mv: v_saw,
                };

                let solver = FqhInterferometerSolver::new(params);
                let m = solver.solve();

                FqhInterferometerSweepPoint {
                    precision_error: m.charge_precision_error,
                    fano_factor: m.fano_factor,
                    fringe_visibility_pct: m.fringe_visibility_pct,
                    coherence_length_um: m.coherence_length_um,
                    cross_correlation_db: m.cross_correlation_db,
                    readout_snr_db: m.readout_snr_db,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1e-6);

        let mut sum_err = 0.0;
        let mut max_err = f64::MIN;

        let mut sum_fano = 0.0;

        let mut sum_vis = 0.0;
        let mut min_vis = f64::MAX;

        let mut sum_l = 0.0;
        let mut min_l = f64::MAX;

        let mut sum_corr = 0.0;
        let mut max_corr = f64::MIN;

        let mut sum_snr = 0.0;
        let mut min_snr = f64::MAX;

        let mut compliant_count = 0;

        for r in &results {
            sum_err += r.precision_error;
            if r.precision_error > max_err {
                max_err = r.precision_error;
            }

            sum_fano += r.fano_factor;

            sum_vis += r.fringe_visibility_pct;
            if r.fringe_visibility_pct < min_vis {
                min_vis = r.fringe_visibility_pct;
            }

            sum_l += r.coherence_length_um;
            if r.coherence_length_um < min_l {
                min_l = r.coherence_length_um;
            }

            sum_corr += r.cross_correlation_db;
            if r.cross_correlation_db > max_corr {
                max_corr = r.cross_correlation_db;
            }

            sum_snr += r.readout_snr_db;
            if r.readout_snr_db < min_snr {
                min_snr = r.readout_snr_db;
            }

            // Physical criteria validation:
            let is_compliant = r.precision_error <= 1.0e-4
                && (r.fano_factor - 0.25).abs() <= 1.0e-4
                && r.fringe_visibility_pct >= 90.0
                && r.coherence_length_um >= 25.0
                && r.cross_correlation_db <= -20.0
                && r.readout_snr_db >= 25.0;

            if is_compliant {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        FqhInterferometerBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_precision_error: sum_err / n,
            max_precision_error: max_err,
            mean_fano_factor: sum_fano / n,
            mean_visibility_pct: sum_vis / n,
            min_visibility_pct: min_vis,
            mean_coherence_length_um: sum_l / n,
            min_coherence_length_um: min_l,
            mean_cross_correlation_db: sum_corr / n,
            max_cross_correlation_db: max_corr,
            mean_readout_snr_db: sum_snr / n,
            min_readout_snr_db: min_snr,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
