//! Parallel Rayon benchmark for Quantum LIDAR & Telemetry across 10,000 detection pulses.
//!
//! Simulates single-photon arrival statistics, detector timing jitter (< 50 ps),
//! dark count suppression (< 1 cps), high detection efficiency (> 98%), and sub-millimeter ranging precision.

use phonon_models::snspd::{NanowireGeometry, QuantumTelemetryModel, SPEED_OF_LIGHT};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark parameters and target configuration for quantum telemetry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumLidarBenchmarkConfig {
    /// True target range in meters.
    pub true_target_range_m: f64,
    /// Operating laser wavelength in nanometers (default 1550 nm).
    pub wavelength_nm: f64,
    /// Detector bias ratio alpha_bias = I_b / I_c.
    pub bias_ratio: f64,
    /// Number of emitted telemetry pulses to benchmark.
    pub num_pulses: usize,
}

impl Default for QuantumLidarBenchmarkConfig {
    fn default() -> Self {
        Self {
            true_target_range_m: 150.0,
            wavelength_nm: 1550.0,
            bias_ratio: 0.90,
            num_pulses: 10_000,
        }
    }
}

/// Results of the parallel quantum telemetry benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumLidarBenchmarkReport {
    /// True target range in meters.
    pub true_range_m: f64,
    /// Reconstructed target range from averaged photon ToF in meters.
    pub reconstructed_range_m: f64,
    /// Absolute ranging error in millimeters.
    pub ranging_error_mm: f64,
    /// Total number of pulses simulated.
    pub total_pulses: usize,
    /// Number of successful single-photon detections.
    pub detected_pulses: usize,
    /// Internal detection efficiency \u{03b7}_int evaluated.
    pub detection_efficiency: f64,
    /// Measured root-mean-square (RMS) timing jitter in picoseconds.
    pub measured_jitter_rms_ps: f64,
    /// Predicted theoretical detector jitter in picoseconds.
    pub theoretical_jitter_ps: f64,
    /// Dark count rate in counts per second (cps).
    pub dark_count_rate_cps: f64,
    /// Total elapsed simulation time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in pulses per second.
    pub throughput_pulses_per_sec: f64,
    /// Verification status flag: meets all quantum performance criteria.
    pub passed_criteria: bool,
}

/// Quantum LIDAR benchmark runner.
pub struct QuantumLidarBenchmarkRunner {
    pub geom: NanowireGeometry,
    pub telemetry: QuantumTelemetryModel,
    pub config: QuantumLidarBenchmarkConfig,
}

impl QuantumLidarBenchmarkRunner {
    pub fn new(
        geom: NanowireGeometry,
        telemetry: QuantumTelemetryModel,
        config: QuantumLidarBenchmarkConfig,
    ) -> Self {
        Self {
            geom,
            telemetry,
            config,
        }
    }

    /// Default runner for standard NbN detector.
    pub fn standard() -> Self {
        let geom = NanowireGeometry::standard_nbn();
        let telemetry = QuantumTelemetryModel::standard_nbn(&geom);
        Self::new(geom, telemetry, QuantumLidarBenchmarkConfig::default())
    }

