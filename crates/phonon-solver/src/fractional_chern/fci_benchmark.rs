//! Multi-threaded Rayon benchmark for Fractional Chern Insulators (FCIs),
//! moiré flat bands, and anyonic quantum state teleportation.

use super::anyonic_teleportation_solver::AnyonicTeleportationSolver;
use super::many_body_fci_solver::ManyBodyFciSolver;
use phonon_models::fractional_chern::{AnyonQudit, FractionalFilling, MoireLatticeParams};
use rayon::prelude::*;
use std::time::Instant;

/// Single FCI sweep point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FciSweepPoint {
    pub twist_angle_deg: f64,
    pub spectral_gap_ev: f64,
    pub teleportation_fidelity: f64,
    pub fubini_study_ratio: f64,
}

/// Comprehensive benchmark report for Fractional Chern Insulators and Anyonic Teleportation.
#[derive(Debug, Clone, PartialEq)]
pub struct FciBenchmarkReport {
    /// Total number of parallel sweeps executed.
    pub total_cycles: usize,
    /// Elapsed benchmark wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Throughput in cycles per second.
    pub throughput_cycles_per_sec: f64,
    /// Mean anyonic teleportation fidelity $\mathcal{F}$ ($\\ge 99\\%$ required).
    pub mean_fidelity: f64,
    /// Minimum anyonic teleportation fidelity $\mathcal{F}$ ($\\ge 99\\%$ required).
    pub min_fidelity: f64,
    /// Mean topological neutral spectral gap in eV ($\\ge 1.0\\text{ meV}$ required).
    pub mean_spectral_gap_ev: f64,
    /// Minimum topological neutral spectral gap in eV.
    pub min_spectral_gap_ev: f64,
    /// Mean Fubini-Study trace condition ratio $\\eta_{\\mathrm{FS}}$.
    pub mean_fubini_study_ratio: f64,
    /// Fraction of sweeps achieving $\mathcal{F} \ge 99\%$ and $\Delta \ge 1.0\text{ meV}$.
    pub high_fidelity_fraction: f64,
}

/// Parallel benchmark runner for FCIs and anyonic teleportation.
#[derive(Debug, Clone)]
pub struct FciBenchmarkRunner {
    pub total_cycles: usize,
    pub base_params: MoireLatticeParams,
}

impl Default for FciBenchmarkRunner {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl FciBenchmarkRunner {
    pub fn new(total_cycles: usize) -> Self {
        Self {
            total_cycles,
            base_params: MoireLatticeParams::twisted_mote2_125(),
        }
    }

    /// Runs parallel Rayon benchmark across `total_cycles` configurations.
    pub fn run_parallel_benchmark(&self) -> FciBenchmarkReport {
        let start = Instant::now();
        let base = self.base_params;

        let sweep_results: Vec<FciSweepPoint> = (0..self.total_cycles)
            .into_par_iter()
            .map(|idx| {
                // Vary twist angle between 1.18° and 1.32°
                let twist = 1.18 + (idx as f64 % 29.0) * 0.005;
                let mut p = base;
                p.twist_angle_deg = twist;

                let filling = match idx % 4 {
                    0 => FractionalFilling::OneThird,
                    1 => FractionalFilling::TwoThirds,
                    2 => FractionalFilling::OneFifth,
                    _ => FractionalFilling::TwoFifths,
                };

                let spec_solver = ManyBodyFciSolver::new(p, filling, 1);
                let spec_res = spec_solver.solve();

                // Teleportation test
                let d = filling.denominator();
                let input_state = AnyonQudit::equal_superposition(d);
                let temp_k = 0.05 + (idx as f64 % 10.0) * 0.01; // 50 to 140 mK
                let channel_len = 200.0 + (idx as f64 % 50.0) * 10.0; // 200 to 700 nm

                let tele_solver = AnyonicTeleportationSolver::new(p, filling, channel_len, temp_k);
                let bsm_m = idx % d;
                let bsm_n = (idx / d) % d;
                let tele_res = tele_solver.execute_teleportation(&input_state, bsm_m, bsm_n);

                FciSweepPoint {
                    twist_angle_deg: twist,
                    spectral_gap_ev: spec_res.spectral_gap_ev,
                    teleportation_fidelity: tele_res.state_fidelity,
                    fubini_study_ratio: spec_res.fubini_study_ratio,
                }
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (self.total_cycles as f64) / elapsed.max(1e-6);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut sum_fs = 0.0;
        let mut high_fid_count = 0;

        for pt in &sweep_results {
            sum_fid += pt.teleportation_fidelity;
            if pt.teleportation_fidelity < min_fid {
                min_fid = pt.teleportation_fidelity;
            }

            sum_gap += pt.spectral_gap_ev;
            if pt.spectral_gap_ev < min_gap {
                min_gap = pt.spectral_gap_ev;
            }

            sum_fs += pt.fubini_study_ratio;

            if pt.teleportation_fidelity >= 0.99 && pt.spectral_gap_ev >= 0.001 {
                high_fid_count += 1;
            }
        }

        let n = self.total_cycles as f64;
        let mean_fid = sum_fid / n;
        let mean_gap = sum_gap / n;
        let mean_fs = sum_fs / n;
        let high_fid_frac = (high_fid_count as f64) / n;

        FciBenchmarkReport {
            total_cycles: self.total_cycles,
            elapsed_seconds: elapsed,
            throughput_cycles_per_sec: throughput,
            mean_fidelity: mean_fid,
            min_fidelity: min_fid,
            mean_spectral_gap_ev: mean_gap,
            min_spectral_gap_ev: min_gap,
            mean_fubini_study_ratio: mean_fs,
            high_fidelity_fraction: high_fid_frac,
        }
    }
}
