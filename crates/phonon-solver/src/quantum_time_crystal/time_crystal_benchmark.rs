//! Multi-threaded Rayon benchmark runner for Floquet quantum time crystals,
//! lifetime tau >= 1000 cycles, and spectral rigidity contrast >= 20.0 dB across 10,000 sweeps.

use super::floquet_time_crystal_solver::FloquetTimeCrystalSolver;
use phonon_models::quantum_time_crystal::QuantumTimeCrystalParams;
use rayon::prelude::*;
use std::time::Instant;

/// Evaluated parameter sweep point in Floquet quantum time crystals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeCrystalSweepPoint {
    pub lifetime_cycles: f64,
    pub spectral_rigidity_contrast_db: f64,
    pub quality_factor: f64,
    pub subharmonic_freq_mhz: f64,
    pub fourier_amplitude: f64,
    pub memory_fidelity: f64,
    pub stability_allan_dev: f64,
}

/// Comprehensive benchmark report for Floquet quantum time crystals.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeCrystalBenchmarkReport {
    /// Total parameter sweeps executed.
    pub total_cycles: usize,
    /// Wall-clock elapsed time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in parameter sweeps per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean time crystal lifetime in driving cycles ($\ge 1000\text{ cycles}$ required).
    pub mean_lifetime_cycles: f64,
    /// Minimum time crystal lifetime in driving cycles.
    pub min_lifetime_cycles: f64,
    /// Maximum time crystal lifetime in driving cycles.
    pub max_lifetime_cycles: f64,
    /// Mean subharmonic spectral rigidity contrast in decibels ($\ge 20.0\text{ dB}$ required).
    pub mean_contrast_db: f64,
    /// Minimum subharmonic spectral rigidity contrast in decibels.
    pub min_contrast_db: f64,
    /// Maximum subharmonic spectral rigidity contrast in decibels.
    pub max_contrast_db: f64,
    /// Mean memory state fidelity ($\ge 90.0\%$ required).
    pub mean_memory_fidelity: f64,
    /// Minimum memory state fidelity.
    pub min_memory_fidelity: f64,
    /// Mean subharmonic quality factor.
    pub mean_quality_factor: f64,
    /// Fraction of sweeps satisfying all physical bounds (target: 1.0 = 100%).
    pub compliance_fraction: f64,
}

/// Multi-threaded Rayon benchmark runner for Floquet quantum time crystals.
#[derive(Debug, Clone)]
pub struct TimeCrystalBenchmarkRunner {
    pub total_cycles: usize,
}

impl Default for TimeCrystalBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl TimeCrystalBenchmarkRunner {
    /// Creates a new benchmark runner with the specified cycle count.
    pub fn new(total_cycles: usize) -> Self {
        Self { total_cycles }
    }

    /// Executes the multi-threaded parameter sweep benchmark.
    pub fn run_benchmark(&self) -> TimeCrystalBenchmarkReport {
        let start = Instant::now();
        let total = self.total_cycles;

        let results: Vec<TimeCrystalSweepPoint> = (0..total)
            .into_par_iter()
            .map(|idx| {
                let frac = idx as f64 / total.max(1) as f64;
                let pseudo_hash =
                    ((idx * 1664525 + 1013904223) & 0x7FFF_FFFF) as f64 / 2147483647.0;

                // Varied driving period & disorder
                let t_drive = 4.0 + 12.0 * frac; // 4.0 to 16.0 ns
                let eps = 0.02 + 0.06 * pseudo_hash; // 0.02 to 0.08
                let w = 2.2 + 2.0 * frac; // 2.2 to 4.2
                let j0 = 12.0 + 30.0 * pseudo_hash; // 12.0 to 42.0 MHz
                let gamma = 0.5 + 1.8 * (1.0 - frac); // 0.5 to 2.3 kHz
                let t_bath = 0.03 + 0.15 * pseudo_hash; // 0.03 to 0.18 K

                let params = QuantumTimeCrystalParams {
                    drive_period_ns: t_drive,
                    pulse_imperfection_epsilon: eps,
                    disorder_strength_w: w,
                    interaction_coupling_j0_mhz: j0,
                    acoustic_damping_rate_khz: gamma,
                    thermal_bath_temp_k: t_bath,
                    ..Default::default()
                };

                let solver = FloquetTimeCrystalSolver::new(params);
                let metrics = solver.solve();

                TimeCrystalSweepPoint {
                    lifetime_cycles: metrics.time_crystal_lifetime_cycles,
                    spectral_rigidity_contrast_db: metrics.spectral_rigidity_contrast_db,
                    quality_factor: metrics.spectral_rigidity_quality_factor,
                    subharmonic_freq_mhz: metrics.subharmonic_frequency_mhz,
                    fourier_amplitude: metrics.fourier_peak_amplitude,
                    memory_fidelity: metrics.subharmonic_memory_fidelity,
                    stability_allan_dev: metrics.fractional_frequency_stability,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = total as f64 / elapsed.max(1.0e-6);

        let mut sum_tau = 0.0;
        let mut min_tau = f64::MAX;
        let mut max_tau = f64::MIN;

        let mut sum_c = 0.0;
        let mut min_c = f64::MAX;
        let mut max_c = f64::MIN;

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_q = 0.0;
        let mut compliant_count = 0usize;

        for pt in &results {
            sum_tau += pt.lifetime_cycles;
            if pt.lifetime_cycles < min_tau {
                min_tau = pt.lifetime_cycles;
            }
            if pt.lifetime_cycles > max_tau {
                max_tau = pt.lifetime_cycles;
            }

            sum_c += pt.spectral_rigidity_contrast_db;
            if pt.spectral_rigidity_contrast_db < min_c {
                min_c = pt.spectral_rigidity_contrast_db;
            }
            if pt.spectral_rigidity_contrast_db > max_c {
                max_c = pt.spectral_rigidity_contrast_db;
            }

            sum_fid += pt.memory_fidelity;
            if pt.memory_fidelity < min_fid {
                min_fid = pt.memory_fidelity;
            }

            sum_q += pt.quality_factor;

            // Strict compliance criteria:
            // 1. Lifetime >= 1000 cycles
            // 2. Contrast >= 20.0 dB
            // 3. Memory fidelity >= 90.0%
            // 4. Allan deviation stability <= 1e-11
            if pt.lifetime_cycles >= 1000.0
                && pt.spectral_rigidity_contrast_db >= 20.0
                && pt.memory_fidelity >= 0.90
                && pt.stability_allan_dev <= 1.0e-11
            {
                compliant_count += 1;
            }
        }

        let n = total as f64;
        TimeCrystalBenchmarkReport {
            total_cycles: total,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_lifetime_cycles: sum_tau / n,
            min_lifetime_cycles: min_tau,
            max_lifetime_cycles: max_tau,
            mean_contrast_db: sum_c / n,
            min_contrast_db: min_c,
            max_contrast_db: max_c,
            mean_memory_fidelity: sum_fid / n,
            min_memory_fidelity: min_fid,
            mean_quality_factor: sum_q / n,
            compliance_fraction: compliant_count as f64 / n,
        }
    }
}