    /// Runs parallel multi-threaded benchmark across N pulses.
    pub fn run(&self) -> QuantumLidarBenchmarkReport {
        let start_time = Instant::now();

        let num_pulses = self.config.num_pulses;
        let true_range_m = self.config.true_target_range_m;
        // True two-way time of flight: t_tof = 2 * z / c in seconds
        let true_tof_s = (2.0 * true_range_m) / SPEED_OF_LIGHT;
        let true_tof_ps = true_tof_s * 1e12;

        let i_b = self.geom.bias_current_ua(self.config.bias_ratio);
        let eta_int =
            self.telemetry
                .internal_quantum_efficiency(&self.geom, i_b, self.config.wavelength_nm);
        let dcr_cps = self.telemetry.dark_count_rate_cps(&self.geom, i_b);
        let theoretical_jitter_ps = self.telemetry.total_timing_jitter_ps(&self.geom);

        // Deterministic pseudo-random parallel generation for reproducible benchmark
        // Each pulse i produces a detection if hash_val < eta_int
        let detections: Vec<Option<f64>> = (0..num_pulses)
            .into_par_iter()
            .map(|idx| {
                // Multiplicative 64-bit congruential hash
                let h1 = (idx as u64)
                    .wrapping_mul(0x5851f42d4c957f2d)
                    .wrapping_add(0x14057b7ef767814f);
                let u1 = ((h1 >> 11) as f64) / ((1u64 << 53) as f64);

                if u1 <= eta_int {
                    // Box-Muller transform for Gaussian jitter: mean 0, std theoretical_jitter_ps
                    let h2 = (h1 ^ 0x9e3779b97f4a7c15)
                        .wrapping_mul(0xbf58476d1ce4e5b9)
                        .wrapping_add(0x94d049bb133111eb);
                    let h3 = (h2 ^ 0x9e3779b97f4a7c15)
                        .wrapping_mul(0x94d049bb133111eb)
                        .wrapping_add(0xbf58476d1ce4e5b9);

                    let u2 = (((h2 >> 11) as f64) / ((1u64 << 53) as f64)).clamp(1e-12, 1.0);
                    let u3 = (((h3 >> 11) as f64) / ((1u64 << 53) as f64)).clamp(0.0, 1.0);

                    let r = (-2.0 * u2.ln()).sqrt();
                    let theta = 2.0 * std::f64::consts::PI * u3;
                    let norm_sample = r * theta.cos();

                    let jitter_sample_ps = norm_sample * theoretical_jitter_ps;
                    let detected_tof_ps = true_tof_ps + jitter_sample_ps;
                    Some(detected_tof_ps)
                } else {
                    None
                }
            })
            .collect();

        let mut sum_tof_ps = 0.0_f64;
        let mut sum_sq_diff_ps = 0.0_f64;
        let mut detected_count = 0usize;

        for &tof_ps in detections.iter().flatten() {
            detected_count += 1;
            sum_tof_ps += tof_ps;
        }

        let mean_tof_ps = if detected_count > 0 {
            sum_tof_ps / detected_count as f64
        } else {
            true_tof_ps
        };

        for &tof_ps in detections.iter().flatten() {
            let diff = tof_ps - mean_tof_ps;
            sum_sq_diff_ps += diff * diff;
        }

        let measured_jitter_rms_ps = if detected_count > 1 {
            (sum_sq_diff_ps / (detected_count - 1) as f64).sqrt()
        } else {
            theoretical_jitter_ps
        };

        // Reconstruct target range
        let mean_tof_s = mean_tof_ps * 1e-12;
        let reconstructed_range_m = self.telemetry.time_of_flight_distance_m(mean_tof_s);
        let ranging_error_mm = (reconstructed_range_m - true_range_m).abs() * 1000.0;

        let elapsed = start_time.elapsed().as_secs_f64();
        let throughput = num_pulses as f64 / elapsed.max(1e-6);

        // Verification criteria:
        // 1. Detection efficiency > 95% (near theoretical > 98%)
        // 2. Timing jitter < 50 ps
        // 3. Dark count rate < 1.0 cps
        // 4. Sub-millimeter ranging precision: ranging_error_mm < 1.0 mm
        let efficiency_ok = (detected_count as f64 / num_pulses as f64) >= 0.95;
        let jitter_ok = measured_jitter_rms_ps < 50.0;
        let dcr_ok = dcr_cps < 1.0;
        let ranging_ok = ranging_error_mm < 1.0;
        let passed_criteria = efficiency_ok && jitter_ok && dcr_ok && ranging_ok;

        QuantumLidarBenchmarkReport {
            true_range_m,
            reconstructed_range_m,
            ranging_error_mm,
            total_pulses: num_pulses,
            detected_pulses: detected_count,
            detection_efficiency: detected_count as f64 / num_pulses as f64,
            measured_jitter_rms_ps,
            theoretical_jitter_ps,
            dark_count_rate_cps: dcr_cps,
            elapsed_seconds: elapsed,
            throughput_pulses_per_sec: throughput,
            passed_criteria,
        }
    }
}
