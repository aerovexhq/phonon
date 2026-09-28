//! Parallel Rayon benchmark for Fractional Quantum Hall interferometry and non-Abelian anyon braiding.
//!
//! Evaluates 10,000 braid sequences in parallel, validating braiding phase fidelity >= 99.9%,
//! even/odd anyon visibility collapse, and high simulation throughput.

use super::braiding_trajectory_solver::{BraidOperation, BraidingTrajectorySolver};
use super::interferometry_solver::InterferometrySolver;
use phonon_models::fqh::{FabryPerotInterferometer, LuttingerEdgeModel};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark configuration for FQH interferometry and braiding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FqhBenchmarkConfig {
    /// Total number of braiding sequences to evaluate (default 10,000).
    pub num_sequences: usize,
    /// Number of gates per sequence (default 8).
    pub sequence_length: usize,
    /// Operating magnetic field in Tesla.
    pub magnetic_field_tesla: f64,
}

impl Default for FqhBenchmarkConfig {
    fn default() -> Self {
        Self {
            num_sequences: 10_000,
            sequence_length: 8,
            magnetic_field_tesla: 3.0,
        }
    }
}

/// Report containing comprehensive benchmark metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct FqhBenchmarkReport {
    /// Total braid sequences evaluated.
    pub total_sequences: usize,
    /// Mean state fidelity across all sequences.
    pub mean_fidelity: f64,
    /// Minimum fidelity observed across all sequences.
    pub min_fidelity: f64,
    /// Even-bulk-anyon interference visibility (N_\u{03c3} = 0 or 2).
    pub even_anyon_visibility: f64,
    /// Odd-bulk-anyon interference visibility (N_\u{03c3} = 1 or 3).
    pub odd_anyon_visibility: f64,
    /// Even-to-odd visibility extinction ratio.
    pub visibility_extinction_ratio: f64,
    /// Total elapsed wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in braid sequences per second.
    pub throughput_sequences_per_sec: f64,
    /// Verification status flag.
    pub passed_criteria: bool,
}

/// FQH benchmark runner.
pub struct FqhBenchmarkRunner {
    pub config: FqhBenchmarkConfig,
}

impl FqhBenchmarkRunner {
    pub fn new(config: FqhBenchmarkConfig) -> Self {
        Self { config }
    }

    /// Default runner for Moore-Read \u{03bd} = 5/2 benchmark.
    pub fn standard() -> Self {
        Self::new(FqhBenchmarkConfig::default())
    }

    /// Runs parallel multi-threaded benchmark.
    pub fn run(&self) -> FqhBenchmarkReport {
        let start_time = Instant::now();

        let num_seq = self.config.num_sequences;
        let seq_len = self.config.sequence_length;

        let solver = BraidingTrajectorySolver::cryogenic_low_noise();

        // 1. Parallel evaluation of 10,000 braiding sequences
        let fidelities: Vec<f64> = (0..num_seq)
            .into_par_iter()
            .map(|seq_idx| {
                // Generate deterministic pseudo-random braid sequence for reproducible benchmark
                let mut sequence = Vec::with_capacity(seq_len);
                let mut h = (seq_idx as u64)
                    .wrapping_mul(0x517cc1b727220a95)
                    .wrapping_add(0x4d34d3d23f3e1b05);

                for _ in 0..seq_len {
                    h = h
                        .wrapping_mul(0xbf58476d1ce4e5b9)
                        .wrapping_add(0x94d049bb133111eb);
                    let op_idx = (h >> 60) % 4;
                    let op = match op_idx {
                        0 => BraidOperation::R12,
                        1 => BraidOperation::B23,
                        2 => BraidOperation::R12Dagger,
                        _ => BraidOperation::B23Dagger,
                    };
                    sequence.push(op);
                }

                let traj = solver.run_braid_sequence(&sequence);
                traj.fidelity
            })
            .collect();

        let mut sum_fid = 0.0_f64;
        let mut min_fid = 1.0_f64;

        for &f in &fidelities {
            sum_fid += f;
            if f < min_fid {
                min_fid = f;
            }
        }

        let mean_fidelity = sum_fid / num_seq as f64;

        // 2. Interferometry visibility verification on Moore-Read \u{03bd} = 5/2
        let edge = LuttingerEdgeModel::moore_read_five_halves(self.config.magnetic_field_tesla);
        let interferometer = FabryPerotInterferometer::standard_ab_regime();
        let if_solver = InterferometrySolver::new(interferometer, edge);

        let even_scan = if_solver.scan_magnetic_field(3.0, 3.02, 50, 0.0, 0, false);
        let odd_scan = if_solver.scan_magnetic_field(3.0, 3.02, 50, 0.0, 1, false);

        let even_anyon_visibility = even_scan.visibility;
        let odd_anyon_visibility = odd_scan.visibility;
        let visibility_extinction_ratio = if odd_anyon_visibility > 1e-6 {
            even_anyon_visibility / odd_anyon_visibility
        } else {
            1000.0 // Practically infinite extinction
        };

        let elapsed = start_time.elapsed().as_secs_f64();
        let throughput = num_seq as f64 / elapsed.max(1e-6);

        // Verification criteria:
        // 1. Mean fidelity >= 99.9%
        // 2. Minimum fidelity >= 99.0%
        // 3. Even anyon visibility >= 0.80
        // 4. Odd anyon visibility < 0.05 (near zero)
        // 5. Visibility extinction ratio >= 50
        let fidelity_ok = mean_fidelity >= 0.999 && min_fid >= 0.990;
        let visibility_ok = even_anyon_visibility >= 0.80 && odd_anyon_visibility < 0.05;
        let extinction_ok = visibility_extinction_ratio >= 50.0;
        let passed_criteria = fidelity_ok && visibility_ok && extinction_ok;

        FqhBenchmarkReport {
            total_sequences: num_seq,
            mean_fidelity,
            min_fidelity: min_fid,
            even_anyon_visibility,
            odd_anyon_visibility,
            visibility_extinction_ratio,
            elapsed_seconds: elapsed,
            throughput_sequences_per_sec: throughput,
            passed_criteria,
        }
    }
}
